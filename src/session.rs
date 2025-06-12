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
use serde_json::{json, Value};

use crate::commands::CommandProcessor;
use crate::styling::STYLER;
use crate::templates::TaskTemplates;

/// Represents a parsed script block
#[derive(Debug, Clone)]
pub struct ScriptBlock {
    pub attributes: HashMap<String, String>,
    pub content: String,
    pub step_number: Option<usize>,  // Track multi-step operations
    pub depends_on: Option<usize>,   // Dependencies between blocks
}

/// Represents the progress tracking of a task
#[derive(Debug)]
pub struct TaskProgress {
    total_steps: usize,
    completed_steps: usize,
    current_task: String,
    subtasks: Vec<(String, bool)>, // (task description, completed)
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
            step_number: None,
            depends_on: None,
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
                let chars: Vec<char> = attr_str.chars().collect();
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
/// Represents environment information detected at runtime
#[derive(Debug)]
pub struct EnvironmentInfo {
    os: String,
    python_version: Option<String>,
    pip_version: Option<String>,
    has_sudo: bool,
    in_venv: bool,
    has_git: bool,
    has_npm: bool,
}

impl PrimeSession {
    /// Track progress for a set of script blocks
    pub fn track_progress(&self, script_blocks: &[ScriptBlock]) -> TaskProgress {
        let total_steps = script_blocks.iter()
            .filter(|b| b.attributes.contains_key("execute"))
            .count();
            
        TaskProgress {
            total_steps,
            completed_steps: 0,
            current_task: String::new(),
            subtasks: Vec::new(),
        }
    }
    
