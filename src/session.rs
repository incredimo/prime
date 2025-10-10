 
 
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use crossterm::style::Stylize;
use indicatif::{ProgressBar, ProgressStyle};
use llm::chat::{ChatMessage, ChatMessageBuilder, ChatProvider, ChatRole};
use textwrap::{wrap, Options};
use crate::commands::CommandProcessor;
use crate::memory::MemoryManager;
use crate::parser::{self, ToolCall};
use glob::glob;

const SPINNER_TICKS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn wrap_text(text: &str, width: usize) -> String {
    wrap(text, Options::new(width).break_words(false)).join("\n")
}

#[derive(Debug)]
pub struct ToolExecutionResult {
    pub tool_call_str: String,
    pub success: bool,
    pub output: String,
}

#[derive(Debug)]
pub struct DiscoveredTool {
    pub name: String,
    pub desc: String,
    pub args: String,
    pub path: PathBuf,
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
            ToolCall::ScriptTool { name, args } => write!(f, "{}: {}", name, args.join(" ")),
            ToolCall::CreateTool { name, desc, args, script_content } => {
                let content_snip = if script_content.len() > 30 {
                    format!("{}...", &script_content[..30].replace('\n', " "))
                } else {
                    script_content.replace('\n', " ")
                };
                write!(f, "create_tool: name={} desc=\"{}\" args=\"{}\" (content: \"{}\")", name, desc, args, content_snip)
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
    pub working_dir: PathBuf,
    pub discovered_tools: Vec<DiscoveredTool>,
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
        let discovered_tools = Self::discover_tools(&working_dir)?;
        Ok(Self {
            base_dir,
            session_id,
            session_log_path,
            llm,
            command_processor: CommandProcessor::new(),
            memory_manager,
            working_dir,
            discovered_tools,
        })
    }

