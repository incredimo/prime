use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context as AnyhowContext, Result};
use crossterm::style::Stylize;
use indicatif::{ProgressBar, ProgressStyle};
use llm::chat::{ChatMessage, ChatMessageBuilder, ChatProvider, ChatRole};

use crate::commands::CommandProcessor;
use crate::memory::MemoryManager;
use crate::parser::{self, ToolCall};

const SPINNER_TICKS: &[&str] = &["⇣", " ", "⇣", " "];

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
            ToolCall::WriteFile { path, content, append, } => {
                let content_snip = if content.len() > 30 {
                    format!("{}...", &content[..30].replace('\n', " "))
                } else {
                    content.replace('\n', " ")
                };
                write!(f, "write_file: {} append={} (content: \"{}\")", path, append, content_snip)
            }
            ToolCall::ListDir { path } => write!(f, "list_dir: {}", path),
            ToolCall::ChangeDir { path } => write!(f, "cd: {}", path),
            ToolCall::WriteMemory { memory_type, content } => {
                let content_snip = if content.len() > 30 {
                    format!("{}...", &content[..30].replace('\n', " "))
                } else {
                    content.replace('\n', " ")
                };
                write!(f, "write_memory: {} (content: \"{}\")", memory_type, content_snip)
            }
            ToolCall::ClearMemory { memory_type } => write!(f, "clear_memory: {}", memory_type),
        }
    }
}

#[allow(dead_code)]
pub struct PrimeSession {
    pub base_dir: PathBuf,
    pub session_id: String,
    pub session_log_path: PathBuf,
    pub llm: Box<dyn ChatProvider>,
    pub command_processor: CommandProcessor,
    pub memory_manager: MemoryManager,
    pub working_dir: PathBuf,
}

impl PrimeSession {
    pub fn new(base_dir: PathBuf, llm: Box<dyn ChatProvider>) -> Result<Self> {
        let session_id = format!("session_{}", chrono::Local::now().format("%Y%m%d_%H%M%S"));
        let conversations_dir = base_dir.join("conversations");
        fs::create_dir_all(&conversations_dir)?;
        let session_log_path = conversations_dir.join(format!("{}.md", session_id));

        let memory_dir = base_dir.join("memory");
        let memory_manager = MemoryManager::new(memory_dir)?;

        let working_dir = std::env::current_dir().context("Failed to get current working directory")?;

        Ok(Self {
            base_dir,
            session_id,
            session_log_path,
            llm,
            command_processor: CommandProcessor::new(),
            memory_manager,
            working_dir,
        })
    }

