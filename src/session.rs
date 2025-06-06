// session.rs
// Session management and message handling for Prime
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::{Context as AnyhowContext, Result};
use chrono;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::Client;
use serde_json::{json, Value};

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
}

/// Holds the result of a single command execution
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
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
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
        let message_content = format!(
            "# System Output\nTimestamp: {}\nCommand: {}\nExit Code: {}\n\n```\n{}\n```",
            timestamp, command, exit_code, output
        );
        
        fs::write(&file_path, message_content)?;
        Ok(file_path)
    }
    
    /// Generate a response from Prime using the LLM.
    ///
    /// `current_turn_prompt`: The specific prompt for this turn (e.g., user input or error correction details).
    /// `is_error_correction_turn`: True if this is a follow-up to correct previous errors.
    pub fn generate_prime_response(&self, current_turn_prompt: &str, is_error_correction_turn: bool) -> Result<String> {
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

        // Call Ollama API
        let response = self.client.post(&self.ollama_api_url)
            .json(&json!({
                "model": self.ollama_model,
                "prompt": ollama_prompt_payload,
                "stream": false,
                "options": {
                    "temperature": 0.5, // Slightly lower for more deterministic corrections
                    "top_p": 0.9
                }
            }))
            .send()
            .context("Failed to send request to Ollama API")?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Ollama API error ({}): {}", 
                response.status(), 
                response.text().unwrap_or_default()
            ));
        }
        
        // Parse response
        let response_json: Value = response.json()
            .context("Failed to parse Ollama API response")?;
        
        let generated_text = response_json["response"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid Ollama response format: 'response' field missing or not a string"))?
            .trim()
            .to_string();
        
        // Save the AI response
        self.add_prime_message(&generated_text)?;
        
        Ok(generated_text)
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
        let memory = self.memory_manager.read_memory(None)?;
        
        let system_prompt = format!(
            "# Prime System Instructions

You are Prime, an advanced terminal assistant that helps users manage and configure their systems.
You can execute shell commands by including them in properly formatted Pandoc attributed markdown code blocks.

## Communication Guidelines
- Respond in a clear, concise manner
- When suggesting actions, provide specific commands in code blocks with proper Pandoc attributes
- After complex operations, summarize what was done
- If you need to remember something important, mention it explicitly

## Command Execution
When you want to run a shell command, include it in a code block with Pandoc attributes like this:
```{{.powershell data-action=\"execute\"}}
Get-Date
```

The system will automatically execute these commands and capture their output.
Wait for command results before continuing with multi-step processes.

## Memory Context
The following represents your current memory about the user's system:

{}

## Guidelines
- For complex tasks, break them down into step-by-step commands
- Always check command results before proceeding with dependent steps
- Be careful with destructive operations (rm, fd, etc.)
- If unsure about a system state, run diagnostic commands first
- Always use proper Pandoc attributed markdown format for all code blocks
- You are currently in a Windows 11 environment using PowerShell
- Use PowerShell commands rather than Unix/Linux commands
",
            memory
        );
        
        Ok(system_prompt)
    }
    
    /// Process any commands in Prime's response
    pub fn process_commands(&self, response: &str) -> Result<Vec<CommandExecutionResult>> {
        let mut results = Vec::new();
        let mut found_commands = false;

        // Helper closure to execute and record a command
        let execute_and_record = |command_str: &str, results_vec: &mut Vec<CommandExecutionResult>| -> Result<()> {
            if command_str.is_empty() {
                return Ok(());
            }
            // Logged by CommandProcessor now
            // println!("Executing command: {}", command_str);
            let (exit_code, output) = self.command_processor.execute_command(command_str)?;
            self.add_system_message(command_str, exit_code, &output)?;
            results_vec.push(CommandExecutionResult {
                command: command_str.to_string(),
                exit_code,
                output,
                success: exit_code == 0,
            });
            Ok(())
        };

        // First try to match Pandoc attributed blocks
        // Try Pandoc format first
        for cap in PANDOC_RE.captures_iter(response) {
            found_commands = true;
            if let Some(cmd_match) = cap.get(1) {
                execute_and_record(cmd_match.as_str().trim(), &mut results)?;
            }
        }
        
        // If no Pandoc blocks found, fall back to standard blocks
        if !found_commands {
            for cap in FALLBACK_RE.captures_iter(response) {
                // found_commands = true; // This assignment is redundant if the loop is entered.
                if let Some(cmd_match) = cap.get(1) {
                    execute_and_record(cmd_match.as_str().trim(), &mut results)?;
                }
            }
        }
        
        // If !found_commands, results will be empty.
        // The caller (Prime::process_user_input) checks if results.is_empty()
        // to determine if the LLM provided any executable commands.

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