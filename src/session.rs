// src/session.rs
// Session management and message handling for Prime
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::{Context as AnyhowContext, Result};
use chrono;
use console::Style;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use reqwest::Client;
use serde_json::json;

use crate::commands::CommandProcessor;
use crate::memory::MemoryManager;
use crate::styling::STYLER; // Assuming STYLER is accessible or passed

lazy_static! {
    // Updated regex to capture various data-actions and their paths/working_dirs
    static ref PANDOC_RE: Regex = Regex::new(
        r#"```\{.*?data-action="(execute|read_file|write_file|list_directory)"(?:[^\}]*?path="([^"]+)")?(?:[^\}]*?data-working-dir="([^"]+)")?[^\}]*\}\s*\n([\s\S]*?)```"#
    ).expect("Failed to compile Pandoc block regex");

    // Fallback for simple shell blocks (no specific action, implies execute)
    static ref FALLBACK_RE: Regex = Regex::new(
        r"```(?:shell|bash|sh|powershell|ps1)\s*\n([\s\S]*?)```"
    ).expect("Failed to compile fallback shell block regex");
}

/// Holds the result of a single command execution
#[derive(Debug, Clone)]
pub struct CommandExecutionResult {
    pub command: String,
    pub exit_code: i32,
    pub output: String,
    pub success: bool,
    pub working_dir: Option<String>, // Added working_dir
}

/// Holds the result of a single file operation
#[derive(Debug, Clone)]
pub struct FileOperationResult {
    pub action: String, // "read_file", "write_file", "list_directory"
    pub path: String,
    pub success: bool,
    pub output: String, // Content for read, list; status message for write
    pub truncated: Option<bool>, // For read_file
}

/// Enum to hold either a command execution result or a file operation result
#[derive(Debug, Clone)]
pub enum ProcessedItemResult {
    Command(CommandExecutionResult),
    FileOp(FileOperationResult),
}

/// Represents a session with the Prime assistant
pub struct PrimeSession {
    pub base_dir: PathBuf, // Workspace root or project base
    pub session_id: String,
    pub session_dir: PathBuf,
    pub message_counter: AtomicUsize,
    pub ollama_model: String,
    pub ollama_api_url: String,
    pub command_processor: CommandProcessor,
    pub memory_manager: MemoryManager,
    client: Client,
}

