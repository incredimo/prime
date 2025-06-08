// session.rs
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
use log;
use regex::Regex;
use reqwest::{Client, header}; // Added reqwest::header
use serde_json::json;

use crate::commands::CommandProcessor;
use crate::memory::MemoryManager;

lazy_static! {
    // Fixed regex patterns to properly capture the command content
    static ref PANDOC_RE: Regex = Regex::new(
        r#"```\{\.(?:shell|bash|sh|powershell|ps1)(?:[^\}]*?)data-action="execute"[^\}]*\}\s*\n([\s\S]*?)```"#
    ).expect("Failed to compile Pandoc shell block regex");
    
    static ref FALLBACK_RE: Regex = Regex::new(
        r"```(?:shell|bash|sh|powershell|ps1)\s*\n([\s\S]*?)```"
    ).expect("Failed to compile fallback shell block regex");

    static ref FILE_OP_RE: Regex = Regex::new(
        r#"```\{\.file_op\s*(?:[^\}]*?)?data-action="(?P<action>[^"]+)"\s*(?:[^\}]*?)?data-path="(?P<path>[^"]+)"(?P<attributes>[^\}]*)\}\s*\n?(?P<content>[\s\S]*?)```"#
    ).expect("Failed to compile FILE_OP_RE regex");

    // Regex for parsing individual data attributes from the captured 'attributes' string
    static ref DATA_ATTR_RE: Regex = Regex::new(
        r#"\s*data-(?P<key>[a-zA-Z0-9_-]+)="(?P<value>[^"]+)""#
    ).expect("Failed to compile DATA_ATTR_RE regex");

    static ref WEB_OP_RE: Regex = Regex::new(
        r#"```\{\.web_op\s*(?:[^\}]*?)?data-action="(?P<action>[^"]+)"\s*(?:[^\}]*?)?data-url="(?P<url>[^"]+)"[^\}]*\}\s*```"#
    ).expect("Failed to compile WEB_OP_RE regex");
}

/// Represents the result of processing a single item (command, file operation, or web operation)
#[derive(Debug)]
pub enum ProcessedItemResult {
    Command(CommandExecutionResult),
    FileOp(FileOperationResult),
    WebOp(WebOperationResult),
}

/// Represents the result of a web operation
#[derive(Debug)]
pub struct WebOperationResult {
    pub action: String,
    pub url: String,
    pub success: bool,
    pub output: String,
}

/// Represents the result of a file operation
#[derive(Debug)]
pub struct FileOperationResult {
    pub action: String,
    pub path: String,
    pub success: bool,
    pub output: String,
}

/// Holds the result of a single command execution
#[derive(Debug)]
pub struct CommandExecutionResult {
    pub command: String,
    pub exit_code: i32,
    pub output: String,
    pub success: bool,
}

/// Represents a session with the Prime assistant
pub struct PrimeSession {
    // Base paths
    pub base_dir: PathBuf,
    pub session_id: String,
    pub session_dir: PathBuf,
    
    // Message tracking
    pub message_counter: AtomicUsize,
    
    // Ollama configuration
    pub ollama_model: String,
    pub ollama_api_url: String,
    
    // Components
    pub command_processor: CommandProcessor,
    pub memory_manager: MemoryManager,
    
    // HTTP client
    client: Client,
}

// Helper functions for parsing attributes from FILE_OP_RE
fn get_bool_attr(attrs: &std::collections::HashMap<String, String>, key: &str, default: bool) -> bool {
    attrs.get(key).and_then(|s| s.parse().ok()).unwrap_or(default)
}

fn get_string_attr<'a>(attrs: &'a std::collections::HashMap<String, String>, key: &str, default: &'a str) -> &'a str {
    attrs.get(key).map(|s| s.as_str()).unwrap_or(default)
}