    /// Update progress tracking information
    pub fn update_progress(&mut self, progress: &mut TaskProgress, description: &str, completed: bool) {
        if completed {
            progress.completed_steps += 1;
        }
        progress.current_task = description.to_string();
        progress.subtasks.push((description.to_string(), completed));
        
        // Print progress update
        let percentage = if progress.total_steps > 0 {
            (progress.completed_steps as f64 / progress.total_steps as f64 * 100.0) as usize
        } else {
            0
        };
        
        println!("{} Progress: {}% - {}",
            STYLER.info_style("•"),
            percentage,
            description
        );
    }
}

pub struct PrimeSession {
    pub base_dir: PathBuf,
    pub session_id: String,
    pub session_dir: PathBuf,
    pub message_counter: AtomicUsize,
    pub ollama_model: String,
    pub ollama_api_url: String,
    command_processor: CommandProcessor,
    templates: TaskTemplates,
    client: Client,
}

impl PrimeSession {
    pub fn render_template(&self, template_name: &str, data: &Value) -> Result<String> {
        self.templates.render_template(template_name, data)
    }

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
            templates: TaskTemplates::new(),
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
    pub async fn generate_prime_response_stream(&mut self, prompt: &str) -> Result<String> {
        let mut ollama_prompt_payload = String::new();

        // System Instructions
        let system_prompt = self.get_system_prompt()?;
        ollama_prompt_payload.push_str(&system_prompt);
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
    
    fn get_system_prompt(&mut self) -> Result<String> {
        let env_info = self.detect_environment();
        let mut prompt = include_str!("../prompts/system_prompt.md").to_string();
        
        prompt.push_str("\n\n## Current Environment\n");
        prompt.push_str(&format!("- OS: {}\n", env_info.os));
        prompt.push_str(&format!("- Python: {}\n", env_info.python_version.unwrap_or_else(|| "Not found".to_string())));
        prompt.push_str(&format!("- Pip: {}\n", env_info.pip_version.unwrap_or_else(|| "Not found".to_string())));
        prompt.push_str(&format!("- Has sudo: {}\n", env_info.has_sudo));
        prompt.push_str(&format!("- In virtualenv: {}\n", env_info.in_venv));
        prompt.push_str(&format!("- Has Git: {}\n", env_info.has_git));
        prompt.push_str(&format!("- Has npm: {}\n", env_info.has_npm));

        Ok(prompt)
    }

    /// Check if a command is available and get its version output
    fn check_command(&mut self, cmd: &str) -> Option<String> {
        let processor = &mut self.command_processor;
        match processor.execute_command(cmd, None) {
            Ok((0, output)) => Some(output.lines().next().unwrap_or("").to_string()),
            _ => None,
        }
    }

    /// Detect current environment information
    pub fn detect_environment(&mut self) -> EnvironmentInfo {
        EnvironmentInfo {
            os: std::env::consts::OS.to_string(),
            python_version: self.check_command("python --version"),
            pip_version: self.check_command("pip --version"),
            has_sudo: self.check_command("sudo --version").is_some(),
            in_venv: std::env::var("VIRTUAL_ENV").is_ok(),
            has_git: self.check_command("git --version").is_some(),
            has_npm: self.check_command("npm --version").is_some(),
        }
    }
    
    /// Enhanced variable substitution
    fn substitute_variables(&self, mut text: String, script_path: Option<String>, script_content: &str) -> String {
        // Replace variables in order of specificity
        text = text.replace("${workspace}", &self.base_dir.to_string_lossy());
        text = text.replace("${this.content}", script_content);
        
        if let Some(path) = script_path {
            text = text.replace("${this.path}", &path);
        }
        
        text
    }
    
    /// Enhanced script processing with cleaner flow
    pub async fn process_commands(&mut self, response: &str) -> Result<ProcessingSessionResult> {
        // Configure retry settings
        const MAX_RETRIES: u32 = 3;
        const BASE_DELAY_MS: u64 = 1000;

        // Initialize progress tracking
        let script_blocks = MarkdownParser::parse_script_blocks(response);
        let mut progress = self.track_progress(&script_blocks);
        let mut processor = &mut self.command_processor;
        
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
            
            // Update step number and dependencies from attributes
            if let Some(step_str) = block.attributes.get("step") {
                if let Ok(step_num) = step_str.parse::<usize>() {
                    if let Some(depends_str) = block.attributes.get("depends") {
                        if let Ok(depends_num) = depends_str.parse::<usize>() {
                            // Check dependency
                            if depends_num >= step_num {
                                println!("{} Invalid dependency: Step {} cannot depend on step {}",
                                    STYLER.error_style("✖"), step_num, depends_num);
                                continue;
                            }
                            // Check if dependency is satisfied
                            if !progress.subtasks.get(depends_num - 1)
                                .map(|(_, completed)| *completed)
                                .unwrap_or(false)
                            {
                                println!("{} Skipping step {}: Dependency on step {} not satisfied",
                                    STYLER.info_style("→"), step_num, depends_num);
                                continue;
                            }
                        }
                    }
                }
            }

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
            
            let current_content = block.content.clone();
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
            
            // Handle execution with retry logic
            if let Some(execute_cmd) = block.attributes.get("execute") {
                let path_str = script_file_path.as_ref().map(|p| p.to_string_lossy().into_owned());
                // Create owned copies before substitution
                let command_template = execute_cmd.to_string();
                let command = self.substitute_variables(
                    command_template,
                    path_str,
                    &current_content
                );

                println!("{} Executing: {} [{}/{}]",
                    STYLER.info_style("→"),
                    command,
                    progress.completed_steps + 1,
                    progress.total_steps
                );

                // Update progress before execution
                let description_str = block.attributes.get("description")
                    .map_or("Executing command".to_string(), |s| s.to_string());
                self.update_progress(&mut progress, &description_str, false);

                let mut attempt = 0;
                let mut last_error = None;
                let mut success = false;

                while attempt < MAX_RETRIES {
                    if attempt > 0 {
                        let delay = Duration::from_millis(BASE_DELAY_MS * 2_u64.pow(attempt - 1));
                        println!("{} Retry attempt {} of {}. Waiting {:?}...",
                            STYLER.info_style("↻"), attempt + 1, MAX_RETRIES, delay);
                        tokio::time::sleep(delay).await;
                    }

                    match self.command_processor.execute_command(&command, Some(&self.base_dir)) {
                        Ok((exit_code, output)) => {
                            script_result.executed = true;
                            script_result.exit_code = Some(exit_code);
                            script_result.output = output.clone();

                            // Check for common error patterns
                            if exit_code != 0 {
                                let error_patterns = [
                                    ("pip install", "permission denied", "Use --user flag or run with elevated privileges", true),
                                    ("pip install", "externally-managed-environment", "Use virtual environment or --break-system-packages", true),
                                    ("npm install", "EACCES", "Fix npm permissions or use --unsafe-perm", true),
                                    ("git clone", "Authentication failed", "Check credentials or use HTTPS URL", false),
                                    ("command not found", "", "Ensure required tool is installed and in PATH", false),
                                    ("cargo build", "could not find", "Run cargo update or check dependencies", true),
                                    ("mvn", "Could not resolve dependencies", "Check Maven repository access and settings", true),
                                    ("gradlew", "FAILURE: Build failed", "Check Gradle build errors", false),
                                    ("python", "ModuleNotFoundError", "Install missing Python package", true),
                                ];

                                for (cmd_pattern, error_pattern, suggestion, can_retry) in error_patterns {
                                    if command.contains(cmd_pattern) &&
                                       output.to_lowercase().contains(&error_pattern.to_lowercase()) {
                                        script_result.operations_performed.push(format!("Error Analysis: {}", suggestion));
                                        if !can_retry {
                                            attempt = MAX_RETRIES; // Skip further retries
                                        }
                                        break;
                                    }
                                }

                                if attempt == MAX_RETRIES - 1 {
                                    script_result.success = false;
                                } else {
                                    continue; // Try next attempt
                                }
                            } else {
                                script_result.success = true;
                                let description_str = block.attributes.get("description")
                                    .map_or("Executing command".to_string(), |s| s.to_string());
                                self.update_progress(&mut progress, &description_str, true);
                                break; // Command succeeded
                            }
                            
                            last_error = Some(format!("Command failed with exit code: {}", exit_code));
                            break;
                        }
                        Err(e) => {
                            last_error = Some(e.to_string());
                            if attempt == MAX_RETRIES - 1 {
                                script_result.success = false;
                                script_result.operations_performed.push(format!("Error: {}", e));
                            }
                        }
                    }
                    attempt += 1;
                }

                let output = script_result.output.clone();
                let status = if script_result.success { "SUCCESS" } else { "FAILED" };
                let mut error_context = String::new();

                if !script_result.success {
                    error_context.push_str("\nError Analysis:\n");
                    for op in &script_result.operations_performed {
                        if op.starts_with("Error") {
                            error_context.push_str(&format!("- {}\n", op));
                        }
                    }
                }

                if attempt > 1 {
                    error_context.push_str(&format!("\nRetry Information:\n- Attempted {} times\n", attempt));
                    if let Some(error) = &last_error {
                        error_context.push_str(&format!("- Final error: {}\n", error));
                    }
                }

                self.add_system_message(
                    "execute command",
                    status,
                    &format!(
                        "Command: {}\nExit Code: {}\nOutput:\n```\n{}\n```{}",
                        command,
                        script_result.exit_code.unwrap_or(-1),
                        output,
                        error_context
                    )
                )?;

                if script_result.success {
                    let retry_info = if attempt > 1 {
                        format!(" (after {} retries)", attempt - 1)
                    } else {
                        String::new()
                    };
                    script_result.operations_performed.push(
                        format!("Command succeeded{}", retry_info)
                    );
                    execution_outputs.push(format!(
                        "✓ Command executed successfully{}\nOutput:\n{}",
                        retry_info,
                        output
                    ));
                } else {
                    execution_outputs.push(format!(
                        "✖ Command failed after {} attempt(s)\nOutput:\n{}\nError Context:{}",
                        attempt,
                        output,
                        error_context
                    ));
                }
                
            }
            
            // Handle return value
            if let Some(return_template) = block.attributes.get("return") {
                let path_str = script_file_path.as_ref().map(|p| p.to_string_lossy().into_owned());
                let template = return_template.to_string();
                let return_value = self.substitute_variables(
                    template,
                    path_str,
                    &current_content
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