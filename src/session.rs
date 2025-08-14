 
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
        // Allow overriding from env; default to 20 recursive steps.
        let max_turns: usize = std::env::var("LLM_MAX_TURNS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(20);

        // Basic loop-safety: detect identical action blocks repeating.
        use std::collections::VecDeque;
        const DUP_WINDOW: usize = 3;
        let mut recent_action_signatures: VecDeque<String> = VecDeque::new();

        loop {
            if turn_count >= max_turns {
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

            // ------- loop-safety: detect repeating action signatures -------
            let sig = parsed
                .tool_calls
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            recent_action_signatures.push_back(sig.clone());
            if recent_action_signatures.len() > DUP_WINDOW {
                recent_action_signatures.pop_front();
            }
            if recent_action_signatures.len() == DUP_WINDOW
                && recent_action_signatures.iter().all(|s| *s == sig)
            {
                println!("{}", "⚠ Detected repeated identical actions. Stopping to avoid an infinite loop.".yellow());
                break;
            }
            // ----------------------------------------------------------------

            let action_results = self.execute_actions(parsed.tool_calls).await?;

            let results_prompt = self.format_tool_results_for_llm(&action_results)?;
            self.save_log("Tool Results", &results_prompt)?;

            // Tiny pacing so the spinner messages feel crisp
            tokio::time::sleep(std::time::Duration::from_millis(80)).await;
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
        // Give the model enough context to chain steps; configurable window.
        let history_window: usize = std::env::var("LLM_HISTORY_WINDOW")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(50);
        let history = self.get_history(Some(history_window))?;
        let mut messages = vec![ChatMessage::user()
            .content(self.get_system_prompt()?)
            .build()];
        messages.extend(history);

        // A small, consistent “continue or finish” instruction each turn.
        messages.push(
            ChatMessage::user()
                .content("Given the latest <tool_output> blocks, either:\n- produce a ```primeactions block with the next minimal set of tool calls, or\n- if the task is fully complete, reply with brief confirmation and NO primeactions block.")
                .build(),
        );

        let spinner = ProgressBar::new_spinner();
        spinner
            .set_style(ProgressStyle::with_template("{spinner:.blue.bold} {msg}").unwrap());
        spinner.set_message("Thinking...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));

        let response = self.llm.chat(&messages).await.map_err(|e| {
            spinner.finish_and_clear();
            e
        })?;

        spinner.finish_and_clear();

        // Add a small delay to show the completion message
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let full_response = response.to_string();
        println!("{}", "  ✓ Response generated".green());
        self.save_log("Prime Response", &full_response)?;
        Ok(full_response)
    }

    fn get_system_prompt(&self) -> Result<String> {
        let memory = self.memory_manager.read_memory(None)?;
        let operating_system = std::env::consts::OS;
        let working_dir = std::env::current_dir()?.display().to_string();

        // Parser-aligned prompt. Only the actions your parser understands.
        let prompt = format!(
r#"
You are PRIME, a terminal-only assistant that can **plan → act → observe → repeat** until the task is complete.

TOOLS (exact syntax):
````

```primeactions
shell: <command with args>
list_dir: <path>
read_file: <path> [optional: lines=START-END]
write_file: <path> [optional: append=true]
<file content for write_file, terminated by the literal line>
EOF_PRIME
```

```

MEMORY ACTIONS (same block, content terminated by EOF_PRIME):
```

```primeactions
write_memory: long_term
<content>
EOF_PRIME
```

```primeactions
clear_memory: short_term
```

````

RULES:
1) **Recurse until done**. After receiving <tool_output> blocks, decide the next minimal set of actions. If finished, **do not** emit a primeactions block.
2) Prefer deterministic, non-interactive shell invocations. Keep outputs concise (avoid pagers).
3) Read only what you need; use `lines=…` for large files.
4) For `write_file`, emit the full desired content and terminate with `EOF_PRIME`.
5) Use memory tools via `write_memory` and `clear_memory` actions.
6) Safety: avoid destructive commands unless the user clearly asked; if needed, proceed with minimal scope.

OBSERVATIONS COME AS:
<tool_output id="N" for="…" status="SUCCESS|FAILURE">
<stdout/preview …>
</tool_output>