impl PrimeSession {
    /// Create a new Prime session
    pub fn new(base_dir: PathBuf, ollama_model: &str, ollama_api_base: &str) -> Result<Self> {
        // Create session ID with timestamp
        let session_id = format!("session_{}", chrono::Local::now().format("%Y%m%d_%H%M%S"));
        
        // Create required directories
        let session_dir = base_dir.join("conversations").join(&session_id);
        fs::create_dir_all(&session_dir)?;
        
        // Create memory directory if it doesn't exist
        let memory_dir = base_dir.join("memory");
        fs::create_dir_all(&memory_dir)?;
        
        // Initialize memory files if they don't exist
        for memory_file in ["long_term.md", "short_term.md"].iter() {
            let file_path = memory_dir.join(memory_file);
            if !file_path.exists() {
                let header = format!("# Prime {} Memory\n\n", 
                    if *memory_file == "long_term.md" { "Long-term" } else { "Short-term" });
                fs::write(&file_path, header)?;
            }
        }
        
        // Create HTTP client
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, header::HeaderValue::from_static("Prime-Assistant/1.0"));

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .gzip(true)
            .default_headers(headers) // Set default headers with User-Agent
            .build()
            .context("Failed to create HTTP client")?;
        
        // Create session
        let session = Self {
            base_dir: base_dir.clone(),
            session_id,
            session_dir,
            message_counter: AtomicUsize::new(0),
            ollama_model: ollama_model.to_string(),
            ollama_api_url: format!("{}/api/generate", ollama_api_base.trim_end_matches('/')),
            command_processor: CommandProcessor::new(),
            memory_manager: MemoryManager::new(base_dir.join("memory")),
            client,
        };
        
