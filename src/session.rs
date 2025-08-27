 
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
use crate::parser::{self, ToolCall, GetSpec, SetSpec, RunSpec};
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
            ToolCall::Get(g) => {
                write!(f, "get{} {}",
                    g.id.as_ref().map(|s| format!(" #{}", s)).unwrap_or_default(),
                    g.targets.join(", ")
                )
            }
            ToolCall::Set(s) => {
                let body_snip = if s.body.len() > 30 {
                    format!("{}...", s.body.replace('\n', " ")[..30].to_string())
                } else { s.body.replace('\n', " ") };
                write!(f, "set{} target=\"{}\" append={} confirm={} (content: \"{}\")",
                    s.id.as_deref().unwrap_or(""),
                    s.target, s.append, s.confirm, body_snip
                )
            }
            ToolCall::Run(r) => {
                if let Some(h) = &r.http {
                    write!(f, "run http {} {}", h.method, h.url)
                } else if r.sh_one_liner {
                    write!(f, "run sh timeout={:?}", r.timeout_secs)
                } else {
                    write!(f, "run lang={} timeout={:?}", r.lang.as_deref().unwrap_or("-"), r.timeout_secs)
                }
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

1. **Continue with tools**: Produce fenced code blocks with the next minimal set of tool calls needed to progress toward task completion, or

2. **Task complete**: If the task is fully complete, reply with brief confirmation and NO code blocks.

## Use UCM Protocol
Use `get`, `set`, `run` blocks as shown in the system prompt. Do NOT use the old `primeactions` format.

## Decision Criteria
- Use minimal tool calls needed for next step
- Prefer `run` when it reduces total steps
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

        let prompt = format!(
            r#"
# PRIME - Terminal-Only Assistant (UCM Protocol)

Emit **fenced code blocks** using the three verbs below. Do NOT use the old `primeactions`.

## VERBS

### 1) get
```get {{#opt_id cwd="{working_dir}"}}
file:Cargo.toml
dir:src/
glob:src/**/*.rs
http:https://example.com
mem:long
input:Your name?
confirm:Proceed?
````

### 2) set

```set {{ target="file:README.md" }}
Hello
```

```set {{ target="mem:short" }}
Next step: parse config
```

### 3) run

```run {{ sh=true }}
git status -sb
```

```run {{ lang=python timeout=30 }}
print("ok")
```

```run {{ mode=http method=GET url="https://api.github.com/user" }}
Accept: application/json
```

## Rules

* Use the **fewest** blocks required; continue planning → acting until done.
* Read before write. Confirm destructive ops with `set {{target="rm:…", confirm=true}}`.
* If done, reply in natural language **without** any code block.

OS: {operating_system}
PWD: {working_dir}
Interpreters: {interpreters}

{memory}
"#,
            operating_system = operating_system,
            working_dir = working_dir,
            interpreters = has.iter().cloned().collect::<Vec<_>>().join(", "),
            memory = memory
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
            ToolCall::Get(g) => {
                use std::path::Path;
                use crate::commands::CommandProcessor;
                let cp = &self.command_processor;
                let mut parts = Vec::new();

                for tgt in g.targets {
                    if let Some(rest) = tgt.strip_prefix("file:") {
                        let path = rest.trim();
                        let lines = g.lines;
                        match cp.read_file_to_string_with_limit(Path::new(path), lines) {
                            Ok((content, truncated)) => {
                                let mut s = format!("== file:{} ==\n{}", path, content);
                                if truncated { s.push_str("\n… (file content truncated)"); }
                                parts.push(s);
                            }
                            Err(e) => parts.push(format!("== file:{} ==\n[ERROR] {}", path, e)),
                        }
                    } else if let Some(rest) = tgt.strip_prefix("dir:") {
                        let path = rest.trim();
                        match cp.list_directory_smart(Path::new(path)) {
                            Ok(items) => parts.push(format!("== dir:{} ==\n{}", path, items.join("\n"))),
                            Err(e) => parts.push(format!("== dir:{} ==\n[ERROR] {}", path, e)),
                        }
                    } else if let Some(pat) = tgt.strip_prefix("glob:") {
                        let mut acc = Vec::new();
                        match glob::glob(pat) {
                            Ok(paths) => {
                                for entry in paths {
                                    match entry {
                                        Ok(p) => acc.push(p.display().to_string()),
                                        Err(e) => acc.push(format!("[glob error] {}", e)),
                                    }
                                }
                            }
                            Err(e) => acc.push(format!("[glob pattern error] {}", e)),
                        }
                        parts.push(format!("== glob:{} ==\n{}", pat, if acc.is_empty() { "(no matches)".into() } else { acc.join("\n") }));
                    } else if let Some(url) = tgt.strip_prefix("http:").or_else(|| tgt.strip_prefix("https:")) {
                        let full = if tgt.starts_with("http:") { format!("http:{}", url) } else { format!("https:{}", url) };
                        let client = reqwest::Client::new();
                        let limit = g.limit_bytes.unwrap_or(1024 * 1024);
                        match client.get(&full).send().await {
                            Ok(resp) => {
                                let status = resp.status();
                                let bytes = match resp.bytes().await {
                                    Ok(b) => {
                                        let mut v = b.to_vec();
                                        if v.len() > limit { v.truncate(limit); }
                                        v
                                    }
                                    Err(e) => return (false, format!("GET {} error: {}", full, e)),
                                };
                                let text = String::from_utf8_lossy(&bytes).to_string();
                                let mut out = format!("== http:{} ==\nSTATUS {}\n", full, status);
                                out.push_str(&crate::ui::preview(&text, 80, 4000));
                                parts.push(out);
                            }
                            Err(e) => parts.push(format!("== http:{} ==\n[ERROR] {}", full, e)),
                        }
                    } else if tgt == "mem:long" {
                        match self.read_memory(Some("long_term")) {
                            Ok(s) => parts.push(format!("== mem:long ==\n{}", s)),
                            Err(e) => parts.push(format!("== mem:long ==\n[ERROR] {}", e)),
                        }
                    } else if tgt == "mem:short" {
                        match self.read_memory(Some("short_term")) {
                            Ok(s) => parts.push(format!("== mem:short ==\n{}", s)),
                            Err(e) => parts.push(format!("== mem:short ==\n[ERROR] {}", e)),
                        }
                    } else if let Some(prompt) = tgt.strip_prefix("input:") {
                        print!("{} ", prompt.trim());
                        use std::io::{self, Write};
                        let _ = io::stdout().flush();
                        let mut line = String::new();
                        match std::io::stdin().read_line(&mut line) {
                            Ok(_) => parts.push(format!("== input ==\n{}", line.trim_end())),
                            Err(e) => parts.push(format!("== input ==\n[ERROR] {}", e)),
                        }
                    } else if let Some(q) = tgt.strip_prefix("confirm:") {
                        print!("{} [y/N]: ", q.trim());
                        use std::io::{self, Write};
                        let _ = io::stdout().flush();
                        let mut line = String::new();
                        match std::io::stdin().read_line(&mut line) {
                            Ok(_) => {
                                let yes = line.trim().eq_ignore_ascii_case("y");
                                parts.push(format!("== confirm ==\n{}", if yes { "yes" } else { "no" }));
                            }
                            Err(e) => parts.push(format!("== confirm ==\n[ERROR] {}", e)),
                        }
                    } else {
                        parts.push(format!("[WARN] unknown get target: {}", tgt));
                    }
                }
                (true, parts.join("\n\n"))
            }

            ToolCall::Set(s) => {
                use std::path::Path;
                if s.target.starts_with("file:") {
                    let path = s.target.trim_start_matches("file:");
                    if s.target.starts_with("rm:") || s.target.starts_with("mkdir:") {
                        // (fall-through cases handled below)
                    }
                    match self.command_processor.write_file_to_path(Path::new(path), &s.body, s.append) {
                        Ok(()) => (true, format!("wrote {} (append={})", path, s.append)),
                        Err(e) => (false, format!("write {} failed: {}", path, e)),
                    }
                } else if s.target.starts_with("mkdir:") {
                    let path = s.target.trim_start_matches("mkdir:");
                    match std::fs::create_dir_all(path) {
                        Ok(()) => (true, format!("mkdir -p {}", path)),
                        Err(e) => (false, format!("mkdir {} failed: {}", path, e)),
                    }
                } else if s.target.starts_with("rm:") {
                    if !s.confirm {
                        return (false, "refused rm without confirm=true".into());
                    }
                    let path = s.target.trim_start_matches("rm:");
                    match std::fs::remove_file(path).or_else(|_| std::fs::remove_dir_all(path)) {
                        Ok(()) => (true, format!("removed {}", path)),
                        Err(e) => (false, format!("remove {} failed: {}", path, e)),
                    }
                } else if s.target.starts_with("mem:") {
                    let which = if s.target.ends_with("long") { "long_term" } else { "short_term" };
                    match self.write_memory(which, &s.body) {
                        Ok(()) => (true, format!("memory {} updated", which)),
                        Err(e) => (false, format!("memory write failed: {}", e)),
                    }
                } else {
                    (false, format!("unsupported set target: {}", s.target))
                }
            }

            ToolCall::Run(r) => {
                // HTTP mode
                if let Some(h) = r.http {
                    let client = reqwest::Client::new();
                    let mut req = match h.method.to_uppercase().as_str() {
                        "GET" => client.get(&h.url),
                        "POST" => client.post(&h.url),
                        "PUT" => client.put(&h.url),
                        "PATCH" => client.patch(&h.url),
                        "DELETE" => client.delete(&h.url),
                        _ => return (false, format!("unsupported HTTP method {}", h.method)),
                    };
                    for (k, v) in h.headers {
                        req = req.header(k, v);
                    }
                    if let Some(b) = h.body.clone() {
                        req = req.body(b);
                    }
                    match req.send().await {
                        Ok(resp) => {
                            let status = resp.status();
                            let text = resp.text().await.unwrap_or_default();
                            (status.is_success(), format!("STATUS {}\n{}", status, crate::ui::preview(&text, 80, 8000)))
                        }
                        Err(e) => (false, format!("http error: {}", e)),
                    }
                // SHELL one-liner
                } else if r.sh_one_liner {
                    let cmd = r.code.unwrap_or_default();
                    match self.command_processor.execute_command_async(&cmd, None).await {
                        Ok((0, out)) => (true, out),
                        Ok((code, out)) => (false, format!("exit code {}\n{}", code, out)),
                        Err(e) => (false, format!("exec error: {}", e)),
                    }
                // SCRIPT
                } else {
                    let lang = r.lang.clone().unwrap_or_default();
                    let code = r.code.unwrap_or_default();
                    // Reuse current run_script implementation by piping through a temp file
                    use std::{fs, time::Duration};
                    use tempfile::Builder;

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
                        return (false, format!("unsupported lang {}", lang));
                    };

                    let tmp = match Builder::new().prefix("prime_ucm_").suffix(&format!(".{}", ext)).tempfile() {
                        Ok(f) => f,
                        Err(e) => return (false, format!("tempfile error: {}", e)),
                    };
                    if let Err(e) = fs::write(tmp.path(), &code) {
                        return (false, format!("write temp script failed: {}", e));
                    }

                    let mut cmd = tokio::process::Command::new(interp);
                    cmd.arg(tmp.path());
                    if let Some(a) = &r.args { cmd.arg(a); }
                    cmd.stdout(std::process::Stdio::piped())
                       .stderr(std::process::Stdio::piped());

                    let duration = std::time::Duration::from_secs(r.timeout_secs.unwrap_or(60));
                    match tokio::time::timeout(duration, async { cmd.output().await }).await {
                        Err(_) => (false, format!("script timed out after {:?}", duration)),
                        Ok(Ok(out)) => {
                            let code = out.status.code().unwrap_or(-1);
                            let mut merged = out.stdout;
                            if !out.stderr.is_empty() {
                                merged.extend_from_slice(b"\n\nSTDERR:\n");
                                merged.extend_from_slice(&out.stderr);
                            }
                            let text = String::from_utf8_lossy(&merged).to_string();
                            if code == 0 { (true, text) } else { (false, format!("exit code {}\n{}", code, text)) }
                        }
                        Ok(Err(e)) => (false, format!("spawn failed: {}", e)),
                    }
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
 