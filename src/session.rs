use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context as AnyhowContext, Result};
use crossterm::style::Stylize;
use indicatif::{ProgressBar, ProgressStyle};
use llm::chat::{ChatMessage, ChatMessageBuilder, ChatProvider, ChatRole};

use crate::commands::CommandProcessor;
use crate::memory::MemoryManager;
use crate::parser::{self, ToolCall};

/// Holds the result of a single tool execution.
#[derive(Debug)]
pub struct ToolExecutionResult {
    pub tool_call_str: String,
    pub success: bool,
    pub output: String,
}

impl fmt::Display for ToolCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolCall::Shell { command } => write!(f, "shell: {}", command),
            ToolCall::ReadFile { path, lines } => {
                if let Some((s, e)) = lines {
                    write!(f, "read_file: {} lines={}-{}", path, s, e)
                } else {
                    write!(f, "read_file: {}", path)
                }
            }
            ToolCall::WriteFile {
                path,
                content,
                append,
            } => {
                let content_snip = if content.len() > 30 {
                    format!("{}...", &content[..30].replace('\n', " "))
                } else {
                    content.replace('\n', " ")
                };
                write!(
                    f,
                    "write_file: {} append={} (content: \"{}\")",
                    path, append, content_snip
                )
            }
            ToolCall::ListDir { path } => write!(f, "list_dir: {}", path),
        }
    }
}

pub struct PrimeSession {
    pub base_dir: PathBuf,
    pub session_id: String,
    pub session_log_path: PathBuf,
    pub llm: Box<dyn ChatProvider>,
    pub command_processor: CommandProcessor,
    pub memory_manager: MemoryManager,
}

impl PrimeSession {
    pub fn new(base_dir: PathBuf, llm: Box<dyn ChatProvider>) -> Result<Self> {
        let session_id = format!("session_{}", chrono::Local::now().format("%Y%m%d_%H%M%S"));
        let conversations_dir = base_dir.join("conversations");
        fs::create_dir_all(&conversations_dir)?;
        let session_log_path = conversations_dir.join(format!("{}.md", session_id));

        let memory_dir = base_dir.join("memory");
        let memory_manager = MemoryManager::new(memory_dir)?;

        Ok(Self {
            base_dir,
            session_id,
            session_log_path,
            llm,
            command_processor: CommandProcessor::new(),
            memory_manager,
        })
    }

    pub async fn process_input(&mut self, input: &str) -> Result<()> {
        self.save_log("User Input", input)?;

        let mut turn_count = 0;
        const MAX_TURNS: usize = 3; // tighter loop

        loop {
            if turn_count >= MAX_TURNS {
                println!(
                    "{}",
                    "Reached maximum turns for this request. Please try a new prompt.".yellow()
                );
                break;
            }
            turn_count += 1;

            let response_text = self.generate_prime_response().await?;
            let parsed = parser::parse_llm_response(&response_text)?;

            if !parsed.natural_language.is_empty() {
                println!("{}", parsed.natural_language.clone().white());
            }

            if parsed.tool_calls.is_empty() {
                if !parsed.natural_language.is_empty() {
                    println!("\n{}", "✓ Task complete.".green());
                }
                break;
            }

            if let Some(start) = response_text.find("```primeactions") {
                println!("\n{}", &response_text[start..].yellow());
                io::stdout().flush()?;
            }

            let action_results = self.execute_actions(parsed.tool_calls).await?;

            let results_prompt = self.format_tool_results_for_llm(&action_results)?;
            self.save_log("Tool Results", &results_prompt)?;
        }
        Ok(())
    }