        Ok(session)
    }
    
    /// Get the next sequential message number
    fn next_message_number(&self) -> usize {
        self.message_counter.fetch_add(1, Ordering::SeqCst) + 1
    }
    
    /// Add a user message to the conversation
    pub fn add_user_message(&self, content: &str) -> Result<PathBuf> {
        let message_number = self.next_message_number();
        let file_name = format!("{:03}_user.md", message_number);
        let file_path = self.session_dir.join(&file_name);
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let message_content = format!("# User Message\nTimestamp: {}\n\n{}", timestamp, content);
        
        fs::write(&file_path, message_content)?;
        Ok(file_path)
    }
    
    /// Add a Prime (AI) message to the conversation
    pub fn add_prime_message(&self, content: &str) -> Result<PathBuf> {
        let message_number = self.next_message_number();
        let file_name = format!("{:03}_prime.md", message_number);
        let file_path = self.session_dir.join(&file_name);
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let message_content = format!("# Prime Response\nTimestamp: {}\n\n{}", timestamp, content);
        
        fs::write(&file_path, message_content)?;
        Ok(file_path)
    }
    
    /// Add a system message to the conversation (command output)
    pub fn add_system_message(&self, command: &str, exit_code: i32, output: &str) -> Result<PathBuf> {
        let message_number = self.next_message_number();
        let file_name = format!("{:03}_system.md", message_number);
        let file_path = self.session_dir.join(&file_name);
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let success_style = Style::new().green();
        let error_style = Style::new().red();
        let command_style = Style::new().cyan();
        
        // Print command execution feedback
        if exit_code == 0 {
            println!("{} {}",
                success_style.apply_to("✔"), // Using a checkmark for success
                command_style.apply_to(command)
            );
        } else {
            println!("{} {} (exit code: {})",
                error_style.apply_to("✖"), // Using an X for error
                command_style.apply_to(command),
                error_style.apply_to(exit_code.to_string())
            );
        }
        
        // Format message content with status
        let status_text = if exit_code == 0 { "Success" } else { "Error" };
        let message_content = format!(
            "# System Output\nTimestamp: {}\nStatus: {}\nCommand: {}\nExit Code: {}\n\n```\n{}\n```",
            timestamp, status_text, command, exit_code, output
        );
        
        fs::write(&file_path, message_content)?;
        Ok(file_path)
    }
    
    /// Generate a streamed response from Prime using the LLM, with a spinner while waiting.
    ///
    /// `current_turn_prompt`: The specific prompt for this turn (e.g., user input or error correction details).
    /// `is_error_correction_turn`: True if this is a follow-up to correct previous errors.
    pub async fn generate_prime_response_stream(&self, current_turn_prompt: &str, is_error_correction_turn: bool) -> Result<String> {
        let mut ollama_prompt_payload = String::new();

        // 1. System Instructions
        ollama_prompt_payload.push_str(&self.get_system_prompt()?);
        ollama_prompt_payload.push_str("\n\n");

        // 2. Recent Conversation History (if any)
        let history_limit = 10; // Number of past messages to include
        let conversation_history = self.get_full_conversation_history_prompt(history_limit)?;
        if !conversation_history.is_empty() {
            ollama_prompt_payload.push_str("## Recent Conversation History:\n");
            ollama_prompt_payload.push_str(&conversation_history);
        }

        // 3. The Current Task/Prompt for the LLM
        if is_error_correction_turn {
            ollama_prompt_payload.push_str("## Error Correction Task:\n");
        } else {
            ollama_prompt_payload.push_str("## Current User Request:\n");
        }
        ollama_prompt_payload.push_str(current_turn_prompt);
        ollama_prompt_payload.push_str("\n\n# Prime Response:\n"); // Cue for LLM's response

        // Setup spinner
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::with_template("{spinner:.blue.bold} {msg}") // Added color and bold to spinner
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏") // Using a different set of spinner characters
        );
        spinner.enable_steady_tick(Duration::from_millis(80));
        spinner.set_message("Thinking...");

        // Call Ollama API with streaming
        let response = self.client.post(&self.ollama_api_url)
            .json(&json!({
                "model": self.ollama_model,
                "prompt": ollama_prompt_payload,
                "stream": true,
                "options": {
                    "temperature": 0.5,
                    "top_p": 0.9
                }
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

        // Process the byte stream
        let mut full_response = String::new();
        let mut stream = response.bytes_stream();
        
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.context("Stream error")?;
            
            // Process newline-delimited JSON responses
            for piece in std::str::from_utf8(&bytes)?.split('\n') {
                if piece.trim().is_empty() { continue }
                if let Ok(obj) = serde_json::from_str::<serde_json::Value>(piece) {
                    if let Some(tok) = obj.get("response").and_then(|v| v.as_str()) {
                        // First real token? Stop spinner
                        spinner.finish_and_clear();
                        
                        // Print token immediately
                        print!("{}", tok);
                        io::stdout().flush().unwrap();
                        full_response.push_str(tok);
                    }
                }
            }
        }

        // Clean up & newline
        println!();

        // Save the AI response
        self.add_prime_message(&full_response)?;
        
        // Print completion message
        let status_style = Style::new().green();
        println!("\n{}", status_style.apply_to("✓ Response complete"));
        
        Ok(full_response)
    }
    
    /// Get a string representation of the recent conversation history.
    fn get_full_conversation_history_prompt(&self, limit: usize) -> Result<String> {
        let mut context_str = String::new();
        // Get messages
        let messages = self.get_messages(Some(limit))?;
        for message in messages {
            context_str.push_str(&message.content); // These already have headers like "# User Message"
            context_str.push_str("\n\n");
        }
        Ok(context_str)
    }
    
    /// Get system prompt for Prime
    fn get_system_prompt(&self) -> Result<String> {
        const PROMPT_TEMPLATE: &str = include_str!("../prompts/system_prompt.md");

        let memory_content = self.memory_manager.read_memory(None)
            .context("Failed to read memory for system prompt")?;
        
        let final_prompt = PROMPT_TEMPLATE.replace("{{MEMORY_CONTEXT}}", &memory_content);
        
        Ok(final_prompt)
    }
    
    /// Process any commands in Prime's response
    pub async fn process_commands(&self, response: &str) -> Result<Vec<ProcessedItemResult>> {
        let mut results = Vec::new();
        let mut found_structured_op = false; // Used to track if any .web_op or .file_op was found

        // --- 1. Process Web Operations ---
        for cap in WEB_OP_RE.captures_iter(response) {
            found_structured_op = true;
            let action_str = cap.name("action").map_or("", |m| m.as_str()).to_lowercase();
            let url_str = cap.name("url").map_or("", |m| m.as_str());

            if action_str == "fetch_text" {
                // TODO: Replace with actual call to crate::web_ops::handle_fetch_text_web_op(&self.client, url_str).await;
                // For now, using a placeholder direct result for structure.
                // let handler_result = Ok(format!("Successfully fetched (placeholder) text from URL: {}", url_str));
                let handler_result = crate::web_ops::handle_fetch_text_web_op(&self.client, url_str).await;


                let (success, output) = match handler_result {
                    Ok(content) => (true, content),
                    Err(e) => (false, e.to_string()),
                };

                let log_summary = format!("web_fetch_text {}", url_str);
                let log_exit_code = if success { 0 } else { -1 };
                self.add_system_message(&log_summary, log_exit_code, &output)?;

                results.push(ProcessedItemResult::WebOp(WebOperationResult {
                    action: action_str.clone(),
                    url: url_str.to_string(),
                    success,
                    output,
                }));
            } else {
                log::warn!("Unknown web operation action: {}", action_str);
                // Optionally, add a result indicating this unknown action
                results.push(ProcessedItemResult::WebOp(WebOperationResult {
                    action: action_str.clone(),
                    url: url_str.to_string(),
                    success: false,
                    output: format!("Unknown web operation action: {}", action_str),
                }));
            }
        }

        // --- 2. Process File Operations ---
        // Only proceed if no web ops claimed the response, or make sure regexes are mutually exclusive.
        // Assuming .web_op and .file_op are distinct and won't match the same block.
        for cap in FILE_OP_RE.captures_iter(response) {
            found_structured_op = true; // A structured operation was found
            let action = cap.name("action").map_or("", |m| m.as_str()).to_lowercase();
            let path_str = cap.name("path").map_or("", |m| m.as_str());
            let attributes_str = cap.name("attributes").map_or("", |m| m.as_str());
            let content_str = cap.name("content").map_or("", |m| m.as_str());

            let resolved_path = self.base_dir.join(path_str.trim());

            let mut attributes_map = std::collections::HashMap::new();
            for attr_cap in DATA_ATTR_RE.captures_iter(attributes_str) {
                let key = attr_cap.name("key").map_or("", |m| m.as_str()).to_string();
                let value = attr_cap.name("value").map_or("", |m| m.as_str()).to_string();
                attributes_map.insert(key, value);
            }

            let overwrite = get_bool_attr(&attributes_map, "overwrite", false);
            let recursive = get_bool_attr(&attributes_map, "recursive", false);
            let diff_format = get_string_attr(&attributes_map, "diff-format", "unified").to_string(); // owned

            let operation_result: Result<String> = match action.as_str() {
                "file_create" => {
                    crate::commands::command_utils::handle_create_file(&resolved_path, content_str.trim_end_matches('\n'), overwrite)
                }
                "file_read" => {
                    crate::commands::command_utils::handle_read_file(&resolved_path)
                }
                "file_update" => {
                    crate::commands::command_utils::handle_update_file(&resolved_path, content_str.trim_end_matches('\n'))
                }
                "file_delete" => {
                    crate::commands::command_utils::handle_delete_file(&resolved_path, recursive)
                }
                "file_patch" => {
                    crate::commands::command_utils::handle_patch_file(&resolved_path, content_str.trim_matches('\n'), &diff_format)
                }
                unknown_action => {
                    log::warn!("Unknown file operation action: {}", unknown_action);
                    Err(anyhow::anyhow!("Unknown file operation action: {}", unknown_action))
                }
            };

            let (success, output_str) = match operation_result {
                Ok(s) => (true, s),
                Err(e) => (false, e.to_string()),
            };

            let file_op_summary_for_log = format!("file_{} {}", action, path_str);
            let log_exit_code = if success { 0 } else { -1 };
            self.add_system_message(&file_op_summary_for_log, log_exit_code, &output_str)?;

            results.push(ProcessedItemResult::FileOp(FileOperationResult {
                action, // action is already to_lowercase()
                path: path_str.to_string(),
                success,
                output: output_str,
            }));
        }

        // --- 3. Process Shell Commands if no structured operations (web or file) were found ---
        if !found_structured_op {
            let mut found_shell_commands = false; // Tracks if any shell block (pandoc or fallback) was found
            let execute_and_record = |command_str: &str, results_vec: &mut Vec<ProcessedItemResult>| -> Result<()> {
                if command_str.is_empty() {
                    return Ok(());
                }
                let command_style = Style::new().cyan();
                println!("\n{}", command_style.apply_to("Executing command:"));
                let (exit_code, output) = self.command_processor.execute_command(command_str, None)?; // Assuming execution in base_dir for now
                self.add_system_message(command_str, exit_code, &output)?;
                results_vec.push(ProcessedItemResult::Command(CommandExecutionResult {
                    command: command_str.to_string(),
                    exit_code,
                    output,
                    success: exit_code == 0,
                }));
                Ok(())
            };

            for cap in PANDOC_RE.captures_iter(response) {
                found_shell_commands = true;
                if let Some(cmd_match) = cap.get(1) {
                    execute_and_record(cmd_match.as_str().trim(), &mut results)?;
                }
            }

            if !found_shell_commands { // Only try fallback if no Pandoc shell commands were found
                for cap in FALLBACK_RE.captures_iter(response) {
                    found_shell_commands = true; // Mark that a fallback shell command was found
                    if let Some(cmd_match) = cap.get(1) {
                        execute_and_record(cmd_match.as_str().trim(), &mut results)?;
                    }
                }
            }
        }
        Ok(results)
    }
    
    /// Get list of messages in the session
    pub fn get_messages(&self, limit: Option<usize>) -> Result<Vec<Message>> {
        let entries = fs::read_dir(&self.session_dir)?;
        
        let mut messages = Vec::new();
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                // Extract message number and type
                if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                    let parts: Vec<&str> = file_name.splitn(2, '_').collect();
                    
                    if parts.len() < 2 {
                        continue;
                    }
                    
                    let number = parts[0].parse::<usize>().unwrap_or(0);
                    let msg_type = parts[1].split('.').next().unwrap_or("unknown");
                    
                    // Read file content
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
        
        // Sort by message number
        messages.sort_by_key(|m| m.number);
        
        // Apply limit if requested
        if let Some(limit) = limit {
            if messages.len() > limit {
                // Take the most recent messages
                messages = messages.into_iter().rev().take(limit).collect();
                messages.sort_by_key(|m| m.number);
            }
        }
        
        Ok(messages)
    }
    
    /// List messages in the current session
    pub fn list_messages(&self) -> Result<Vec<String>> {
        let messages = self.get_messages(None)?;
        
        let mut result = Vec::new();
        for message in messages {
            // Extract first line of content for summary
            let first_line = message.content.lines()
                .filter(|line| !line.is_empty())
                .next()
                .unwrap_or("[Empty message]");
            
            result.push(format!("{:03} - {}: {}", message.number, message.msg_type, first_line));
        }
        
        Ok(result)
    }
    
    /// Read a specific message by number
    pub fn read_message(&self, number: usize) -> Result<String> {
        // Format number with leading zeros
        let number_str = format!("{:03}", number);
        
        // Find matching files
        let entries = fs::read_dir(&self.session_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                    if file_name.starts_with(&number_str) {
                        return Ok(fs::read_to_string(&path)
                            .context(format!("Failed to read message file: {}", path.display()))?);
                    }
                }
            }
        }
        
        Err(anyhow::anyhow!("Message {} not found", number))
    }
    
    /// Read memory (wrapper for memory manager)
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