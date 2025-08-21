 
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
use crate::ui;

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
            ToolCall::WriteFile { path, content, append } => {
                let content_snip = if content.len() > 30 {
                    format!("{}...", &content[..30].replace('\n', " "))
                } else {
                    content.replace('\n', " ")
                };
                write!(f, "write_file: {} append={} (content: \"{}\")", path, append, content_snip)
            }
            ToolCall::ListDir { path } => write!(f, "list_dir: {}", path),
            ToolCall::WriteMemory { memory_type, content } => {
                let content_snip = if content.len() > 30 {
                    format!("{}...", &content[..30].replace('\n', " "))
                } else {
                    content.replace('\n', " ")
                };
                write!(f, "write_memory: {} (content: \"{}\")", memory_type, content_snip)
            }
            ToolCall::ClearMemory { memory_type } => write!(f, "clear_memory: {}", memory_type),
            ToolCall::RunScript { lang, args, code, timeout_secs } => {
                let code_snip = if code.len() > 30 {
                    format!("{}...", &code[..30].replace('\n', " "))
                } else {
                    code.replace('\n', " ")
                };
                write!(f, "run_script: lang={} timeout={:?} args={:?} (code: \"{}\")", 
                       lang, timeout_secs, args, code_snip)
            }
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

    /// NOTE: We now take both `display_input` (shown to the user) and `llm_input` (with @context).
    pub async fn process_input(&mut self, display_input: &str, llm_input: &str) -> Result<()> {
        // Save the full input (with context) for reproducibility/logging.
        self.save_log("User Input", llm_input)?;

        // USER section shows only what user typed, no context bundle.
        println!("{}", ui::panel("USER", display_input));

        let mut turn_count = 0;
        let max_turns: usize = std::env::var("LLM_MAX_TURNS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(20);

        use std::collections::VecDeque;
        const DUP_WINDOW: usize = 3;
        let mut recent_action_signatures: VecDeque<String> = VecDeque::new();

        // Seed the conversation with the provided llm_input.
        self.append_user_message(llm_input)?;

        loop {
            if turn_count >= max_turns {
                println!("{}", ui::panel("WARN", &ui::warn_tag("Turn limit reached; stopping.")));
                break;
            }
            turn_count += 1;

            let response_text = self.generate_prime_response().await?;
            let parsed = parser::parse_llm_response(&response_text)?;

            // If no more actions, print final AI answer and exit.
            if parsed.tool_calls.is_empty() {
                if !parsed.natural_language.is_empty() {
                    println!("{}", ui::panel("AI", &parsed.natural_language));
                }
                break;
            }

            // Loop-safety: stop if the same set of actions repeats.
            let sig = parsed.tool_calls.iter().map(|t| t.to_string()).collect::<Vec<_>>().join("\n");
            recent_action_signatures.push_back(sig.clone());
            if recent_action_signatures.len() > DUP_WINDOW { recent_action_signatures.pop_front(); }
            if recent_action_signatures.len() == DUP_WINDOW && recent_action_signatures.iter().all(|s| *s == sig) {
                println!("{}", ui::panel("WARN", "Detected repeated identical actions. Stopping to avoid a loop."));
                break;
            }

            // Execute each tool with COMMAND / RESULT panels and minimal noise.
            let action_results = self.execute_actions(parsed.tool_calls).await?;
            let results_prompt = self.format_tool_results_for_llm(&action_results)?;
            self.save_log("Tool Results", &results_prompt)?;
            // Feed results back to the model for the next step.
            self.append_user_message(&results_prompt)?;
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

    fn append_user_message(&self, content: &str) -> Result<()> {
        // Append to the session log in the same format the LLM sees.
        self.save_log("User Input (appended)", content)
    }

    async fn generate_prime_response(&mut self) -> Result<String> {
        let history_window: usize = std::env::var("LLM_HISTORY_WINDOW")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(50);
        let history = self.get_history(Some(history_window))?;
        let mut messages = vec![ChatMessage::user()
            .content(self.get_system_prompt()?)
            .build()];
        messages.extend(history);

        messages.push(
            ChatMessage::user()
                .content(r#"Given the latest <tool_output> blocks, either:

1. **Continue with tools**: Produce a ```primeactions block with the next minimal set of tool calls needed to progress toward task completion, or

2. **Task complete**: If the task is fully complete, reply with brief confirmation and NO primeactions block.

## Action Block Format
```
primeactions
tool_name: parameter1="value1" parameter2="value2"
tool_name: parameter1="value3"
```

## Decision Criteria
- Use minimal tool calls needed for next step
- Prefer `run_script` when it reduces total steps
- Read files before modifying them
- Verify command success before proceeding
- Stop when task objectives are met"#)
                .build(),
        );

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(ProgressStyle::with_template("{spinner:.blue.bold} {msg}").unwrap());
        spinner.set_message("planning…");
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));

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
        let working_dir = std::env::current_dir()?.display().to_string();

        // Environment probe (available interpreters)
        let has: std::collections::HashSet<&'static str> = ["python3","node","bash","pwsh","ruby","php"]
            .iter()
            .filter_map(|b| which::which(b).ok().map(|_| *b))
            .collect();

        let reg = crate::actions::ActionRegistry::default();
        let ctx = crate::actions::ActionContext {
            user_input: "", // optional, set by caller if you want per-turn gating by text
            cwd: &working_dir,
            has_interpreters: has.clone(),
        };
        let snippets = reg.select_for(&ctx).into_iter()
            .map(|a| a.prompt_snippet.as_str()).collect::<Vec<_>>().join("\n\n");

        let prompt = format!(r#"
# PRIME - Terminal-Only Assistant

You are PRIME, a **terminal-only coding assistant** that **plans → acts → observes → repeats**.

## CORE WORKFLOW
1. **Plan**: Analyze the task and determine minimal tool calls needed
2. **Act**: Execute tools with precise, deterministic commands
3. **Observe**: Review results and plan next steps
4. **Repeat**: Continue until task completion

## TOOL USAGE RULES

### 1) Execution Strategy
- **Recurse until done**: Continue tool calls until task is complete
- **Emit no action block when complete**: Reply with natural language confirmation only
- **Minimal viable steps**: Use fewest tool calls possible
- **Deterministic commands**: All commands must be non-interactive and exit cleanly

### 2) File Operations
- **Read minimally**: Use `lines=(start,end)` for targeted reads
- **Write completely**: Include full file content, end with `EOF_PRIME` marker
- **Append carefully**: Only use `append=true` when explicitly adding to existing content
- **Verify paths**: Ensure file paths are relative to current working directory

### 3) Script Execution
- **Prefer run_script**: Use when it reduces total steps/tokens vs shell commands
- **Supported languages**: python, node, bash, powershell, ruby, php
- **Timeout safety**: Default 60s timeout, specify longer if needed
- **Clean execution**: Scripts should not require user interaction

### 4) Shell Commands
- **Non-interactive**: Commands must not prompt for input
- **Exit codes**: Success = 0, handle failures gracefully
- **Output capture**: Both stdout and stderr are captured
- **Working directory**: Commands run in current PWD

### 5) Memory Management
- **Context persistence**: Use memory for cross-session context
- **Type-specific**: Use appropriate memory types for different data
- **Clean up**: Clear memory when no longer needed

### 6) Safety & Scoping
- **Smallest possible scope**: Limit operations to necessary files/directories
- **No destructive actions**: Avoid rm -rf, overwrite protection
- **Verify before execute**: Read files before modifying
- **Error handling**: Graceful failure handling with informative messages

## RESPONSE FORMAT
- Use ```primeactions blocks for tool calls
- One tool call per block
- Natural language only when task is complete
- Clear, concise explanations

## ENVIRONMENT CONTEXT
- **OS**: {operating_system}
- **PWD**: {working_dir}
- **Available Interpreters**: {interpreters}

## PERSISTENT MEMORY
{memory}

## AVAILABLE TOOLS (this turn only)
{snippets}


"#,
        interpreters = has.iter().cloned().collect::<Vec<_>>().join(", ")
        );
        Ok(prompt)
    }

    pub async fn execute_actions(&self, tool_calls: Vec<ToolCall>) -> Result<Vec<ToolExecutionResult>> {
        let mut all_results = Vec::new();
        let mut failures = 0usize;

        for tool_call in tool_calls.into_iter() {
            // COMMAND panel
            println!("{}", ui::panel("COMMAND", &tool_call.to_string()));

            // Spinner while running
            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::with_template("{spinner:.green.bold} executing…").unwrap());
            pb.enable_steady_tick(std::time::Duration::from_millis(80));

            let start_time = std::time::Instant::now();
            let (success, output) = self.run_tool(tool_call).await;
            let elapsed = start_time.elapsed();

            pb.finish_and_clear();

            // RESULT panel
            let pv_ok = ui::preview(&output, 20, 4000);
            let pv_err = ui::preview(&output, 30, 6000);

            let body = if success {
                if pv_ok.is_empty() {
                    ui::ok_tag(&format!("completed ({:?})", elapsed))
                } else {
                    format!("{}\n{}", ui::ok_tag(&format!("completed ({:?})", elapsed)), pv_ok.dark_grey())
                }
            } else {
                failures += 1;
                if pv_err.is_empty() {
                    ui::err_tag(&format!("failed ({:?})", elapsed))
                } else {
                    format!("{}\n{}", ui::err_tag(&format!("failed ({:?})", elapsed)), pv_err.red())
                }
            };
            println!("{}", ui::panel("RESULT", &body));

            all_results.push(ToolExecutionResult {
                tool_call_str: String::new(),
                success,
                output,
            });
        }

        if failures > 0 {
            println!("{}", ui::panel("WARN", &format!("{} tool(s) failed", failures)));
        }

        Ok(all_results)
    }

    async fn run_tool(&self, tool_call: ToolCall) -> (bool, String) {
        match tool_call {
            ToolCall::Shell { command } => {
                match self.command_processor.execute_command_async(&command, None).await {
                    Ok((0, out)) => (true, out),
                    Ok((code, out)) => {
                        if code == -1 {
                            (false, out)
                        } else {
                            (false, format!("Command failed with exit code {}\nOutput:\n{}", code, out))
                        }
                    }
                    Err(e) => (false, format!("Failed to execute command: {}", e)),
                }
            }
            ToolCall::ReadFile { path, lines } => {
                match self.command_processor.read_file_to_string_with_limit(Path::new(&path), lines) {
                    Ok((content, truncated)) => {
                        let result = if truncated { format!("{}\nNote: File content was truncated", content) } else { content };
                        (true, result)
                    }
                    Err(e) => (false, format!("Failed to read file '{}': {}", path, e)),
                }
            }
            ToolCall::WriteFile { path, content, append } => {
                match self.command_processor.write_file_to_path(Path::new(&path), &content, append) {
                    Ok(()) => (true, format!("Successfully wrote to {}", path)),
                    Err(e) => (false, format!("Failed to write file '{}': {}", path, e)),
                }
            }
            ToolCall::ListDir { path } => {
                match self.command_processor.list_directory_smart(Path::new(&path)) {
                    Ok(items) => {
                        let body = if items.is_empty() { "Directory is empty".into() } else { items.join("\n") };
                        (true, body)
                    }
                    Err(e) => (false, format!("Failed to list directory '{}': {}", path, e)),
                }
            }
            ToolCall::WriteMemory { memory_type, content } => {
                match self.write_memory(&memory_type, &content) {
                    Ok(()) => (true, format!("Successfully wrote to {} memory", memory_type)),
                    Err(e) => (false, format!("Failed to write to {} memory: {}", memory_type, e)),
                }
            }
            ToolCall::ClearMemory { memory_type } => {
                match self.clear_memory(&memory_type) {
                    Ok(()) => (true, format!("Successfully cleared {} memory", memory_type)),
                    Err(e) => (false, format!("Failed to clear {} memory: {}", memory_type, e)),
                }
            }
            ToolCall::RunScript { lang, args, code, timeout_secs } => {
                use std::{env, fs, process::Command, time::Duration};
                use tempfile::Builder;

                // map language → (file extension, interpreter argv[0])
                fn map_lang(lang: &str) -> Option<(&'static str, &'static str)> {
                    match lang.to_ascii_lowercase().as_str() {
                        "python" | "py" => Some(("py", "python3")),
                        "node" | "javascript" | "js" => Some(("js", "node")),
                        "bash" | "sh" => Some(("sh", "bash")),
                        "powershell" | "pwsh" => Some(("ps1", "pwsh")),
                        "ruby" | "rb" => Some(("rb", "ruby")),
                        "php" => Some(("php", "php")),
                        _ => None,
                    }
                }

                let Some((ext, interp)) = map_lang(&lang) else {
                    return (false, format!("Unsupported lang '{}'", lang));
                };

                let tmp = match Builder::new().prefix("prime_script_").suffix(&format!(".{}", ext)).tempfile() {
                    Ok(f) => f,
                    Err(e) => return (false, format!("tempfile error: {}", e)),
                };
                if let Err(e) = fs::write(tmp.path(), &code) {
                    return (false, format!("write temp script failed: {}", e));
                }

                // build command line
                let mut cmd = Command::new(interp);
                cmd.arg(tmp.path());
                if let Some(a) = &args {
                    // naive split: let the shell-like args be passed as one token; advanced parsing can be added.
                    cmd.arg(a);
                }
                cmd.stdout(std::process::Stdio::piped())
                   .stderr(std::process::Stdio::piped());

                // spawn + timeout
                let duration = Duration::from_secs(timeout_secs.unwrap_or(60));
                match tokio::time::timeout(duration, async {
                    tokio::process::Command::from(cmd).output().await
                }).await {
                    Err(_) => (false, format!("script timed out after {:?}.", duration)),
                    Ok(Ok(out)) => {
                        let code = out.status.code().unwrap_or(-1);
                        let mut merged = out.stdout;
                        if !out.stderr.is_empty() {
                            merged.extend_from_slice(b"\n\nSTDERR:\n");
                            merged.extend_from_slice(&out.stderr);
                        }
                        let text = String::from_utf8_lossy(&merged).to_string();
                        if code == 0 { (true, text) }
                        else { (false, format!("exit code {}\n{}", code, text)) }
                    }
                    Ok(Err(e)) => (false, format!("spawn failed: {}", e)),
                }
            }
        }
    }

    pub fn format_tool_results_for_llm(&self, results: &[ToolExecutionResult]) -> Result<String> {
        let formatted_results = results
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                let status = if result.success { "SUCCESS" } else { "FAILURE" };
                format!(
                    "<tool_output id=\"{}\" status=\"{}\">\n{}\n</tool_output>",
                    idx,
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
 