    pub async fn process_input(&mut self, input: &str) -> Result<()> {
        self.save_log("User Input", input)?;

        const MAX_CONSECUTIVE_TOOL_TURNS: usize = 10;
        let mut tool_turn_count = 0;

        loop {
            if tool_turn_count >= MAX_CONSECUTIVE_TOOL_TURNS {
                println!("{}", "Reached maximum tool execution turns. The session might be in a loop. Please try a new prompt.".red());
                break;
            }

            let response_text = self.generate_prime_response().await?;
            let parsed = parser::parse_llm_response(&response_text)?;

            if !parsed.natural_language.is_empty() {
                println!("{}", parsed.natural_language.clone().white());
                io::stdout().flush()?;
            }

            if parsed.tool_calls.is_empty() {
                if !parsed.natural_language.is_empty() {
                    println!("\n{}", "Task complete.".green());
                }
                break;
            }

            tool_turn_count += 1;

            println!("\n{}", "┏━ PROPOSED ACTION PLAN ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bold().yellow());
            for (i, tool) in parsed.tool_calls.iter().enumerate() {
                println!("┃ {}. {}", i + 1, tool.to_string().yellow());
            }
            println!("{}", "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bold().yellow());

            // Check if any tool calls are destructive
            let is_destructive = parsed.tool_calls.iter().any(|tool_call| {
                match tool_call {
                    ToolCall::Shell { command } => {
                        self.command_processor.is_command_destructive(command)
                    }
                    _ => false, // Other tool calls are considered non-destructive
                }
            });

            let should_execute = if is_destructive {
                print!("This plan contains potentially destructive commands. Execute? (y/N): ");
                io::stdout().flush().context("Failed to flush stdout")?;

                let mut confirmation = String::new();
                io::stdin().read_line(&mut confirmation).context("Failed to read user input")?;
                confirmation.trim().eq_ignore_ascii_case("y")
            } else {
                // Auto-execute after 2-second countdown for non-destructive commands
                for i in (1..=20).rev() {
                    print!("\rAuto-executing in {:.1}s... (press Ctrl+C to cancel)", i as f32 / 10.0);
                    io::stdout().flush().context("Failed to flush stdout")?;
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                println!("\rExecuting plan...{}", " ".repeat(30));
                true
            };

            if !should_execute {
                println!("{}", "Plan cancelled by user.".red());
                self.save_log("System", "Plan cancelled by user.")?;
                break;
            }

            match self.execute_actions(parsed.tool_calls).await {
                Ok(successful_results) => {
                    let results_prompt = self.format_tool_results_for_llm(&successful_results)?;
                    self.save_log("Tool Results", &results_prompt)?;
                }
                Err(failed_result) => {
                    let error_prompt = self.format_tool_failure_for_llm(&failed_result)?;
                    println!("\n{}", "A tool failed. The AI will attempt to self-correct.".bold().red());
                    self.save_log("Tool Failure", &error_prompt)?;
                }
            }
        }
        Ok(())
    }

    fn save_log(&self, title: &str, content: &str) -> Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(&self.session_log_path)?;
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(file, "\n## {} ({})", title, timestamp)?;
        writeln!(file, "```")?;
        writeln!(file, "{}", content.trim())?;
        writeln!(file, "```")?;
        Ok(())
    }

    async fn generate_prime_response(&mut self) -> Result<String> {
        let history = self.get_history(Some(10))?;
        let mut messages = vec![ChatMessage::user().content(self.get_system_prompt()?).build()];
        messages.extend(history);

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(ProgressStyle::with_template("{spinner:.yellow.bold} {msg}").unwrap().tick_strings(&SPINNER_TICKS));
        spinner.set_message("Generating response...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(120));

        let response = self.llm.chat(&messages).await.map_err(|e| {
            spinner.finish_and_clear();
            e
        })?;

        spinner.finish_and_clear();

        let full_response = response.to_string();
        self.save_log("Prime Response", &full_response)?;
        Ok(full_response)
    }

    fn get_system_prompt(&self) -> Result<String> {
        let memory = self.memory_manager.read_memory(None)?;
        let operating_system = std::env::consts::OS;
        let working_dir = self.working_dir.display().to_string();

        let behavioral_prompt = r#"
You are PRIME, an AI terminal assistant designed to help users accomplish tasks efficiently.

CORE PRINCIPLES:
1. You operate through the terminal interface only.
2. Formulate a plan, present it, and await execution.
3. On failure, analyze the error and formulate a new, corrected plan.

ENVIRONMENTAL AWARENESS:
- Before performing complex operations like software installation, always perform pre-flight checks to gather context.
- Use commands like `python --version`, `uname -a` (or `ver` on Windows), `uv --version`, and `nvidia-smi` to understand the system. Incorporate this information into your plan.

COMMAND EXECUTION:
- Use non-interactive commands with non-paginated output.
- Use the `cd` command to change the working directory; this state is maintained across turns.
- Use absolute paths when possible for clarity, or paths relative to the current working directory.
- Handle errors gracefully by analyzing the output and providing a corrected plan.

RESPONSE FORMAT:
- Provide natural language responses for context and explanations.
- Use annotated Markdown code blocks for actions.

TOOLS:
- Only use the provided tools.
- Never reference tool names directly in user communications.
- Always follow tool-specific rules and constraints.

TASK COMPLETION:
- Focus on exactly what the user requested.
- If a tool fails, DO NOT RE-TRY THE EXACT SAME COMMAND. Analyze the error message and change your approach.
- Verify task completion before responding with a final message.
"#;

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
    - Executes a shell command in the current working directory.
    - Example: `shell: ls -l`

2.  `cd: <path>`
    - Changes the current working directory. The new directory persists for all future commands.
    - Example: `cd: src/`

3.  `read_file: <path> [lines=start-end]`
    - Reads a file. Optionally, you can specify a line range.
    - Example: `read_file: src/main.rs lines=1-20`

4.  `write_file: <path> [append=true]`
    - Writes content to a file. Overwrites by default. Use `append=true` to append.
    - The content to write must follow on new lines, terminated by `EOF_PRIME`.
    - Example:
      ```primeactions
      write_file: new_file.txt
      Hello, world!
      EOF_PRIME
      ```

5.  `list_dir: <path>`
    - Lists the contents of a directory.
    - Example: `list_dir: .`

6.  `write_memory: <long_term|short_term>`
    - Writes content to your memory for context.
    - Content follows on new lines, terminated by `EOF_PRIME`.
    - Example:
      ```primeactions
      write_memory: short_term
      The user wants to refactor the `console.rs` file.
      EOF_PRIME
      ```

7.  `clear_memory: <long_term|short_term>`
    - Clears one of your memories.
    - Example: `clear_memory: short_term`


**TOOL RESULTS**

After you provide a `primeactions` block, I will execute the tools and return the output to you. If a command fails, I will return only the error, and you must formulate a new plan to fix it.

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
        &mut self,
        tool_calls: Vec<ToolCall>,
    ) -> Result<Vec<ToolExecutionResult>, ToolExecutionResult> {
        let total_tools = tool_calls.len();
        println!("\n{}", format!("╭─ running {} tools", total_tools).dark_grey());

        let mut all_results = Vec::new();
        for (index, tool_call) in tool_calls.into_iter().enumerate() {
            let result = self.execute_tool(tool_call, index + 1, total_tools).await;
            if !result.success {
                let summary_msg = format!("╰─ execution halted. {} of {} tools failed.", 1, total_tools);
                println!("{}", summary_msg.red());
                return Err(result);
            }
            all_results.push(result);
        }

        let summary_msg = format!("╰─ executed {} tools successfully.", total_tools);
        println!("{}", summary_msg.green());

        Ok(all_results)
    }

    async fn execute_tool(
        &mut self,
        tool_call: ToolCall,
        index: usize,
        total_tools: usize,
    ) -> ToolExecutionResult {
        let tool_call_str = tool_call.to_string();
        let start_time = std::time::Instant::now();
        let tool_header = format!("[{}/{}] {}", index, total_tools, tool_call_str);

        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::with_template("{spinner:.yellow.bold} {msg}").unwrap().tick_strings(&SPINNER_TICKS));
        pb.set_message(tool_header.clone());
        pb.enable_steady_tick(std::time::Duration::from_millis(120));

        let (success, output) = match tool_call {
            ToolCall::ChangeDir { path } => {
                let new_path = self.working_dir.join(&path);
                if new_path.is_dir() {
                    match new_path.canonicalize() {
                        Ok(canonical_path) => {
                            self.working_dir = canonical_path;
                            (true, format!("Changed working directory to {}", self.working_dir.display()))
                        }
                        Err(e) => (false, format!("Failed to canonicalize path '{}': {}", new_path.display(), e)),
                    }
                } else {
                    (false, format!("Directory not found: {}", new_path.display()))
                }
            }
            ToolCall::Shell { command } => {
                match self.command_processor.execute_command(&command, Some(&self.working_dir)) {
                    Ok((0, out)) => (true, out),
                    Ok((code, out)) => {
                        if code == -1 { (false, out) } else { (false, format!("Command failed with exit code {}\nOutput:\n{}", code, out)) }
                    }
                    Err(e) => (false, format!("Failed to execute command: {}", e)),
                }
            }
            ToolCall::ReadFile { path, lines } => {
                let absolute_path = self.working_dir.join(&path);
                match self.command_processor.read_file_to_string_with_limit(&absolute_path, lines) {
                    Ok((content, truncated)) => {
                        let result = if truncated { format!("{}\nNote: File content was truncated", content) } else { content };
                        (true, result)
                    }
                    Err(e) => (false, format!("Failed to read file '{}': {}", absolute_path.display(), e)),
                }
            }
            ToolCall::WriteFile { path, content, append, } => {
                let absolute_path = self.working_dir.join(&path);
                match self.command_processor.write_file_to_path(&absolute_path, &content, append) {
                    Ok(()) => (true, format!("Successfully wrote to {}", absolute_path.display())),
                    Err(e) => (false, format!("Failed to write file '{}': {}", absolute_path.display(), e)),
                }
            }
            ToolCall::ListDir { path } => {
                let absolute_path = self.working_dir.join(&path);
                match self.command_processor.list_directory_smart(&absolute_path) {
                    Ok(items) => {
                        if items.is_empty() { (true, "Directory is empty".to_string()) } else { (true, items.join("\n")) }
                    }
                    Err(e) => (false, format!("Failed to list directory '{}': {}", absolute_path.display(), e)),
                }
            }
            ToolCall::WriteMemory { memory_type, content, } => match self.write_memory(&memory_type, &content) {
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

        let status_str = if success { format!("Completed in {:?}", duration).green().to_string() } else { format!("Failed after {:?}", duration).red().to_string() };
        println!("│ {} ({})", tool_header.dim(), status_str);

        if !output.trim().is_empty() {
            for line in output.trim().lines() {
                let formatted_line = if success { line.dim().to_string() } else { line.red().to_string() };
                println!("│   {}", formatted_line);
            }
        }

        ToolExecutionResult { tool_call_str, success, output }
    }

    pub fn format_tool_results_for_llm(&self, results: &[ToolExecutionResult]) -> Result<String> {
        let formatted_results = results.iter().enumerate().map(|(idx, result)| {
            let status = if result.success { "SUCCESS" } else { "FAILURE" };
            format!("<tool_output id=\"{}\" for=\"{}\" status=\"{}\">\n{}\n</tool_output>", idx, result.tool_call_str, status, result.output.trim())
        }).collect::<Vec<String>>().join("\n");
        Ok(formatted_results)
    }

    pub fn format_tool_failure_for_llm(&self, result: &ToolExecutionResult) -> Result<String> {
        let formatted_result = format!("<tool_output for=\"{}\" status=\"FAILURE\">\n{}\n</tool_output>", result.tool_call_str, result.output.trim());
        Ok(formatted_result)
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
                } else if header.starts_with("Tool Results") || header.starts_with("Tool Failure") || header.starts_with("System") {
                    Some(ChatRole::User)
                } else {
                    None
                };

                if let Some(role) = role {
                    let content = content_part.trim_start_matches("```\n").trim_end_matches("\n```").trim().to_string();
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