// session.rs
// Enhanced session management with cleaner flow control
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::{Context as AnyhowContext, Result};
use chrono;
use console::Style;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, header};
use serde_json::json;

use crate::commands::CommandProcessor;
use crate::styling::STYLER;

/// Represents a parsed script block
#[derive(Debug, Clone)]
pub struct ScriptBlock {
    pub attributes: HashMap<String, String>,
    pub content: String,
}

/// Represents the result of processing a script block
#[derive(Debug)]
pub struct ScriptProcessingResult {
    pub script_content: String,
    pub operations_performed: Vec<String>,
    pub saved_to: Option<String>,
    pub executed: bool,
    pub exit_code: Option<i32>,
    pub output: String,
    pub return_value: Option<String>,
    pub completed: bool,
    pub success: bool,
}

/// Represents the result of processing all script blocks
#[derive(Debug)]
pub struct ProcessingSessionResult {
    pub script_results: Vec<ScriptProcessingResult>,
    pub has_completed: bool,
    pub final_message: Option<String>,
    pub execution_summary: String,
}

/// Enhanced markdown parser for Pandoc attributed code blocks
pub struct MarkdownParser;

impl MarkdownParser {
    /// Parse markdown content and extract script blocks
    pub fn parse_script_blocks(content: &str) -> Vec<ScriptBlock> {
        let lines: Vec<&str> = content.lines().collect();
        let mut script_blocks = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();
            
            // Look for code block start with script class
            if line.starts_with("```{.script") || line.starts_with("```{.text") || line.starts_with("```{.powershell") { // Added .text and .powershell for flexibility
                if let Some(script_block) = Self::parse_single_block(&lines, i) {
                    script_blocks.push(script_block.0);
                    i = script_block.1; // Jump to end of this block
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        script_blocks
    }

    /// Parse a single script block starting at the given line index
    fn parse_single_block(lines: &[&str], start_idx: usize) -> Option<(ScriptBlock, usize)> {
        let start_line = lines[start_idx].trim();
        
        // Parse attributes from the opening line
        let attributes = Self::parse_attributes(start_line);
        
        // Find the content between ``` markers
        let mut content_lines = Vec::new();
        let mut end_idx = start_idx + 1;
        
        while end_idx < lines.len() {
            let line = lines[end_idx];
            if line.trim() == "```" {
                break;
            }
            content_lines.push(line);
            end_idx += 1;
        }
        
        if end_idx >= lines.len() {
            // No closing ``` found
            return None;
        }
        
        let content = content_lines.join("\n");
        
        Some((ScriptBlock {
            attributes,
            content,
        }, end_idx + 1))
    }

    /// Parse attributes from opening line like ```{.script attr1="value1" attr2="value2"}
    fn parse_attributes(line: &str) -> HashMap<String, String> {
        let mut attributes = HashMap::new();
        
        // Find the content between { and }
        if let Some(start) = line.find('{') {
            if let Some(end) = line.rfind('}') {
                let attr_str = &line[start + 1..end];
                
                // Remove the class part (e.g., ".script", ".text", ".powershell")
                let attr_str = attr_str.split_whitespace().skip_while(|s| s.starts_with('.')).collect::<Vec<&str>>().join(" ");
                let mut chars: Vec<char> = attr_str.chars().collect();
                let mut i = 0;
                
                while i < chars.len() {
                    // Skip whitespace
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    
                    if i >= chars.len() {
                        break;
                    }
                    
                    // Read key
                    let mut key = String::new();
                    while i < chars.len() && chars[i] != '=' && !chars[i].is_whitespace() {
                        key.push(chars[i]);
                        i += 1;
                    }
                    
                    // Skip whitespace and '='
                    while i < chars.len() && (chars[i].is_whitespace() || chars[i] == '=') {
                        i += 1;
                    }
                    
                    if i >= chars.len() {
                        break;
                    }
                    
                    // Read value
                    let mut value = String::new();
                    if chars[i] == '"' {
                        // Quoted value - handle escaped quotes
                        i += 1; // Skip opening quote
                        while i < chars.len() && chars[i] != '"' {
                            if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == '"' {
                                value.push('"');
                                i += 2; // Skip escaped quote
                            } else {
                                value.push(chars[i]);
                                i += 1;
                            }
                        }
                        if i < chars.len() {
                            i += 1; // Skip closing quote
                        }
                    } else {
                        // Unquoted value
                        while i < chars.len() && !chars[i].is_whitespace() {
                            value.push(chars[i]);
                            i += 1;
                        }
                    }
                    
                    if !key.is_empty() && !value.is_empty() {
                        attributes.insert(key, value);
                    }
                }
            }
        }
        
        attributes
    }
}

/// Represents a session with the Prime assistant
pub struct PrimeSession {
    pub base_dir: PathBuf,
    pub session_id: String,
    pub session_dir: PathBuf,
    pub message_counter: AtomicUsize,
    pub ollama_model: String,
    pub ollama_api_url: String,
    pub command_processor: CommandProcessor,
    client: Client,
}

impl PrimeSession {
    /// Create a new Prime session
    pub fn new(base_dir: PathBuf, ollama_model: &str, ollama_api_base: &str) -> Result<Self> {
        let session_id = format!("session_{}", chrono::Local::now().format("%Y%m%d_%H%M%S"));
        
        // Create required directories
        let session_dir = base_dir.join("conversations").join(&session_id);
        fs::create_dir_all(&session_dir)?;
        
        let scripts_dir = base_dir.join("scripts");
        fs::create_dir_all(&scripts_dir)?;
        
        // Create HTTP client
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, header::HeaderValue::from_static("Prime-Assistant/1.0"));

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .gzip(true)
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;
        
        let session = Self {
            base_dir: base_dir.clone(),
            session_id,
            session_dir,
            message_counter: AtomicUsize::new(0),
            ollama_model: ollama_model.to_string(),
            ollama_api_url: format!("{}/api/generate", ollama_api_base.trim_end_matches('/')),
            command_processor: CommandProcessor::new(),
            client,
        };
        
        Ok(session)
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
    
    pub fn add_system_message(&self, operation: &str, status: &str, details: &str) -> Result<PathBuf> {
        let message_number = self.next_message_number();
        let file_name = format!("{:03}_system.md", message_number);
        let file_path = self.session_dir.join(&file_name);
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        
        // Print operation feedback with enhanced styling
        let operation_style = Style::new().cyan().bold();
        let success_style = Style::new().green();
        let error_style = Style::new().red();
        let info_style = Style::new().blue();
        
        match status {
            "SUCCESS" => println!("{} {}", success_style.apply_to("✔"), operation_style.apply_to(operation)),
            "FAILED" | "ERROR" => println!("{} {}", error_style.apply_to("✖"), operation_style.apply_to(operation)),
            _ => println!("{} {}", info_style.apply_to("•"), operation_style.apply_to(operation)),
        }
        
        let message_content = format!(
            "# System Operation\nTimestamp: {}\nOperation: {}\nStatus: {}\n\n{}", 
            timestamp, operation, status, details
        );
        
        fs::write(&file_path, message_content)?;
        Ok(file_path)
    }
    
    /// Generate a streamed response from Prime using the LLM
    pub async fn generate_prime_response_stream(&self, prompt: &str) -> Result<String> {
        let mut ollama_prompt_payload = String::new();

        // System Instructions
        ollama_prompt_payload.push_str(&self.get_system_prompt()?);
        ollama_prompt_payload.push_str("\n\n");

        // Recent conversation history (limit to prevent context overflow)
        let history_limit = 8;
        let conversation_history = self.get_conversation_history(history_limit)?;
        if !conversation_history.is_empty() {
            ollama_prompt_payload.push_str("## Recent Conversation:\n");
            ollama_prompt_payload.push_str(&conversation_history);
            ollama_prompt_payload.push_str("\n\n");
        }

        // Current prompt
        ollama_prompt_payload.push_str("## Current Request:\n");
        ollama_prompt_payload.push_str(prompt);
        ollama_prompt_payload.push_str("\n\n# Prime Response:\n");

        // Setup enhanced spinner
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::with_template("{spinner:.blue.bold} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        );
        spinner.enable_steady_tick(Duration::from_millis(80));
        spinner.set_message("Prime is thinking...");

        // Call Ollama API
        let response = self.client.post(&self.ollama_api_url)
            .json(&json!({
                "model": self.ollama_model,
                "prompt": ollama_prompt_payload,
                "stream": true,
                "options": {
                    "temperature": 0.3,
                    "top_p": 0.9,
                    "num_ctx": 8192
                }
            }))
            .send()
            .await
            .context("Failed to send request to Ollama API")?;

        if !response.status().is_success() {
            spinner.finish_and_clear();
            return Err(anyhow::anyhow!(
                "Ollama API error ({}): {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let mut full_response = String::new();
        let mut stream = response.bytes_stream();
        let mut first_token = true;
        
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.context("Stream error")?;
            
            for piece in std::str::from_utf8(&bytes)?.split('\n') {
                if piece.trim().is_empty() { continue }
                if let Ok(obj) = serde_json::from_str::<serde_json::Value>(piece) {
                    if let Some(tok) = obj.get("response").and_then(|v| v.as_str()) {
                        if first_token {
                            spinner.finish_and_clear();
                            first_token = false;
                        }
                        print!("{}", tok);
                        io::stdout().flush().unwrap();
                        full_response.push_str(tok);
                    }
                }
            }
        }

        println!();
        self.add_prime_message(&full_response)?;
        
        let status_style = Style::new().green();
        println!("\n{}", status_style.apply_to("✓ Response generated"));
        
        Ok(full_response)
    }
    
    fn get_conversation_history(&self, limit: usize) -> Result<String> {
        let mut context_str = String::new();
        let messages = self.get_messages(Some(limit))?;
        for message in messages {
            context_str.push_str(&message.content);
            context_str.push_str("\n\n");
        }
        Ok(context_str)
    }
    
    fn get_system_prompt(&self) -> Result<String> {
        const PROMPT_TEMPLATE: &str = include_str!("../prompts/system_prompt.md");
        Ok(PROMPT_TEMPLATE.to_string())
    }
    
    /// Enhanced variable substitution
    fn substitute_variables(&self, text: String, script_path: Option<String>, script_content: String) -> String {
        let mut result = text;
        
        // Replace variables in order of specificity
        result = result.replace("${workspace}", &self.base_dir.to_string_lossy());
        result = result.replace("${this.content}", &script_content);
        
        if let Some(path) = script_path {
            result = result.replace("${this.path}", &path);
        }
        
        result
    }
    
    /// Enhanced script processing with cleaner flow
    pub async fn process_commands(&self, response: &str) -> Result<ProcessingSessionResult> {
        let script_blocks = MarkdownParser::parse_script_blocks(response);
        
        if script_blocks.is_empty() {
            println!("{} No script blocks found in response", STYLER.info_style("•"));
            return Ok(ProcessingSessionResult {
                script_results: Vec::new(),
                has_completed: false,
                final_message: None,
                execution_summary: "No script blocks to execute.".to_string(),
            });
        }
        
        println!("{} Found {} script block(s)", STYLER.info_style("•"), script_blocks.len());
        
        let mut script_results = Vec::new();
        let mut has_completed = false;
        let mut final_message = None;
        let mut execution_outputs = Vec::new();
        
        for (idx, block) in script_blocks.iter().enumerate() {
            println!("\n{} Processing script block {}/{}", 
                STYLER.info_style("→"), idx + 1, script_blocks.len());
            
            let mut script_result = ScriptProcessingResult {
                script_content: block.content.clone(),
                operations_performed: Vec::new(),
                saved_to: None,
                executed: false,
                exit_code: None,
                output: String::new(),
                return_value: None,
                completed: block.attributes.get("completed").map(|v| v == "true").unwrap_or(false),
                success: true,
            };
            
            let mut current_content = block.content.clone();
            let mut script_file_path: Option<PathBuf> = None; // Use PathBuf for consistent path handling
            
            // Handle save operation
            if let Some(save_path_str) = block.attributes.get("save") {
                let resolved_path = self.base_dir.join(save_path_str);
                
                if let Some(parent) = resolved_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
                }
                
                fs::write(&resolved_path, &current_content)
                    .with_context(|| format!("Failed to save file: {}", resolved_path.display()))?;
                
                script_result.saved_to = Some(save_path_str.clone());
                script_result.operations_performed.push(format!("Saved to: {}", save_path_str));
                script_file_path = Some(resolved_path); // Store the actual PathBuf
                
                self.add_system_message(
                    &format!("save: {}", save_path_str),
                    "SUCCESS",
                    "File saved successfully"
                )?;
                
                execution_outputs.push(format!("✓ File saved: {}", save_path_str));
            }
            
            // Handle find/replace operations
            if let (Some(find_pattern), Some(replace_with), Some(path_str)) = 
                (block.attributes.get("find"), block.attributes.get("replace"), block.attributes.get("path")) {
                
                let target_path = self.base_dir.join(path_str);

                if !target_path.exists() {
                    let error = format!("Target file for find/replace not found: {}", target_path.display());
                    println!("{} {}", STYLER.error_style("✖"), error);
                    execution_outputs.push(format!("✖ {}", error));
                    script_result.success = false;
                } else {
                    let file_content = fs::read_to_string(&target_path)
                        .with_context(|| format!("Failed to read file for find/replace: {}", target_path.display()))?;
                    
                    let new_content = file_content.replace(find_pattern, replace_with);
                    fs::write(&target_path, &new_content)
                        .with_context(|| format!("Failed to write updated file: {}", target_path.display()))?;
                    
                    script_result.operations_performed.push(format!("Find/Replace in {}: '{}' → '{}'", path_str, find_pattern, replace_with));
                    
                    self.add_system_message(
                        &format!("find/replace in: {}", path_str),
                        "SUCCESS",
                        &format!("Replaced '{}' with '{}' in {}", find_pattern, replace_with, path_str)
                    )?;
                    
                    execution_outputs.push(format!("✓ Find/Replace completed in {}", path_str));
                }
            }
            
            // Handle patch operations using start and end lines
            if let (Some(patch_content), Some(start_line_str), Some(end_line_str), Some(path_str)) = 
                (block.attributes.get("patch"), block.attributes.get("start"), block.attributes.get("end"), block.attributes.get("path")) {
                
                let target_path = self.base_dir.join(path_str);

                let start_line = start_line_str.parse::<usize>().context("Invalid start line number")?;
                let end_line = end_line_str.parse::<usize>().context("Invalid end line number")?;

                if !target_path.exists() {
                    let error = format!("Target file for patch not found: {}", target_path.display());
                    println!("{} {}", STYLER.error_style("✖"), error);
                    execution_outputs.push(format!("✖ {}", error));
                    script_result.success = false;
                } else {
                    let file_content = fs::read_to_string(&target_path)
                        .with_context(|| format!("Failed to read file for patching: {}", target_path.display()))?;
                    
                    let lines: Vec<&str> = file_content.lines().collect();
                    
                    if start_line == 0 || end_line == 0 || start_line > end_line || start_line > lines.len() {
                         let error = format!("Invalid line range for patching: start={}, end={}, total_lines={}", start_line, end_line, lines.len());
                         println!("{} {}", STYLER.error_style("✖"), error);
                         execution_outputs.push(format!("✖ {}", error));
                         script_result.success = false;
                    } else {
                        let mut new_lines = Vec::new();
                        for (i, line) in lines.into_iter().enumerate() {
                            let current_line_num = i + 1;
                            if current_line_num < start_line || current_line_num > end_line {
                                new_lines.push(line);
                            } else if current_line_num == start_line {
                                // Insert the patch content at the start line
                                new_lines.push(patch_content);
                            }
                            // Lines between start_line and end_line (exclusive of start_line, inclusive of end_line) are effectively replaced by patch_content
                        }
                        
                        let new_content = new_lines.join("\n");
                        fs::write(&target_path, &new_content)
                            .with_context(|| format!("Failed to write patched file: {}", target_path.display()))?;
                        
                        script_result.operations_performed.push(format!("Patched lines {}-{} in: {}", start_line, end_line, path_str));
                        
                        self.add_system_message(
                            &format!("patch in: {}", path_str),
                            "SUCCESS",
                            &format!("Patched content into {} from line {} to {}", path_str, start_line, end_line)
                        )?;
                        
                        execution_outputs.push(format!("✓ Patch applied to {} (lines {}-{})", path_str, start_line, end_line));
                    }
                }
            }
            
            // Handle execution
            if let Some(execute_cmd) = block.attributes.get("execute") {
                let path_str = script_file_path.as_ref().map(|p| p.to_string_lossy().into_owned());
                let command = self.substitute_variables(
                    execute_cmd.to_string(),
                    path_str,
                    current_content.clone()
                );
                
                println!("{} Executing: {}", STYLER.info_style("→"), command);
                
                let (exit_code, output) = self.command_processor.execute_command(&command, Some(&self.base_dir))?;
                
                script_result.executed = true;
                script_result.exit_code = Some(exit_code);
                script_result.success = script_result.success && exit_code == 0;
                script_result.output = output.clone();
                
                script_result.operations_performed.push("Executed command".to_string());
                
                let status = if exit_code == 0 { "SUCCESS" } else { "FAILED" };
                self.add_system_message(
                    "execute command",
                    status,
                    &format!("Exit code: {}\nOutput:\n```\n{}\n```", exit_code, output)
                )?;
                
                if exit_code == 0 {
                    execution_outputs.push(format!("✓ Command executed successfully\nOutput:\n{}", output));
                } else {
                    execution_outputs.push(format!("✖ Command failed (exit code: {})\nOutput:\n{}", exit_code, output));
                }
            }
            
            // Handle return value
            if let Some(return_template) = block.attributes.get("return") {
                let path_str = script_file_path.as_ref().map(|p| p.to_string_lossy().into_owned());
                let return_value = self.substitute_variables(
                    return_template.to_string(),
                    path_str,
                    current_content.clone()
                );
                script_result.return_value = Some(return_value.clone());
                script_result.operations_performed.push(format!("Return: {}", return_value));
                
                if script_result.completed {
                    final_message = Some(return_value.clone());
                    println!("\n{} {}", STYLER.success_style("✓ Task completed:"), return_value);
                } else {
                    println!("{} {}", STYLER.info_style("→"), return_value);
                }
            }
            
            // Check for completion
            if script_result.completed {
                has_completed = true;
                println!("{} Script block marked as completed", STYLER.success_style("✓"));
            }
            
            script_results.push(script_result);
        }
        
        // Create execution summary for LLM
        let execution_summary = if execution_outputs.is_empty() {
            "Script blocks processed with no output.".to_string()
        } else {
            format!("Execution Results:\n\n{}", execution_outputs.join("\n\n"))
        };
        
        Ok(ProcessingSessionResult {
            script_results,
            has_completed,
            final_message,
            execution_summary,
        })
    }
    
    // Message handling methods remain the same
    pub fn get_messages(&self, limit: Option<usize>) -> Result<Vec<Message>> {
        let entries = fs::read_dir(&self.session_dir)?;
        let mut messages = Vec::new();
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                    let parts: Vec<&str> = file_name.splitn(2, '_').collect();
                    
                    if parts.len() < 2 {
                        continue;
                    }
                    
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
        
        if let Some(limit) = limit {
            if messages.len() > limit {
                messages = messages.into_iter().rev().take(limit).collect();
                messages.sort_by_key(|m| m.number);
            }
        }
        
        Ok(messages)
    }
    
    pub fn list_messages(&self) -> Result<Vec<String>> {
        let messages = self.get_messages(None)?;
        let mut result = Vec::new();
        
        for message in messages {
            let first_line = message.content.lines()
                .filter(|line| !line.is_empty())
                .next()
                .unwrap_or("[Empty message]");
            
            result.push(format!("{:03} - {}: {}", message.number, message.msg_type, first_line));
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
                        return Ok(fs::read_to_string(&path)
                            .context(format!("Failed to read message file: {}", path.display()))?);
                    }
                }
            }
        }
        
        Err(anyhow::anyhow!("Message {} not found", number))
    }
}

pub struct Message {
    pub number: usize,
    pub msg_type: String,
    pub path: PathBuf,
    pub content: String,
}