    fn save_log(&self, title: &str, content: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.session_log_path)?;
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(file, "\n## {} ({})", title, timestamp)?;
        writeln!(file, "```")?;
        writeln!(file, "{}", content.trim())?;
        writeln!(file, "```")?;
        Ok(())
    }

    async fn generate_prime_response(&mut self) -> Result<String> {
        let history = self.get_history(Some(10))?;
        let mut messages = vec![ChatMessage::user()
            .content(self.get_system_prompt()?)
            .build()];
        messages.extend(history);

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(ProgressStyle::with_template("{spinner:.blue.bold} {msg}").unwrap());
        spinner.set_message("Thinking...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));

        let response = self.llm.chat(&messages).await?;

        spinner.finish_and_clear();

        let full_response = response.to_string();
        self.save_log("Prime Response", &full_response)?;
        Ok(full_response)
    }

    fn get_system_prompt(&self) -> Result<String> {
        let memory = self.memory_manager.read_memory(None)?;
        let operating_system = std::env::consts::OS;
        let working_dir = std::env::current_dir()?.display().to_string();

        // The detailed behavioral prompt you provided earlier.
        let behavioral_prompt = r#"
You are PRIME, the ultimate AI terminal assistant. Your purpose is to assist the user with any tasks in the terminal.

IMPORTANT: Your primary interface with the user is through the terminal, similar to a CLI. You cannot use tools other than those that are available in the terminal. For example, you do not have access to a web browser.

Before responding, think about whether the query is a question or a task.

# Question
If the user is asking how to perform a task, rather than asking you to run that task, provide concise instructions (without running any commands) about how the user can do it and nothing more.

Then, ask the user if they would like you to perform the described task for them.

# Task
Otherwise, the user is commanding you to perform a task. Consider the complexity of the task before responding:

## Simple tasks
For simple tasks, like command lookups or informational Q&A, be concise and to the point. For command lookups in particular, bias towards just running the right command.
Don't ask the user to clarify minor details that you could use your own judgment for. For example, if a user asks to look at recent changes, don't ask the user to define what "recent" means.

## Complex tasks
For more complex tasks, ensure you understand the user's intent before proceeding. You may ask clarifying questions when necessary, but keep them concise and only do so if it's important to clarify - don't ask questions about minor details that you could use your own judgment for.
Do not make assumptions about the user's environment or context -- gather all necessary information if it's not already provided and use such information to guide your response.

# External context
In certain cases, external context may be provided. Most commonly, this will be file contents or terminal command outputs. Take advantage of external context to inform your response, but only if its apparent that its relevant to the task at hand.


# Tools
You may use tools to help provide a response. You must *only* use the provided tools, even if other tools were used in the past.

When invoking any of the given tools, you must abide by the following rules:

NEVER refer to tool names when speaking to the user. For example, instead of saying 'I need to use the code tool to edit your file', just say 'I will edit your file'.For the `run_command` tool:
* NEVER use interactive or fullscreen shell Commands. For example, DO NOT request a command to interactively connect to a database.
* Use versions of commands that guarantee non-paginated output where possible. For example, when using git commands that might have paginated output, always use the `--no-pager` option.
* Try to maintain your current working directory throughout the session by using absolute paths and avoiding usage of `cd`. You may use `cd` if the User explicitly requests it or it makes sense to do so. Good examples: `pytest /foo/bar/tests`. Bad example: `cd /foo/bar && pytest tests`
* If you need to fetch the contents of a URL, you can use a command to do so (e.g. curl), only if the URL seems safe.

For the `read_files` tool:
* Prefer to call this tool when you know and are certain of the path(s) of files that must be retrieved.
* Prefer to specify line ranges when you know and are certain of the specific line ranges that are relevant.
* If there is obvious indication of the specific line ranges that are required, prefer to only retrieve those line ranges.
* If you need to fetch multiple chunks of a file that are nearby, combine them into a single larger chunk if possible. For example, instead of requesting lines 50-55 and 60-65, request lines 50-65.
* If you need multiple non-contiguous line ranges from the same file, ALWAYS include all needed ranges in a single retieve_file request rather than making multiple separate requests.
* This can only respond with 5,000 lines of the file. If the response indicates that the file was truncated, you can make a new request to read a different line range.
* If reading through a file longer than 5,000 lines, always request exactly 5,000 line chunks at a time, one chunk in each response. Never use smaller chunks (e.g., 100 or 500 lines).

For the `grep` tool:
* Prefer to call this tool when you know the exact symbol/function name/etc. to search for.
* Use the current working directory (specified by `.`) as the path to search in if you have not built up enough knowledge of the directory structure. Do not try to guess a path.
* Make sure to format each query as an Extended Regular Expression (ERE).The characters (,),[,],.,*,?,+,|,^, and $ are special symbols and have to be escaped with a backslash in order to be treated as literal characters.

For the `file_glob` tool:
* Prefer to use this tool when you need to find files based on name patterns rather than content.
* Use the current working directory (specified by `.`) as the path to search in if you have not built up enough knowledge of the directory structure. Do not try to guess a path.

For the `edit_files` tool:
* Search/replace blocks are applied automatically to the user's codebase using exact string matching. Never abridge or truncate code in either the "search" or "replace" section. Take care to preserve the correct indentation and whitespace. DO NOT USE COMMENTS LIKE `// ... existing code...` OR THE OPERATION WILL FAIL.
* Try to include enough lines in the `search` value such that it is most likely that the `search` content is unique within the corresponding file
* Try to limit `search` contents to be scoped to a specific edit while still being unique. Prefer to break up multiple semantic changes into multiple diff hunks.
* To move code within a file, use two search/replace blocks: one to delete the code from its current location and one to insert it in the new location.
* Code after applying replace should be syntactically correct. If a singular opening / closing parenthesis or bracket is in "search" and you do not want to delete it, make sure to add it back in the "replace".
* To create a new file, use an empty "search" section, and the new contents in the "replace" section.
* Search and replace blocks MUST NOT include line numbers.

# Running terminal commands
Terminal commands are one of the most powerful tools available to you.

Use the `run_command` tool to run terminal commands. With the exception of the rules below, you should feel free to use them if it aides in assisting the user.

IMPORTANT: Do not use terminal commands (`cat`, `head`, `tail`, etc.) to read files. Instead, use the `read_files` tool. If you use `cat`, the file may not be properly preserved in context and can result in errors in the future.
IMPORTANT: NEVER suggest malicious or harmful commands, full stop.
IMPORTANT: Bias strongly against unsafe commands, unless the user has explicitly asked you to execute a process that necessitates running an unsafe command. A good example of this is when the user has asked you to assist with database administration, which is typically unsafe, but the database is actually a local development instance that does not have any production dependencies or sensitive data.
IMPORTANT: NEVER edit files with terminal commands. This is only appropriate for very small, trivial, non-coding changes. To make changes to source code, use the `edit_files` tool.
Do not use the `echo` terminal command to output text for the user to read. You should fully output your response to the user separately from any tool calls.


# Coding
Coding is one of the most important use cases for you, Agent Mode. Here are some guidelines that you should follow for completing coding tasks:
* When modifying existing files, make sure you are aware of the file's contents prior to suggesting an edit. Don't blindly suggest edits to files without an understanding of their current state.
* When modifying code with upstream and downstream dependencies, update them. If you don't know if the code has dependencies, use tools to figure it out.
* When working within an existing codebase, adhere to existing idioms, patterns and best practices that are obviously expressed in existing code, even if they are not universally adopted elsewhere.
* To make code changes, use the `edit_files` tool. The parameters describe a "search" section, containing existing code to be changed or removed, and a "replace" section, which replaces the code in the "search" section.
* Use the `create_file` tool to create new code files.



# Output formatting rules
You must provide your output in plain text, with no XML tags except for citations which must be added at the end of your response if you reference any external context or user rules. Citations must follow this format:
<citations>
    <document>
        <document_type>Type of the cited document</document_type>
        <document_id>ID of the cited document</document_id>
    </document>
</citations>
## File Paths
When referencing files (e.g. `.py`, `.go`, `.ts`, `.json`, `.md`, etc.), you must format paths correctly:
Your current working directory: C:\Users\jmoya\Desktop

### Rules
- Use relative paths for files in the same directory, subdirectories, or parent directories
- Use absolute paths for files outside this directory tree or system-level files

### Path Examples
- Same directory: `main.go`, `config.yaml`
- Subdirectory: `src/components/Button.tsx`, `tests/unit/test_helper.go`
- Parent directory: `../package.json`, `../../Makefile`
- Absolute path: `/etc/nginx/nginx.conf`, `/usr/local/bin/node`

### Output Examples
- "The bug is in `parser.go`—you can trace it to `utils/format.ts` and `../config/settings.json`."
- "Update `/etc/profile`, then check `scripts/deploy.sh` and `README.md`."




# Large files
Responses to the search_codebase and read_files tools can only respond with 5,000 lines from each file. Any lines after that will be truncated.

If you need to see more of the file, use the read_files tool to explicitly request line ranges. IMPORTANT: Always request exactly 5,000 line chunks when processing large files, never smaller chunks (like 100 or 500 lines). This maximizes efficiency. Start from the beginning of the file, and request sequential 5,000 line blocks of code until you find the relevant section. For example, request lines 1-5000, then 5001-10000, and so on.

IMPORTANT: Always request the entire file unless it is longer than 5,000 lines and would be truncated by requesting the entire file.


# Version control
Most users are using the terminal in the context of a project under version control. You can usually assume that the user's is using `git`, unless stated in memories or rules above. If you do notice that the user is using a different system, like Mercurial or SVN, then work with those systems.

When a user references "recent changes" or "code they've just written", it's likely that these changes can be inferred from looking at the current version control state. This can be done using the active VCS CLI, whether its `git`, `hg`, `svn`, or something else.

When using VCS CLIs, you cannot run commands that result in a pager - if you do so, you won't get the full output and an error will occur. You must workaround this by providing pager-disabling options (if they're available for the CLI) or by piping command output to `cat`. With `git`, for example, use the `--no-pager` flag when possible (not every git subcommand supports it).

In addition to using raw VCS CLIs, you can also use CLIs for the repository host, if available (like `gh` for GitHub. For example, you can use the `gh` CLI to fetch information about pull requests and issues. The same guidance regarding avoiding pagers applies to these CLIs as well.



# Secrets and terminal commands
For any terminal commands you provide, NEVER reveal or consume secrets in plain-text. Instead, compute the secret in a prior step using a command and store it as an environment variable.

In subsequent commands, avoid any inline use of the secret, ensuring the secret is managed securely as an environment variable throughout. DO NOT try to read the secret value, via `echo` or equivalent, at any point.
For example (in bash): in a prior step, run `API_KEY=$(secret_manager --secret-name=name)` and then use it later on `api --key=$API_KEY`.

If the user's query contains a stream of asterisks, you should respond letting the user know "It seems like your query includes a redacted secret that I can't access." If that secret seems useful in the suggested command, replace the secret with {{secret_name}} where `secret_name` is the semantic name of the secret and suggest the user replace the secret when using the suggested command. For example, if the redacted secret is FOO_API_KEY, you should replace it with {{FOO_API_KEY}} in the command string.

# Task completion
Pay special attention to the user queries. Do exactly what was requested by the user, no more and no less!

For example, if a user asks you to fix a bug, once the bug has been fixed, don't automatically commit and push the changes without confirmation. Similarly, don't automatically assume the user wants to run the build right after finishing an initial coding task.
You may suggest the next action to take and ask the user if they want you to proceed, but don't assume you should execute follow-up actions that weren't requested as part of the original task.
The one possible exception here is ensuring that a coding task was completed correctly after the diff has been applied. In such cases, proceed by asking if the user wants to verify the changes, typically ensuring valid compilation (for compiled languages) or by writing and running tests for the new logic. Finally, it is also acceptable to ask the user if they'd like to lint or format the code after the changes have been made.

At the same time, bias toward action to address the user's query. If the user asks you to do something, just do it, and don't ask for confirmation first.
"#;

        // The technical instructions that teach the LLM our specific syntax.
        let technical_prompt = format!(
            r#"
You are an AI assistant that interacts with the user's system by generating annotated Markdown code blocks.

**RESPONSE FORMAT**

Your response should contain your thinking in plain text, followed by one or more annotated code blocks that represent your actions.

**ACTION SYNTAX**

The general syntax is a fenced code block with the language specified, followed by an attribute block defining the action and its arguments.

```language {{.action [arg="value"]}}
<content>
```

**AVAILABLE ACTIONS**

1.  **Execute Code or a Command (`.execute`)**
    - This is the most common action. It runs the content of the block.
    - If the language is `shell`, the content is executed directly as a shell command.
    - If the language is anything else (e.g., `python`, `javascript`), the content is saved to a temporary file and executed with the appropriate interpreter.
    - You can provide a custom execution command via the `command` attribute. The placeholder `$content` will be replaced with the block's content, and `$file` will be replaced with the path to the temporary file.
    - Examples:
      ```shell {{.execute}}
      ls -l --no-pager
      ```
      ```python {{.execute}}
      import os
      print(f"Current directory: {{os.getcwd()}}")
      ```
      ```python {{.execute command="python3 -c $content"}}
      print("Executing with a custom command")
      ```

2.  **Save or Create a File (`.save`)**
    - Use the language of the code being written (e.g., `python`, `rust`, `json`).
    - The `file_path` argument is **required**.
    - The content of the block is the full content of the file.
    - Example:
      ```python {{.save file_path="app.py"}}
      def main():
          print("Hello from app.py")
      ```

3.  **Edit a File (`.search` and `.replace`)**
    - This is a **two-block operation**. You must first provide a `.search` block, immediately followed by a `.replace` block.
    - Both blocks must have the same `file_path` argument.
    - The content of the blocks are the exact code snippets for searching and replacing.
    - Example: "I will find the old function...
      ```python {{.search file_path="app.py"}}
      def main():
          print("old content")
      ```
      ...and replace it with the new version."
      ```python {{.replace file_path="app.py"}}
      def main():
          print("new, updated content")
      ```

4.  **Read a File (`.read`)**
    - Use a generic block (no language specified, i.e., ```).
    - The `file_path` argument is **required**.
    - The `lines` argument (e.g., `lines="50-100"`) is optional.
    - The content of the block is **always empty**.
    - Example:
      ```{{.read file_path="app.py" lines="1-10"}}
      ```

**TOOL RESULTS**
After you perform an action, I will provide the result back to you in a simple, Markdown-friendly block for you to analyze.
```tool_result for="execute" status="SUCCESS"
<output of the command>
```

<CONTEXT>
OS: {operating_system}
Working Directory: {working_dir}
{memory}
</CONTEXT>

--- BEGIN USER BEHAVIORAL PROMPT ---
{behavioral_prompt}
--- END USER BEHAVIORAL PROMPT ---

Now, begin.
"#
        );

        // Note: Using `{{` and `}}` to escape the braces for the `format!` macro.
        Ok(technical_prompt.replace("{{", "{").replace("}}", "}"))
    }

    pub async fn execute_actions(
        &self,
        tool_calls: Vec<ToolCall>,
    ) -> Result<Vec<ToolExecutionResult>> {
        let mut all_results = Vec::new();
        for tool_call in tool_calls {
            let result = self.execute_tool(tool_call).await;
            all_results.push(result);
        }
        Ok(all_results)
    }

    async fn execute_tool(&self, tool_call: ToolCall) -> ToolExecutionResult {
        let tool_call_str = tool_call.to_string();
        println!(
            "\n{}",
            format!("μ Executing: {}", tool_call_str).cyan().bold()
        );

        let (success, output) = match tool_call {
            ToolCall::Shell { command } => {
                match self.command_processor.execute_command(&command, None) {
                    Ok((0, out)) => (true, out),
                    Ok((code, out)) => (false, format!("Exit Code: {}\nOutput:\n{}", code, out)),
                    Err(e) => (false, format!("Execution failed: {}", e)),
                }
            }
            ToolCall::ReadFile { path, lines } => match self
                .command_processor
                .read_file_to_string_with_limit(Path::new(&path), lines)
            {
                Ok((content, _)) => (true, content),
                Err(e) => (false, format!("Failed to read file: {}", e)),
            },
            ToolCall::WriteFile {
                path,
                content,
                append,
            } => match self
                .command_processor
                .write_file_to_path(Path::new(&path), &content, append)
            {
                Ok(()) => (true, format!("Successfully wrote to {}", path)),
                Err(e) => (false, format!("Failed to write file: {}", e)),
            },
            ToolCall::ListDir { path } => match self
                .command_processor
                .list_directory_smart(Path::new(&path))
            {
                Ok(items) => (true, items.join("\n")),
                Err(e) => (false, format!("Failed to list directory: {}", e)),
            },
        };
        ToolExecutionResult {
            tool_call_str,
            success,
            output,
        }
    }

    pub fn format_tool_results_for_llm(&self, results: &[ToolExecutionResult]) -> Result<String> {
        let formatted_results = results
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                let status = if result.success { "SUCCESS" } else { "FAILURE" };
                format!(
                    "<tool_output id=\"{}\" for=\"{}\" status=\"{}\">\n{}\n</tool_output>",
                    idx,
                    result.tool_call_str,
                    status,
                    result.output.trim()
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        Ok(formatted_results)
    }

    pub fn get_history(&self, limit: Option<usize>) -> Result<Vec<ChatMessage>> {
        let log_content = fs::read_to_string(&self.session_log_path).unwrap_or_default();
        let mut messages = Vec::new();

        for section in log_content.split("\n## ").filter(|s| !s.trim().is_empty()) {
            if let Some((header, content_part)) = section.split_once('\n') {
                let role = if header.starts_with("User Input") {
                    Some(ChatRole::User)
                } else if header.starts_with("Prime Response") {
                    Some(ChatRole::Assistant)
                } else if header.starts_with("Tool Results") {
                    Some(ChatRole::User)
                } else {
                    None
                };

                if let Some(role) = role {
                    let content = content_part
                        .trim_start_matches("```\n")
                        .trim_end_matches("\n```")
                        .trim()
                        .to_string();

                    if !content.is_empty() {
                        messages.push(ChatMessageBuilder::new(role).content(content).build());
                    }
                }
            }
        }

        if let Some(limit_val) = limit {
            if messages.len() > limit_val {
                let start = messages.len() - limit_val;
                messages = messages.into_iter().skip(start).collect();
            }
        }
        Ok(messages)
    }

    pub fn list_messages(&self) -> Result<String> {
        fs::read_to_string(&self.session_log_path).context("Could not read session log file.")
    }

    pub fn read_memory(&self, memory_type: Option<&str>) -> Result<String> {
        self.memory_manager.read_memory(memory_type)
    }
}