OS: {operating_system}
PWD: {working_dir}
{memory}
"#
        );

        Ok(prompt)
    }

    pub async fn execute_actions(
        &self,
        tool_calls: Vec<ToolCall>,
    ) -> Result<Vec<ToolExecutionResult>> {
        let total_tools = tool_calls.len();
        println!(
            "\n{}",
            format!("Executing {} tool(s)...", total_tools).cyan().bold()
        );

        let mut all_results = Vec::new();
        for (index, tool_call) in tool_calls.into_iter().enumerate() {
            println!(
                "{}",
                format!("  [{}/{}] {}", index + 1, total_tools, tool_call.to_string()).dim()
            );
            let result = self.execute_tool(tool_call).await;
            all_results.push(result);
        }

        // Print summary
        let success_count = all_results.iter().filter(|r| r.success).count();
        if success_count == total_tools {
            println!(
                "\n{}",
                format!("✓ All {} tool(s) executed successfully", total_tools).green()
            );
        } else {
            println!(
                "\n{}",
                format!("⚠ {} of {} tool(s) executed successfully", success_count, total_tools).yellow()
            );
        }

        Ok(all_results)
    }

    async fn execute_tool(&self, tool_call: ToolCall) -> ToolExecutionResult {
        let tool_call_str = tool_call.to_string();
        let start_time = std::time::Instant::now();

        // Print initial execution message
        println!(
            "\n{}",
            format!("μ Executing: {}", tool_call_str).cyan().bold()
        );

        // Create a progress bar for the tool execution
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::with_template("{spinner:.green.bold} {msg}").unwrap());
        pb.set_message("Executing...");
        pb.enable_steady_tick(std::time::Duration::from_millis(80));

        let (success, output) = match tool_call {
            ToolCall::Shell { command } => {
                println!("{}", format!("  → Running command: {}", command).dim());
                match self.command_processor.execute_command(&command, None) {
                    Ok((0, out)) => (true, out),
                    Ok((code, out)) => {
                        if code == -1 {
                            // Command cancelled by user
                            (false, out)
                        } else {
                            // Command failed with exit code
                            (false, format!("Command failed with exit code {}\nOutput:\n{}", code, out))
                        }
                    },
                    Err(e) => (false, format!("Failed to execute command: {}", e)),
                }
            }
            ToolCall::ReadFile { path, lines } => {
                let lines_str = if let Some((s, e)) = lines {
                    format!("lines {}-{}", s, e)
                } else {
                    "full file".to_string()
                };
                println!("{}", format!("  → Reading file: {} ({})", path, lines_str).dim());
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
                    },
                    Err(e) => (false, format!("Failed to read file '{}': {}", path, e)),
                }
            }
            ToolCall::WriteFile {
                path,
                content,
                append,
            } => {
                let action = if append { "Appending to" } else { "Writing to" };
                println!("{}", format!("  → {} file: {}", action, path).dim());
                match self
                    .command_processor
                    .write_file_to_path(Path::new(&path), &content, append)
                {
                    Ok(()) => (true, format!("Successfully wrote to {}", path)),
                    Err(e) => (false, format!("Failed to write file '{}': {}", path, e)),
                }
            }
            ToolCall::ListDir { path } => {
                println!("{}", format!("  → Listing directory: {}", path).dim());
                match self
                    .command_processor
                    .list_directory_smart(Path::new(&path))
                {
                    Ok(items) => {
                        if items.is_empty() {
                            (true, "Directory is empty".to_string())
                        } else {
                            (true, items.join("\n"))
                        }
                    },
                    Err(e) => (false, format!("Failed to list directory '{}': {}", path, e)),
                }
            }
            ToolCall::WriteMemory { memory_type, content } => {
                println!("{}", format!("  → Writing to {} memory", memory_type).dim());
                match self
                    .write_memory(&memory_type, &content)
                {
                    Ok(()) => (true, format!("Successfully wrote to {} memory", memory_type)),
                    Err(e) => (false, format!("Failed to write to {} memory: {}", memory_type, e)),
                }
            }
            ToolCall::ClearMemory { memory_type } => {
                println!("{}", format!("  → Clearing {} memory", memory_type).dim());
                match self
                    .clear_memory(&memory_type)
                {
                    Ok(()) => (true, format!("Successfully cleared {} memory", memory_type)),
                    Err(e) => (false, format!("Failed to clear {} memory: {}", memory_type, e)),
                }
            }
        };

        // Finish the progress bar
        pb.finish_and_clear();

        // Print completion message with timing
        let duration = start_time.elapsed();
        if success {
            println!("{}", format!("  ✓ Completed in {:?}", duration).green());
        } else {
            println!("{}", format!("  ✗ Failed after {:?}", duration).red());
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
 