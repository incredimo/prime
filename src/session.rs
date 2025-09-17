 
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context as AnyhowContext, Result};
use crossterm::style::Stylize;
use indicatif::{ProgressBar, ProgressStyle};
use llm::chat::{ChatMessage, ChatMessageBuilder, ChatProvider, ChatRole};

use crate::commands::CommandProcessor;
use crate::memory::MemoryManager;
use crate::parser::{self, ToolCall};

const SPINNER_TICKS: &[&str] = &["⇣..", ".⇣.", "..⇣", ".⇣."];

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
            ToolCall::WriteMemory { memory_type, content } => {
                let content_snip = if content.len() > 30 {
                    format!("{}...", &content[..30].replace('\n', " "))
                } else {
                    content.replace('\n', " ")
                };
                write!(
                    f,
                    "write_memory: {} (content: \"{}\")",
                    memory_type, content_snip
                )
            }
            ToolCall::ClearMemory { memory_type } => write!(f, "clear_memory: {}", memory_type),
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
                    println!("\n{}", "Task complete.".green());
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
        spinner.set_style(
            ProgressStyle::with_template("{spinner:.yellow.bold} {msg}")
                .unwrap()
                .tick_strings(&SPINNER_TICKS),
        );
        spinner.set_message("Generating response...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(120));

        let response = self.llm.chat(&messages).await.map_err(|e| {
            spinner.finish_and_clear();
            e
        })?;

        spinner.finish_and_clear();

        let full_response = response.to_string();
        println!("{}", "  Response generated".dark_grey());
        self.save_log("Prime Response", &full_response)?;
        Ok(full_response)
    }

    fn get_system_prompt(&self) -> Result<String> {
        let memory = self.memory_manager.read_memory(None)?;
        let operating_system = std::env::consts::OS;
        let working_dir = std::env::current_dir()?.display().to_string();

        let behavioral_prompt = r#"
You are PRIME, an AI terminal assistant designed to help users accomplish tasks efficiently.

CORE PRINCIPLES:
1. You operate through the terminal interface only
2. You can execute commands, read/write files, and navigate directories
3. You distinguish between questions (requiring explanations) and tasks (requiring execution)
4. For questions, provide concise instructions without executing commands
5. For tasks, execute the necessary actions with minimal user interaction
6. For complex tasks, gather required information before proceeding

COMMAND EXECUTION:
- Use non-interactive commands with non-paginated output
- Maintain current working directory when possible
- Use absolute paths for clarity
- Handle errors gracefully and provide informative feedback

FILE OPERATIONS:
- Read files only when necessary for task completion
- Specify line ranges when reading large files
- Write files with appropriate content and permissions

MEMORY OPERATIONS:
- Use write_memory to save important information for future reference
- Specify memory type as either "long_term" or "short_term"
- Save important context, decisions, and outcomes to long-term memory
- Use short-term memory for temporary information relevant to current tasks
- Use clear_memory to reset either long-term or short-term memory when needed

RESPONSE FORMAT:
- Provide natural language responses for context and explanations
- Use annotated Markdown code blocks for actions
- Follow the specific syntax for each action type

TOOLS:
- Only use the provided tools (shell, read_file, write_file, list_dir, write_memory, clear_memory)
- Never reference tool names directly in user communications
- Always follow tool-specific rules and constraints

MEMORY:
- Utilize provided memory context to inform responses
- Reference previous interactions when relevant to the current task
- You have access to both long-term and short-term memory
- Long-term memory persists across sessions and should contain important information
- Short-term memory is cleared more frequently and contains recent interactions
- When appropriate, save important information to long-term memory for future reference

TASK COMPLETION:
- Focus on exactly what the user requested
- Verify task completion before responding
- Offer next steps only when appropriate
"#;

        // BUG FIX: The previous technical prompt described a completely different syntax
        // (`{{.execute}}`) than what the parser actually expects (`primeactions`).
        // This new prompt correctly instructs the LLM on how to use the available tools.
        let technical_prompt = format!(
            r#"
You are an AI assistant. Your goal is to help the user by executing commands on their system.

**RESPONSE FORMAT**

Your response must contain your plan and reasoning in plain text. If you need to perform actions, you must follow it with a single `primeactions` fenced code block.

**ACTION SYNTAX**

```primeactions
tool_name: arguments
another_tool: some other arguments
```

**AVAILABLE TOOLS**

1.  `shell: <command>`
    - Executes a shell command.
    - Example: `shell: ls -l`

2.  `read_file: <path> [lines=start-end]`
    - Reads a file. Optionally, you can specify a line range.
    - Example: `read_file: src/main.rs lines=1-20`
    - Example: `read_file: Cargo.toml`

3.  `write_file: <path> [append=true]`
    - Writes content to a file. Overwrites by default. Use `append=true` to append.
    - The content to write must follow on new lines, terminated by `EOF_PRIME`.
    - Example:
      ```primeactions
      write_file: new_file.txt
      Hello, world!
      This is a new file.
      EOF_PRIME
      ```

4.  `list_dir: <path>`
    - Lists the contents of a directory.
    - Example: `list_dir: .`

5.  `write_memory: <long_term|short_term>`
    - Writes content to your memory. Use `long_term` for persistent info, `short_term` for temporary context.
    - Content follows on new lines, terminated by `EOF_PRIME`.
    - Example:
      ```primeactions
      write_memory: short_term
      The user wants to refactor the `console.rs` file.
      EOF_PRIME
      ```

6.  `clear_memory: <long_term|short_term>`
    - Clears one of your memories.
    - Example: `clear_memory: short_term`


**TOOL RESULTS**

After you provide a `primeactions` block, I will execute the tools and return the output to you in the next turn, inside `<tool_output>` tags. You can then analyze the results and continue.

<CONTEXT>
OS: {operating_system}
Working Directory: {working_dir}
{memory}
</CONTEXT>

--- BEGIN BEHAVIORAL PROMPT ---
{behavioral_prompt}
--- END BEHAVIORAL PROMPT ---

Now, begin.
"#
        );

        Ok(technical_prompt)
    }

    pub async fn execute_actions(
        &self,
        tool_calls: Vec<ToolCall>,
    ) -> Result<Vec<ToolExecutionResult>> {
        let total_tools = tool_calls.len();
        println!("\n{}", format!("╭─ Running {} tool(s)", total_tools).dark_grey());

        let mut all_results = Vec::new();
        for (index, tool_call) in tool_calls.into_iter().enumerate() {
            let result = self.execute_tool(tool_call, index + 1, total_tools).await;
            all_results.push(result);
        }

        // Print summary
        let success_count = all_results.iter().filter(|r| r.success).count();
        let summary_msg = format!(
            "╰─ Executed {} tool(s): {} successful, {} failed.",
            total_tools,
            success_count,
            total_tools - success_count
        );

        if success_count == total_tools {
            println!("{}", summary_msg.green());
        } else {
            println!("{}", summary_msg.yellow());
        }

        Ok(all_results)
    }

    async fn execute_tool(
        &self,
        tool_call: ToolCall,
        index: usize,
        total_tools: usize,
    ) -> ToolExecutionResult {
        let tool_call_str = tool_call.to_string();
        let start_time = std::time::Instant::now();
        let tool_header = format!("[{}/{}] {}", index, total_tools, tool_call_str);

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::with_template("{spinner:.yellow.bold} {msg}")
                .unwrap()
                .tick_strings(&SPINNER_TICKS),
        );
        pb.set_message(tool_header.clone());
        pb.enable_steady_tick(std::time::Duration::from_millis(120));

        let (success, output) = match tool_call {
            ToolCall::Shell { command } => {
                match self.command_processor.execute_command(&command, None) {
                    Ok((0, out)) => (true, out),
                    Ok((code, out)) => {
                        if code == -1 {
                            (false, out)
                        } else {
                            (
                                false,
                                format!("Command failed with exit code {}\nOutput:\n{}", code, out),
                            )
                        }
                    }
                    Err(e) => (false, format!("Failed to execute command: {}", e)),
                }
            }
            ToolCall::ReadFile { path, lines } => {
                match self
                    .command_processor
                    .read_file_to_string_with_limit(Path::new(&path), lines)
                {
                    Ok((content, truncated)) => {
                        let result = if truncated {
                            format!("{}\nNote: File content was truncated", content)
                        } else {
                            content
                        };
                        (true, result)
                    }
                    Err(e) => (false, format!("Failed to read file '{}': {}", path, e)),
                }
            }
            ToolCall::WriteFile {
                path,
                content,
                append,
            } => {
                match self
                    .command_processor
                    .write_file_to_path(Path::new(&path), &content, append)
                {
                    Ok(()) => (true, format!("Successfully wrote to {}", path)),
                    Err(e) => (false, format!("Failed to write file '{}': {}", path, e)),
                }
            }
            ToolCall::ListDir { path } => {
                match self.command_processor.list_directory_smart(Path::new(&path)) {
                    Ok(items) => {
                        if items.is_empty() {
                            (true, "Directory is empty".to_string())
                        } else {
                            (true, items.join("\n"))
                        }
                    }
                    Err(e) => (false, format!("Failed to list directory '{}': {}", path, e)),
                }
            }
            ToolCall::WriteMemory {
                memory_type,
                content,
            } => match self.write_memory(&memory_type, &content) {
                Ok(()) => (true, format!("Successfully wrote to {} memory", memory_type)),
                Err(e) => (false, format!("Failed to write to {} memory: {}", memory_type, e)),
            },
            ToolCall::ClearMemory { memory_type } => match self.clear_memory(&memory_type) {
                Ok(()) => (true, format!("Successfully cleared {} memory", memory_type)),
                Err(e) => (false, format!("Failed to clear {} memory: {}", memory_type, e)),
            },
        };

        pb.finish_and_clear();
        let duration = start_time.elapsed();

        let status_str = if success {
            format!("Completed in {:?}", duration).green().to_string()
        } else {
            format!("Failed after {:?}", duration).red().to_string()
        };
        println!("│ {} ({})", tool_header.dim(), status_str);

        if !output.trim().is_empty() {
            for line in output.trim().lines() {
                let formatted_line = if success {
                    line.dim().to_string()
                } else {
                    line.red().to_string()
                };
                println!("│   {}", formatted_line);
            }
        }

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
    
    pub fn write_memory(&self, memory_type: &str, content: &str) -> Result<()> {
        self.memory_manager.write_memory(memory_type, content)
    }
    
    pub fn clear_memory(&self, memory_type: &str) -> Result<()> {
        self.memory_manager.clear_memory(memory_type)
    }
}