impl PrimeSession {
    pub fn new(base_dir: PathBuf, ollama_model: &str, ollama_api_base: &str) -> Result<Self> {
        let session_id = format!("session_{}", chrono::Local::now().format("%Y%m%d_%H%M%S"));
        let session_dir = base_dir.join("conversations").join(&session_id);
        fs::create_dir_all(&session_dir)?;

        let memory_dir = base_dir.join("memory");
        fs::create_dir_all(&memory_dir)?;
        for memory_file in ["long_term.md", "short_term.md"].iter() {
            let file_path = memory_dir.join(memory_file);
            if !file_path.exists() {
                let header = format!(
                    "# Prime {} Memory\n\n",
                    if *memory_file == "long_term.md" { "Long-term" } else { "Short-term" }
                );
                fs::write(&file_path, header)?;
            }
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(120)) // Increased timeout
            .gzip(true)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_dir, // Store the base_dir for path resolution
            session_id,
            session_dir,
            message_counter: AtomicUsize::new(0),
            ollama_model: ollama_model.to_string(),
            ollama_api_url: format!("{}/api/generate", ollama_api_base.trim_end_matches('/')),
            command_processor: CommandProcessor::new(),
            memory_manager: MemoryManager::new(memory_dir), // Corrected path
            client,
        })
    }

    fn next_message_number(&self) -> usize {
        self.message_counter.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn add_user_message(&self, content: &str) -> Result<PathBuf> {
        let message_number = self.next_message_number();
        let file_name = format!("{:03}_user.md", message_number);
        let file_path = self.session_dir.join(&file_name);
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let message_content = format!("# User Message\nTimestamp: {}\n\n{}", timestamp, content);
        fs::write(&file_path, message_content)?;
        Ok(file_path)
    }

    pub fn add_prime_message(&self, content: &str) -> Result<PathBuf> {
        let message_number = self.next_message_number();
        let file_name = format!("{:03}_prime.md", message_number);
        let file_path = self.session_dir.join(&file_name);
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let message_content = format!("# Prime Response\nTimestamp: {}\n\n{}", timestamp, content);
        fs::write(&file_path, message_content)?;
        Ok(file_path)
    }

    /// Add a system message for command output
    pub fn add_cmd_system_message(
        &self,
        command: &str,
        exit_code: i32,
        output: &str,
        working_dir: Option<&str>,
    ) -> Result<PathBuf> {
        let message_number = self.next_message_number();
        let file_name = format!("{:03}_system_cmd.md", message_number);
        let file_path = self.session_dir.join(&file_name);
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

        let wd_info = working_dir.map_or_else(String::new, |wd| format!("\nWorking Directory: {}", wd));
        let status_text = if exit_code == 0 { "Success" } else { "Error" };

        // Print command execution feedback to console
        if exit_code == 0 {
            println!(
                "{} {} {}",
                STYLER.success_style("✔"),
                STYLER.command_style(command),
                working_dir.map_or("".to_string(), |wd| STYLER.dim_gray_style(format!("(in {})", wd)).to_string())
            );
        } else {
            println!(
                "{} {} (exit code: {}) {}",
                STYLER.error_style("✖"),
                STYLER.command_style(command),
                STYLER.error_style(exit_code.to_string()),
                working_dir.map_or("".to_string(), |wd| STYLER.dim_gray_style(format!("(in {})", wd)).to_string())
            );
        }

        let message_content = format!(
            "# System Output (Command)\nTimestamp: {}\nStatus: {}{}\nCommand: {}\nExit Code: {}\n\n```\n{}\n```",
            timestamp, status_text, wd_info, command, exit_code, output
        );
        fs::write(&file_path, message_content)?;
        Ok(file_path)
    }

    /// Add a system message for file operation output
    pub fn add_file_op_system_message(
        &self,
        action: &str,
        path_str: &str,
        success: bool,
        details: &str, // Content for read/list, status for write
        truncated: Option<bool>,
    ) -> Result<PathBuf> {
        let message_number = self.next_message_number();
        let file_name = format!("{:03}_system_file_op.md", message_number);
        let file_path = self.session_dir.join(&file_name);
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let status_text = if success { "Success" } else { "Error" };

        let mut message_content = format!(
            "# System Output (File Operation)\nTimestamp: {}\nAction: {}\nPath: {}\nStatus: {}\n\n",
            timestamp, action, path_str, status_text
        );

        match action {
            "read_file" => {
                if let Some(true) = truncated {
                    message_content.push_str("(File content truncated)\n");
                }
                message_content.push_str(&format!("Content:\n```\n{}\n```", details));
            }
            "list_directory" => {
                message_content.push_str(&format!("Contents:\n```\n{}\n```", details));
            }
            "write_file" => {
                message_content.push_str(&format!("Details: {}", details));
            }
            _ => { // Should not happen with current actions
                message_content.push_str(&format!("Details:\n```\n{}\n```", details));
            }
        }
        fs::write(&file_path, message_content)?;
        Ok(file_path)
    }

    pub async fn generate_prime_response_stream(
        &self,
        current_turn_prompt: &str,
        is_error_correction_turn: bool,
    ) -> Result<String> {
        let mut ollama_prompt_payload = String::new();
        ollama_prompt_payload.push_str(&self.get_system_prompt()?);
        ollama_prompt_payload.push_str("\n\n");

        let history_limit = 10;
        let conversation_history = self.get_full_conversation_history_prompt(history_limit)?;
        if !conversation_history.is_empty() {
            ollama_prompt_payload.push_str("## Recent Conversation History:\n");
            ollama_prompt_payload.push_str(&conversation_history);
        }

        if is_error_correction_turn {
            ollama_prompt_payload.push_str("## Error Correction Task:\n");
        } else {
            ollama_prompt_payload.push_str("## Current User Request:\n");
        }
        ollama_prompt_payload.push_str(current_turn_prompt);
        ollama_prompt_payload.push_str("\n\n# Prime Response:\n");

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::with_template("{spinner:.blue.bold} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        spinner.enable_steady_tick(Duration::from_millis(80));
        spinner.set_message("Thinking...");

        let response = self
            .client
            .post(&self.ollama_api_url)
            .json(&json!({
                "model": self.ollama_model,
                "prompt": ollama_prompt_payload,
                "stream": true,
                "options": { "temperature": 0.5, "top_p": 0.9 }
            }))
            .send()
            .await
            .context("Failed to send request to Ollama API")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Ollama API error ({}): {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let mut full_response = String::new();
        let mut stream = response.bytes_stream();
        let mut first_token_received = false;

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.context("Stream error")?;
            for piece in std::str::from_utf8(&bytes)?.split('\n') {
                if piece.trim().is_empty() {
                    continue;
                }
                if let Ok(obj) = serde_json::from_str::<serde_json::Value>(piece) {
                    if let Some(tok) = obj.get("response").and_then(|v| v.as_str()) {
                        if !first_token_received {
                            spinner.finish_and_clear();
                            first_token_received = true;
                        }
                        print!("{}", STYLER.llm_response_style(tok)); // Apply styling
                        io::stdout().flush().unwrap();
                        full_response.push_str(tok);
                    }
                } else {
                    // Fallback for non-JSON pieces, though Ollama usually sends JSON lines
                    if !piece.is_empty() && !first_token_received {
                         spinner.finish_and_clear();
                         first_token_received = true;
                    }
                    print!("{}", STYLER.llm_response_style(piece));
                    io::stdout().flush().unwrap();
                    full_response.push_str(piece);
                }
            }
        }
        if !first_token_received { // If stream was empty or only newlines
            spinner.finish_and_clear();
        }
        println!(); // Newline after streaming

        self.add_prime_message(&full_response)?;
        println!("\n{}", STYLER.success_style("✓ Response complete"));
        Ok(full_response)
    }

    fn get_full_conversation_history_prompt(&self, limit: usize) -> Result<String> {
        let mut context_str = String::new();
        let messages = self.get_messages(Some(limit))?;
        for message in messages {
            context_str.push_str(&message.content);
            context_str.push_str("\n\n");
        }
        Ok(context_str)
    }

    fn get_system_prompt(&self) -> Result<String> {
        let memory = self.memory_manager.read_memory(None)?;
        // Note: The working directory for the LLM is implicitly self.base_dir unless a command specifies otherwise.
        // The LLM should be instructed that paths are relative to this base_dir if not absolute.
        let system_prompt = format!(
            r#"# Prime System Instructions

You are Prime, an advanced terminal assistant. Your primary workspace directory is '{}'.
All relative paths you provide in commands or file operations will be resolved against this directory unless a `data-working-dir` attribute is specified for an `execute` action.

## Capabilities
You can execute shell commands and perform file operations.

### Command Execution
- Use Pandoc-attributed markdown code blocks with `data-action="execute"`.
- Optionally, specify `data-working-dir="path/to/dir"` to run the command in a different directory relative to the workspace.
- Example:
  ```{{.shell data-action="execute" data-working-dir="src"}}
  ls -la
  ```
  ```{{.powershell data-action="execute"}}
  Get-ChildItem C:\Users
  ```

### File Operations
Paths for file operations are resolved relative to the workspace directory ('{}') if not absolute.

1.  **Read File (`read_file`)**:
    - Provide the path in a `path` attribute.
    - Example: ````{{.file data-action="read_file" path="src/main.rs"}}````
    - (The content of the code block itself is ignored for `read_file`)

2.  **Write File (`write_file`)**:
    - Provide the path in a `path` attribute.
    - The content to write should be within the code block.
    - Example:
      ````{{.file data-action="write_file" path="new_feature.rs"}}
      fn new_function() {{
          // Rust code
      }}
      ````

3.  **List Directory (`list_directory`)**:
    - Provide the path in a `path` attribute.
    - Example: ````{{.directory data-action="list_directory" path="src"}}````
    - (The content of the code block itself is ignored for `list_directory`)

## Communication Guidelines
- Respond clearly and concisely.
- After complex operations, summarize what was done.
- Wait for results before continuing multi-step processes.

## Memory Context
{}

## Environment
- OS: Windows 11 (PowerShell) or Linux/macOS (sh/bash) - adapt commands accordingly.
- Current Workspace: {}
"#,
            self.base_dir.display(), // For primary workspace directory
            self.base_dir.display(), // For file operations path resolution
            memory,
            self.base_dir.display()  // For Current Workspace
        );
        Ok(system_prompt)
    }

    /// Resolve a path provided by the LLM.
    /// If path_str is absolute, it's used directly.
    /// Otherwise, it's joined with self.base_dir.
    fn resolve_llm_path(&self, path_str: &str) -> PathBuf {
        let p = PathBuf::from(path_str);
        if p.is_absolute() {
            p
        } else {
            self.base_dir.join(p)
        }
    }


    pub fn process_commands(&self, response: &str) -> Result<Vec<ProcessedItemResult>> {
        let mut results = Vec::new();
        let mut found_actions = false;

        // Helper to extract attributes
        for cap in PANDOC_RE.captures_iter(response) {
            found_actions = true;
            let action = cap.get(1).map_or("", |m| m.as_str());
            let path_attr = cap.get(2).map(|m| m.as_str());
            let working_dir_attr = cap.get(3).map(|m| m.as_str());
            let content = cap.get(4).map_or("", |m| m.as_str());

            let cmd_style = Style::new().cyan(); // Re-declare or ensure STYLER is accessible

            match action {
                "execute" => {
                    let command_str = content;
                    if command_str.is_empty() { continue; }

                    let resolved_wd = working_dir_attr.map(|wd_str| self.resolve_llm_path(wd_str));
                    let wd_for_execute = resolved_wd.as_deref(); // Option<&Path>

                    // Confirmation for "ask me before" commands
                    if self.command_processor.is_ask_me_before_command(command_str) {
                        println!("{}", STYLER.warning_style(format!("Potential risk: The command '{}' may be destructive or require caution.", command_str)));
                        print!("Proceed? (y/N): ");
                        io::stdout().flush()?;
                        let mut user_confirmation = String::new();
                        io::stdin().read_line(&mut user_confirmation)?;
                        if !user_confirmation.trim().eq_ignore_ascii_case("y") {
                            println!("{}", STYLER.info_style("Command skipped by user."));
                            self.add_cmd_system_message(command_str, -1, "Skipped by user confirmation.", working_dir_attr)?;
                            results.push(ProcessedItemResult::Command(CommandExecutionResult {
                                command: command_str.to_string(),
                                exit_code: -1, // Special code for skipped
                                output: "Skipped by user confirmation.".to_string(),
                                success: false,
                                working_dir: working_dir_attr.map(String::from),
                            }));
                            continue;
                        }
                    }

                    match self.command_processor.execute_command(command_str, wd_for_execute) {
                        Ok((exit_code, output)) => {
                            self.add_cmd_system_message(command_str, exit_code, &output, working_dir_attr)?;
                            results.push(ProcessedItemResult::Command(CommandExecutionResult {
                                command: command_str.to_string(),
                                exit_code,
                                output,
                                success: exit_code == 0,
                                working_dir: working_dir_attr.map(String::from),
                            }));
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to execute command: {}", e);
                            self.add_cmd_system_message(command_str, -1, &error_msg, working_dir_attr)?;
                            results.push(ProcessedItemResult::Command(CommandExecutionResult {
                                command: command_str.to_string(),
                                exit_code: -1,
                                output: error_msg,
                                success: false,
                                working_dir: working_dir_attr.map(String::from),
                            }));
                        }
                    }
                }
                "read_file" | "list_directory" | "write_file" => {
                    if let Some(path_str) = path_attr {
                        let resolved_path = self.resolve_llm_path(path_str);
                        println!("{}", cmd_style.apply_to(format!("Processing {} for: {}", action, resolved_path.display())));

                        let (op_success, op_output, op_truncated) = match action {
                            "read_file" => {
                                match self.command_processor.read_file_to_string_with_limit(&resolved_path) {
                                    Ok((file_content, truncated)) => (true, file_content, Some(truncated)),
                                    Err(e) => (false, format!("Error reading file: {}", e), None),
                                }
                            }
                            "list_directory" => {
                                match self.command_processor.list_directory_smart(&resolved_path) {
                                    Ok(items) => (true, items.join("\n"), None),
                                    Err(e) => (false, format!("Error listing directory: {}", e), None),
                                }
                            }
                            "write_file" => {
                                match self.command_processor.write_file_to_path(&resolved_path, content) {
                                    Ok(()) => (true, format!("Successfully wrote to {}", resolved_path.display()), None),
                                    Err(e) => (false, format!("Error writing file: {}", e), None),
                                }
                            }
                            _ => unreachable!(),
                        };
                        self.add_file_op_system_message(action, path_str, op_success, &op_output, op_truncated)?;
                        results.push(ProcessedItemResult::FileOp(FileOperationResult {
                            action: action.to_string(),
                            path: path_str.to_string(),
                            success: op_success,
                            output: op_output,
                            truncated: op_truncated,
                        }));
                    } else {
                        let err_msg = format!("'path' attribute missing for {} action.", action);
                        self.add_file_op_system_message(action, "N/A", false, &err_msg, None)?;
                        results.push(ProcessedItemResult::FileOp(FileOperationResult {
                            action: action.to_string(),
                            path: "N/A".to_string(),
                            success: false,
                            output: err_msg,
                            truncated: None,
                        }));
                    }
                }
                _ => { // Should not happen if regex is correct
                    eprintln!("Unknown action parsed: {}", action);
                }
            }
        }

        // Fallback for simple shell blocks if no Pandoc actions were found
        if !found_actions {
            for cap in FALLBACK_RE.captures_iter(response) {
                if let Some(cmd_match) = cap.get(1) {
                    let command_str = cmd_match.as_str().trim();
                    if command_str.is_empty() { continue; }

                    // For fallback, working_dir is None (current workspace)
                    // And no "ask me before" check for simple blocks, treat as direct execution.
                    // Or, decide if fallback should also have safety. For now, direct.
                    match self.command_processor.execute_command(command_str, None) {
                         Ok((exit_code, output)) => {
                            self.add_cmd_system_message(command_str, exit_code, &output, None)?;
                            results.push(ProcessedItemResult::Command(CommandExecutionResult {
                                command: command_str.to_string(),
                                exit_code,
                                output,
                                success: exit_code == 0,
                                working_dir: None,
                            }));
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to execute fallback command: {}", e);
                            self.add_cmd_system_message(command_str, -1, &error_msg, None)?;
                            results.push(ProcessedItemResult::Command(CommandExecutionResult {
                                command: command_str.to_string(),
                                exit_code: -1,
                                output: error_msg,
                                success: false,
                                working_dir: None,
                            }));
                        }
                    }
                }
            }
        }
        Ok(results)
    }

    pub fn get_messages(&self, limit: Option<usize>) -> Result<Vec<Message>> {
        let entries = fs::read_dir(&self.session_dir)?;
        let mut messages = Vec::new();
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                    let parts: Vec<&str> = file_name.splitn(2, '_').collect();
                    if parts.len() < 2 { continue; }
                    let number = parts[0].parse::<usize>().unwrap_or(0);
                    let msg_type = parts[1].split('.').next().unwrap_or("unknown");
                    let content = fs::read_to_string(&path)?;
                    messages.push(Message {
                        number,
                        msg_type: msg_type.to_string(),
                        path: path.clone(),
                        content,
                    });
                }
            }
        }
        messages.sort_by_key(|m| m.number);
        if let Some(limit_val) = limit {
            if messages.len() > limit_val {
                messages = messages.into_iter().rev().take(limit_val).collect();
                messages.sort_by_key(|m| m.number); // Re-sort to ascending after rev().take()
            }
        }
        Ok(messages)
    }

    pub fn list_messages(&self) -> Result<Vec<String>> {
        let messages = self.get_messages(None)?;
        let mut result = Vec::new();
        for message in messages {
            let first_line = message
                .content
                .lines()
                .filter(|line| !line.is_empty())
                .next()
                .unwrap_or("[Empty message]");
            result.push(format!(
                "{:03} - {}: {}",
                message.number, message.msg_type, first_line
            ));
        }
        Ok(result)
    }

    pub fn read_message(&self, number: usize) -> Result<String> {
        let number_str = format!("{:03}", number);
        let entries = fs::read_dir(&self.session_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                    if file_name.starts_with(&number_str) {
                        return Ok(fs::read_to_string(&path).with_context(|| {
                            format!("Failed to read message file: {}", path.display())
                        })?);
                    }
                }
            }
        }
        Err(anyhow::anyhow!("Message {} not found", number))
    }

    pub fn read_memory(&self, memory_type: Option<&str>) -> Result<String> {
        self.memory_manager.read_memory(memory_type)
    }
}

/// Represents a message in a Prime session
pub struct Message {
    pub number: usize,
    pub msg_type: String,
    pub path: PathBuf,
    pub content: String,
}