    fn discover_tools(workspace: &Path) -> Result<Vec<DiscoveredTool>> {
        let prime_dir = workspace.join("prime");
        if !prime_dir.exists() {
            fs::create_dir_all(&prime_dir)
                .with_context(|| format!("Failed to create ./prime directory: {}", prime_dir.display()))?;
            return Ok(Vec::new());
        }
        #[cfg(target_os = "windows")]
        let glob_pat = prime_dir.join("tool_*.ps1");
        #[cfg(not(target_os = "windows"))]
        let glob_pat = prime_dir.join("tool_*.sh");
        let mut tools = Vec::new();
        if let Ok(entries) = glob(glob_pat.to_str().ok_or_else(|| anyhow!("Invalid glob pattern"))?) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry;
                if let Some(file_name_os) = path.file_name() {
                    let file_name = file_name_os.to_string_lossy();
                    let name_stem = file_name.strip_suffix(if cfg!(target_os = "windows") { ".ps1" } else { ".sh" })
                        .and_then(|s| s.strip_prefix("tool_"))
                        .unwrap_or(&file_name)
                        .to_string();
                    let content = fs::read_to_string(&path)
                        .with_context(|| format!("Failed to read script: {}", path.display()))?;
                    let lines: Vec<&str> = content.lines().collect();
                    let mut header_found = None;
                    for line in lines {
                        let trimmed = line.trim();
                        if trimmed.starts_with("## TOOL:") {
                            header_found = Some(trimmed[9..].trim().to_string());
                            break;
                        }
                    }
                    if let Some(header_str) = header_found {
                        let (parsed_name, parsed_desc, parsed_args) = Self::parse_tool_header(&header_str)?;
                        if parsed_name == name_stem {
                            tools.push(DiscoveredTool { name: parsed_name, desc: parsed_desc, args: parsed_args, path });
                        }
                    }
                }
            }
        }
        Ok(tools)
    }

    fn parse_tool_header(header: &str) -> Result<(String, String, String)> {
        let mut chars = header.chars().peekable();
        let mut name = String::new();
        let mut desc = String::new();
        let mut args_spec = String::new();
        let mut current_key = String::new();
        loop {
            while chars.peek().map_or(false, |&ch| ch.is_ascii_whitespace()) {
                chars.next();
            }
            if chars.peek().is_none() {
                break;
            }
            current_key.clear();
            if let Some(ch) = chars.next() {
                current_key.push(ch);
            }
            while chars.peek().map_or(false, |&ch| ch != '=') {
                if let Some(ch) = chars.next() {
                    current_key.push(ch);
                }
            }
            if chars.peek().map_or(true, |&ch| ch != '=') {
                continue;
            }
            chars.next();
            while chars.peek().map_or(false, |&ch| ch.is_ascii_whitespace()) {
                chars.next();
            }
            if chars.peek().map_or(true, |&ch| ch != '"') {
                continue;
            }
            chars.next();
            let mut value = String::new();
            while let Some(ch) = chars.next() {
                if ch == '"' {
                    break;
                }
                value.push(ch);
            }
            match current_key.trim() {
                "name" => name = value,
                "desc" => desc = value,
                "args" => args_spec = value,
                _ => {}
            }
        }
        if name.is_empty() || desc.is_empty() || args_spec.is_empty() {
            return Err(anyhow!("Invalid tool header: missing name, desc, or args"));
        }
        Ok((name, desc, args_spec))
    }

    pub fn reload_tools(&mut self) -> Result<()> {
        self.discovered_tools = Self::discover_tools(&self.working_dir)?;
        Ok(())
    }

    pub fn is_tool_destructive(&self, tool_call: &ToolCall) -> bool {
        match tool_call {
            ToolCall::Shell { command } => {
                self.command_processor.is_command_destructive(command)
            }
            ToolCall::ScriptTool { name, args } => {
                let ext = if cfg!(target_os = "windows") { "ps1" } else { "sh" };
                let mut full_cmd = format!("./prime/tool_{}.{}", name, ext);
                if !args.is_empty() {
                    full_cmd.push_str(&format!(" {}", args.join(" ")));
                }
                self.command_processor.is_command_destructive(&full_cmd)
            }
            ToolCall::CreateTool { .. } => false,
            _ => false,
        }
    }

    pub async fn process_input(&mut self, input: &str) -> Result<()> {
        self.save_log("User Input", input)?;
        self.reload_tools()?;
        const MAX_CONSECUTIVE_TOOL_TURNS: usize = 10;
        let mut tool_turn_count = 0;
        let mut has_displayed_actions = false;
        loop {
            if tool_turn_count >= MAX_CONSECUTIVE_TOOL_TURNS {
                println!("{}", "Reached maximum tool execution turns. The session might be in a loop. Please try a new prompt.".red());
                break;
            }
            let response_text = self.generate_prime_response().await?;
            let parsed = parser::parse_llm_response(&response_text)?;
            if parsed.tool_calls.is_empty() {
                if !parsed.natural_language.is_empty() {
                    if has_displayed_actions {
                        println!();
                        let wrapped = wrap_text(&parsed.natural_language, 68);
                        for line in wrapped.lines() {
                            println!("{}", format!("┃{}", line).white());
                        }
                        println!("{}", "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".white());
                    } else {
                        let wrapped = wrap_text(&parsed.natural_language, 70);
                        for line in wrapped.lines() {
                            println!("{}", line.white());
                        }
                    }
                }
                break;
            }
            tool_turn_count += 1;
            if !parsed.natural_language.is_empty() {
                let wrapped = wrap_text(&parsed.natural_language, 70);
                for line in wrapped.lines() {
                    println!("{}", line.white());
                }
                io::stdout().flush()?;
            }
            println!();
            println!("{}", "┏━ actions".yellow());
            for tool in &parsed.tool_calls {
                match tool {
                    ToolCall::Shell { command } => println!("{}", format!("┃ {}", command).yellow()),
                    ToolCall::ReadFile { path, lines } => {
                        if let Some((start, end)) = lines {
                            println!("{}", format!("┃ read_file: {} lines={}-{}", path, start, end).yellow());
                        } else {
                            println!("{}", format!("┃ read_file: {}", path).yellow());
                        }
                    }
                    ToolCall::WriteFile { path, .. } => println!("{}", format!("┃ write_file: {}", path).yellow()),
                    ToolCall::ListDir { path } => println!("{}", format!("┃ list_dir: {}", path).yellow()),
                    ToolCall::ChangeDir { path } => println!("{}", format!("┃ cd: {}", path).yellow()),
                    ToolCall::WriteMemory { memory_type, .. } => println!("{}", format!("┃ write_memory: {}", memory_type).yellow()),
                    ToolCall::ClearMemory { memory_type } => println!("{}", format!("┃ clear_memory: {}", memory_type).yellow()),
                    ToolCall::ScriptTool { name, args } => println!("{}", format!("┃ {}: {}", name, args.join(" ")).yellow()),
                    ToolCall::CreateTool { name, desc, args, .. } => println!("{}", format!("┃ create_tool: name={} desc=\"{}\" args=\"{}\"", name, desc, args).yellow()),
                }
            }
            let is_destructive = parsed.tool_calls.iter().any(|tc| self.is_tool_destructive(tc));
            let should_execute = if is_destructive {
                println!("{}", "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ destructive ━━━━━".red());
                print!("{}", "Execute? (y/N): ".red());
                io::stdout().flush().context("Failed to flush stdout")?;
                let mut confirmation = String::new();
                io::stdin().read_line(&mut confirmation).context("Failed to read user input")?;
                confirmation.trim().eq_ignore_ascii_case("y")
            } else {
                println!("{}", "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ executing in 2s ━━━━━".yellow());
                std::thread::sleep(std::time::Duration::from_secs(2));
                true
            };
            if !should_execute {
                println!();
                println!("{}", "┃ Plan cancelled by user.".red());
                println!("{}", "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".red());
                self.save_log("System", "Plan cancelled by user.")?;
                break;
            }
            has_displayed_actions = true;
            match self.execute_actions(parsed.tool_calls).await {
                Ok(successful_results) => {
                    let results_prompt = self.format_tool_results_for_llm(&successful_results)?;
                    self.save_log("Tool Results", &results_prompt)?;
                }
                Err(failed_result) => {
                    let error_prompt = self.format_tool_failure_for_llm(&failed_result)?;
                    println!();
                    println!("{}", "┃ A tool failed. The AI will attempt to self-correct.".red());
                    println!("{}", "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".red());
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
4. You can extend yourself by creating new tools dynamically using the create_tool tool. This allows you to build custom capabilities on the fly without user intervention.
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
- If a task requires a new capability, use create_tool to extend yourself immediately—it will be auto-discovered for future use.
TASK COMPLETION:
- Focus on exactly what the user requested.
- If a tool fails, DO NOT RE-TRY THE EXACT SAME COMMAND. Analyze the error message and change your approach, or create a new tool if needed.
- Verify task completion before responding with a final message.

SELF-EXTENSION EXAMPLE:
To handle a unique task like "analyze PDF metadata", you can create a tool:
```primeactions
create_tool: name=pdf_analyze desc="Extract metadata from PDF files" args="file_path"
#!/bin/bash
## TOOL: name=pdf_analyze desc="Extract metadata from PDF files" args="file_path"
file_path="$1"
exiftool "$file_path" || echo "Error extracting metadata."
EOF_PRIME
```
For PowerShell (if on Windows):
```primeactions
create_tool: name=pdf_analyze desc="Extract metadata from PDF files" args="file_path"
param([string]$file_path)
## TOOL: name=pdf_analyze desc="Extract metadata from PDF files" args="file_path"
Get-ChildItem $file_path | ForEach-Object { $_.VersionInfo } || Write-Output "Error extracting metadata."
EOF_PRIME
```
After creation, reload happens automatically, and you can use `pdf_analyze: some.pdf` in the next turn.

Another example - web scraper stub (extend with curl/wget):
```primeactions
create_tool: name=fetch_url desc="Fetch content from a URL" args="url"
#!/bin/bash
## TOOL: name=fetch_url desc="Fetch content from a URL" args="url"
url="$1"
curl -s "$url" || echo "Failed to fetch URL."
EOF_PRIME
```
PowerShell:
```primeactions
create_tool: name=fetch_url desc="Fetch content from a URL" args="url"
param([string]$url)
## TOOL: name=fetch_url desc="Fetch content from a URL" args="url"
Invoke-WebRequest -Uri $url -UseBasicParsing | Select-Object -ExpandProperty Content || Write-Output "Failed to fetch URL."
EOF_PRIME
```
Use create_tool proactively to build specialized tools for recurring or complex tasks.
"#;
        let mut tools_section = String::new();
        tools_section.push_str(r#"
**AVAILABLE TOOLS**
1. `shell: <command>`
    - Executes a shell command in the current working directory.
    - Example: `shell: ls -l`
2. `cd: <path>`
    - Changes the current working directory. The new directory persists for all future commands.
    - Example: `cd: src/`
3. `read_file: <path> [lines=start-end]`
    - Reads a file. Optionally, you can specify a line range.
    - Example: `read_file: src/main.rs lines=1-20`
4. `write_file: <path> [append=true]`
    - Writes content to a file. Overwrites by default. Use `append=true` to append.
    - The content to write must follow on new lines, terminated by `EOF_PRIME`.
    - Example:
      ```primeactions
      write_file: new_file.txt
      Hello, world!
      EOF_PRIME
      ```
5. `list_dir: <path>`
    - Lists the contents of a directory.
    - Example: `list_dir: .`
6. `write_memory: <long_term|short_term>`
    - Writes content to your memory for context.
    - Content follows on new lines, terminated by `EOF_PRIME`.
    - Example:
      ```primeactions
      write_memory: short_term
      The user wants to refactor the `console.rs` file.
      EOF_PRIME
      ```
7. `clear_memory: <long_term|short_term>`
    - Clears one of your memories.
    - Example: `clear_memory: short_term`
8. `create_tool: name=<name> desc="<description>" args="<arg1 arg2 ...>"`
    - Creates a new custom tool script in ./prime/tool_<name>.{sh|ps1} (OS-appropriate).
    - The script content follows on new lines, terminated by `EOF_PRIME`. Include the required header in the content.
    - After creation, it is immediately available for use in subsequent turns.
    - Example (Bash):
      ```primeactions
      create_tool: name=grep_files desc="Search files for pattern in path" args="pattern path"
      #!/bin/bash
      ## TOOL: name=grep_files desc="Search files for pattern in path" args="pattern path"
      pattern="$1"
      path="${2:-.}"
      grep -r --color=never "$pattern" "$path" 2>/dev/null || echo "No matches."
      EOF_PRIME
      ```
    - PowerShell example:
      ```primeactions
      create_tool: name=grep_files desc="Search files for pattern in path" args="pattern path"
      param([string]$pattern, [string]$path = ".")
      ## TOOL: name=grep_files desc="Search files for pattern in path" args="pattern path"
      Get-ChildItem -Path $path -Recurse -ErrorAction SilentlyContinue | Select-String -Pattern $pattern | ForEach-Object { $_.Line }
      EOF_PRIME
      ```
"#);
        for (i, tool) in self.discovered_tools.iter().enumerate() {
            let num = 9 + i;
            let arg_example = if !tool.args.is_empty() {
                let arg_parts: Vec<&str> = tool.args.split_whitespace().collect();
                if arg_parts.len() >= 2 {
                    format!(" (e.g., {}: {} {})", tool.name, arg_parts[0], arg_parts[1])
                } else if !arg_parts.is_empty() {
                    format!(" (e.g., {}: {})", tool.name, arg_parts[0])
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            tools_section.push_str(&format!("\n{}. `{}` - {}{}", num, tool.name, tool.desc, arg_example));
        }
        if !self.discovered_tools.is_empty() {
            tools_section.push_str("\nFor custom tools, use `tool_name: arg1 arg2` (space-separated).");
        }
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
{tools_section}
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
"#,
            tools_section = tools_section,
            operating_system = operating_system,
            working_dir = working_dir,
            memory = memory,
            behavioral_prompt = behavioral_prompt,
        );
        Ok(technical_prompt)
    }

    pub async fn execute_actions(
        &mut self,
        tool_calls: Vec<ToolCall>,
    ) -> Result<Vec<ToolExecutionResult>, ToolExecutionResult> {
        let start_time = std::time::Instant::now();
        let mut all_results = Vec::new();
        for tool_call in tool_calls.into_iter() {
            let result = self.execute_tool(tool_call).await;
            if !result.success {
                return Err(result);
            }
            all_results.push(result);
        }
        let duration = start_time.elapsed();
        let duration_str = format!("{:.1}s", duration.as_secs_f32());
        println!("{}", format!("╰────────────────────────────────────── completed in {} ────────", duration_str).green());
        Ok(all_results)
    }

    async fn execute_tool(&mut self, tool_call: ToolCall) -> ToolExecutionResult {
        let tool_call_str = tool_call.to_string();
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
            ToolCall::WriteFile { path, content, append } => {
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
            ToolCall::WriteMemory { memory_type, content } => match self.write_memory(&memory_type, &content) {
                Ok(()) => (true, format!("Successfully wrote to {} memory", memory_type)),
                Err(e) => (false, format!("Failed to write to {} memory: {}", memory_type, e)),
            },
            ToolCall::ClearMemory { memory_type } => match self.clear_memory(&memory_type) {
                Ok(()) => (true, format!("Successfully cleared {} memory", memory_type)),
                Err(e) => (false, format!("Failed to clear {} memory: {}", memory_type, e)),
            },
            ToolCall::ScriptTool { name, args } => {
                let ext = if cfg!(target_os = "windows") { "ps1" } else { "sh" };
                let script_path = self.working_dir.join("prime").join(format!("tool_{}.{}", name, ext));
                if !script_path.exists() {
                    (false, format!("Script not found: {}", script_path.display()))
                } else {
                    let mut cmd = format!("{}", script_path.display());
                    if !args.is_empty() {
                        cmd.push_str(&format!(" {}", args.join(" ")));
                    }
                    match self.command_processor.execute_command(&cmd, Some(&self.working_dir)) {
                        Ok((0, out)) => (true, out),
                        Ok((code, out)) => (false, format!("Script failed with exit code {}\nOutput:\n{}", code, out)),
                        Err(e) => (false, format!("Failed to execute script: {}", e)),
                    }
                }
            }
            ToolCall::CreateTool { name, desc, args, script_content } => {
                let ext = if cfg!(target_os = "windows") { "ps1" } else { "sh" };
                let tool_path = self.working_dir.join("prime").join(format!("tool_{}.{}", name, ext));
                let arg_parts: Vec<&str> = args.split_whitespace().collect();
                let params_str = if cfg!(target_os = "windows") {
                    if arg_parts.is_empty() {
                        String::new()
                    } else {
                        format!("param({})", arg_parts.iter().map(|a| format!("[string]${}", a)).collect::<Vec<_>>().join(", "))
                    }
                } else {
                    String::new()
                };
                let shebang = if cfg!(target_os = "windows") {
                    format!("{}\n", params_str)
                } else {
                    "#!/bin/bash\n".to_string()
                };
                let header = format!("## TOOL: name={} desc=\"{}\" args=\"{}\"\n", name, desc, args);
                let full_content = format!("{}{}{}", shebang, header, script_content);
                match self.command_processor.write_file_to_path(&tool_path, &full_content, false) {
                    Ok(()) => {
                        #[cfg(unix)]
                        {
                            if let Err(e) = fs::set_permissions(&tool_path, fs::Permissions::from_mode(0o755)) {
                                eprintln!("Warning: Failed to set executable bit: {}", e);
                            }
                        }
                        self.reload_tools().ok();
                        (true, format!("Created and loaded new tool: {} at {}", name, tool_path.display()))
                    }
                    Err(e) => (false, format!("Failed to create tool '{}': {}", tool_path.display(), e)),
                }
            }
        };
        if !output.trim().is_empty() {
            for line in output.trim().lines() {
                println!("{}", format!("│ {}", line).dim());
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

    pub fn list_tools(&self) -> String {
        let mut out = "Built-in Tools:\n".to_string();
        out.push_str("- shell: Execute any shell command\n");
        out.push_str("- cd: Change working directory\n");
        out.push_str("- read_file: Read file content (with optional line range)\n");
        out.push_str("- write_file: Write to file (with optional append)\n");
        out.push_str("- list_dir: List directory contents\n");
        out.push_str("- write_memory: Add to long/short-term memory\n");
        out.push_str("- clear_memory: Clear memory type\n");
        out.push_str("- create_tool: Create a new self-extending tool script\n");
        out.push_str("\nDiscovered Custom Tools (./prime/):\n");
        if self.discovered_tools.is_empty() {
            out.push_str("None found. Use create_tool to build your own!\n");
        } else {
            for tool in &self.discovered_tools {
                out.push_str(&format!("- {}: {} (args: {}, path: {})\n", tool.name, tool.desc, tool.args, tool.path.display()));
            }
        }
        out
    }
}
 