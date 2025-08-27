use std::borrow::Cow;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::format;
use crossterm::style::Stylize;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::{MatchingBracketValidator, Validator};
use rustyline::{Context as RustylineContext, Editor, Helper};

use crate::session::PrimeSession;
/// Approx chars per token for budget estimate.
const CHARS_PER_TOKEN: usize = 4;
/// Portion of the model's max tokens we allocate to *injected* context.
const INJECT_TOKEN_BUDGET_FRACTION_NUM: usize = 3; // 3/4 of max tokens
const INJECT_TOKEN_BUDGET_FRACTION_DEN: usize = 4;
/// Per-file soft cap before final global trim (keeps I/O bounded).
const PER_FILE_MAX_BYTES: usize = 256 * 1024; // 256 KiB
/// Directory tree limits
const DIR_MAX_DEPTH: usize = 2;
const DIR_MAX_ENTRIES: usize = 200;
//
//█▀█ ▄▀█ █ █▀█▀▄ █▀▀      PRIME V0.1.15
//█▀▀ █▀▄ █ █ █ █ ██▄      google/gemini-2.5-flash
//━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//[gray dim]prime is running in [yellow bold]$pwd[/yellow bold]. you can use
//[yellow bold]:help[/yellow bold] to see available commands or [yellow bold]:exit[/yellow bold] to quit.
//──────────────────────────────────────────────────────────────────────

const BANNER: &str = r#"
 █▀█ ▄▀█ █ █▀█▀▄ █▀▀
 █▀▀ █▀▄ █ █ █ █ ██▄"#;

pub fn display_banner() {
    // Just print the logo; the right-side overlays (PRIME + provider/model)
    // are drawn by `display_init_info` so it can use the actual model/provider.
    println!("{}", BANNER.bold().white());
}

pub fn display_init_info(
    model: &str,
    provider: &str,
    _prime_config_base_dir: &PathBuf,
    _workspace_dir: &PathBuf,
) {
    // Right-side overlays next to the 2-line banner
    let version = env!("CARGO_PKG_VERSION");
    let overlay_col = 28; // looks good with current ASCII logo width

    // Example target per spec:
    // PRIME V0.1.15
    // google/gemini-2.5-flash
    let provider_slug = provider
        .split_whitespace()
        .next()
        .unwrap_or(provider)
        .to_lowercase();
    // PRIME should be white on black and version should be black on white.
    let right_line_1 =
        format!("{} {}", "PRIME".white().bold(), version.black().on_white()).to_string();
    let right_line_2 = format!("{}/{}", provider_slug, model)
        .dark_grey()
        .to_string();

    // Move cursor up two lines (to first logo line), then right to column, print overlays
    print!("\x1B[2A\x1B[{}C", overlay_col);
    println!("{}", right_line_1);
    print!("\x1B[{}C", overlay_col);
    println!("{}", right_line_2);
    // Return to a fresh line below the banner
    // (we already advanced with println above)

    // Thick rule
    println!("{}", "━".repeat(70).dark_grey());

    // Guidance lines in dim gray, with PWD and commands highlighted.
    let pwd = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .display()
        .to_string();

    // Line 1: "prime is running in <PWD>. you can use"
    println!(
        "{} {} {}",
        "prime is running in".dark_grey(),
        pwd.dark_cyan().bold(),
        ". you can use".dark_grey()
    );
    // Line 2: ":help … :exit …"
    println!(
        "{} {} {} {} {}",
        ":help".dark_yellow().bold(),
        "to see available commands or".dark_grey(),
        ":exit".dark_yellow().bold(),
        "to quit.".dark_grey(),
        "" // keep format! arity simple
    );

    // Thin rule
    println!("{}", "─".repeat(70).dark_grey());
}

pub async fn run_repl(mut session: PrimeSession) -> Result<()> {
    let mut editor = Editor::<PrimeHelper, DefaultHistory>::new()
        .context("Failed to initialize rustyline editor")?;
    editor.set_helper(Some(PrimeHelper {}));

    // Load history from file
    let prime_config_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".prime");
    let history_file = prime_config_dir.join("history.txt");

    if history_file.exists() {
        editor.load_history(&history_file).unwrap_or_else(|e| {
            eprintln!(
                "{}",
                format!("Warning: Failed to load history: {}", e).yellow()
            );
        });
    }

    // **ANSI-free prompt** so Rustyline counts width correctly
    let prompt = "» ".to_string();

    loop {
        match editor.readline(&prompt) {
            Ok(line) => {
                let _ = editor.add_history_entry(line.as_str());
                let input = line.trim();

                if input.is_empty() {
                    continue;
                }
                if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                    break;
                }

                // Support both ":command" and "!command" special commands
                if input.starts_with('!') || input.starts_with(':') {
                    if !handle_special_command(&input[1..], &session)? {
                        break;
                    }
                    continue;
                }

                // 1) Expand @-references by injecting file/dir content *appended* to the prompt.
                let final_prompt = match expand_inline_refs_with_budget(input) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("{}", format!("[ERROR] while injecting @ refs: {}", e).red());
                        input.to_string()
                    }
                };

                let display_input = input; // what the user typed
                let llm_input = &final_prompt; // with <<INJECTED_CONTEXT_…>>
                if let Err(e) = session.process_input(display_input, llm_input).await {
                    eprintln!("{}", format!("[ERROR] {}", e).red());
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!(
                    "\n{}",
                    "Interrupted. Type 'exit' or Ctrl-D to exit.".yellow()
                );
            }
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("{}", format!("Input error: {}", err).red());
                break;
            }
        }
    }

    // Save history to file
    if !prime_config_dir.exists() {
        std::fs::create_dir_all(&prime_config_dir).unwrap_or_else(|e| {
            eprintln!(
                "{}",
                format!("Warning: Failed to create config directory: {}", e).yellow()
            );
        });
    }

    editor.save_history(&history_file).unwrap_or_else(|e| {
        eprintln!(
            "{}",
            format!("Warning: Failed to save history: {}", e).yellow()
        );
    });

    Ok(())
}

fn handle_special_command(cmd_line: &str, session: &PrimeSession) -> Result<bool> {
    let parts: Vec<&str> = cmd_line.splitn(2, ' ').collect();
    let command = parts[0].to_lowercase();
    let args = if parts.len() > 1 { parts[1] } else { "" };

    match command.as_str() {
        "clear" | "cls" => {
            print!("\x1B[2J\x1B[1;1H");
            io::stdout()
                .flush()
                .context("Failed to flush stdout for clear command")?;
            Ok(true)
        }
        "help" => {
            println!("{}", "Available Commands:".white().bold());
            println!(
                "  {:<20} - Show this help message.",
                ":help".yellow().bold()
            );
            println!(
                "  {:<20} - Clear the terminal screen.",
                ":clear / :cls".yellow().bold()
            );
            println!(
                "  {:<20} - Show the full conversation log.",
                ":log".yellow().bold()
            );
            println!(
                "  {:<20} - Read long-term or short-term memory.",
                ":memory [long|short]".yellow().bold()
            );
            println!("  {:<20} - Exit Prime.", ":exit / :quit".yellow().bold());
            println!(
                "{}",
                "(Tip: '!' versions like !help also work.)".dark_grey()
            );
            println!("{}", "You can reference files/dirs inline using @path, @\"path with spaces\", or @'quoted path'.".dark_grey());
            Ok(true)
        }
        "log" => {
            match session.list_messages() {
                Ok(content) => println!("{}", content),
                Err(e) => eprintln!("{}", format!("Error reading log: {}", e).red()),
            }
            Ok(true)
        }
        "memory" => {
            let memory_type = if args.contains("long") {
                Some("long_term")
            } else if args.contains("short") {
                Some("short_term")
            } else {
                None
            };
            match session.read_memory(memory_type) {
                Ok(content) => println!("{}", content),
                Err(e) => eprintln!("{}", format!("Error reading memory: {}", e).red()),
            }
            Ok(true)
        }
        "exit" | "quit" => Ok(false),
        _ => {
            println!(
                "{} Unknown command: {}. Type {} for help.",
                "Error:".red(),
                command,
                ":help".yellow().bold()
            );
            Ok(true)
        }
    }
}

pub struct PrimeHelper {}

impl Helper for PrimeHelper {}

impl Highlighter for PrimeHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // Style both ":" and "!" command prefixes
        if let Some(cmd) = line.strip_prefix('!').or_else(|| line.strip_prefix(':')) {
            if let Some(space_idx) = cmd.find(' ') {
                let (command, rest) = cmd.split_at(space_idx);
                let styled = format!(":{}{}", command.yellow().bold(), rest);
                return Cow::Owned(styled);
            } else {
                return Cow::Owned(format!(":{}", cmd.yellow().bold()));
            }
        }

        // Dim any @token (keeps it visible but subtle)
        if let Some((start, end)) = current_at_span(line, line.len()) {
            let mut s = String::new();
            s.push_str(&line[..start]);
            s.push_str(&line[start..end].dark_grey().to_string());
            s.push_str(&line[end..]);
            return Cow::Owned(s);
        }

        Cow::Borrowed(line)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(hint.dark_grey().to_string())
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        _completion: rustyline::CompletionType,
    ) -> Cow<'c, str> {
        Cow::Owned(candidate.to_string())
    }
}

impl Hinter for PrimeHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &RustylineContext<'_>) -> Option<String> {
        None
    }
}

impl Completer for PrimeHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &RustylineContext,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        // Autocomplete for ":" and "!" commands (keep simple).
        if line.starts_with('!') || line.starts_with(':') {
            let commands = [
                (":help", "help"),
                (":clear", "clear"),
                (":cls", "cls"),
                (":log", "log"),
                (":memory", "memory"),
                (":memory long", "memory long"),
                (":memory short", "memory short"),
                (":exit", "exit"),
                (":quit", "quit"),
            ];
            let mut candidates = Vec::new();
            let prefix = &line[..pos];
            for (cmd, display) in commands {
                if cmd.starts_with(prefix) {
                    candidates.push(Pair {
                        display: format!("{} ({})", cmd, display),
                        replacement: format!("{}{}", &line[0..1], &cmd[1..]),
                    });
                }
            }
            return Ok((pos, candidates));
        }

        // @-path completion
        if let Some((start, token)) = current_at_token(line, pos) {
            let mut pairs = Vec::new();
            for s in suggest_paths(&token) {
                pairs.push(Pair {
                    display: s.clone(),
                    replacement: format!("@{}", s),
                });
            }
            return Ok((start, pairs));
        }

        Ok((0, Vec::new()))
    }
}

impl Validator for PrimeHelper {
    fn validate(
        &self,
        _ctx: &mut rustyline::validate::ValidationContext,
    ) -> Result<rustyline::validate::ValidationResult, ReadlineError> {
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }
}

/* ===========================
@-REFERENCE INJECTION LOGIC
=========================== */

/// Expand @file and @dir references by appending blocks to the end of the prompt,
/// with a global size budget derived from LLM_MAX_TOKENS.
fn expand_inline_refs_with_budget(input: &str) -> Result<String> {
    // Collect refs (unique, in order)
    let refs = collect_at_refs(input);
    if refs.is_empty() {
        return Ok(input.to_string());
    }

    // Budget based on model max tokens
    let max_tokens = std::env::var("LLM_MAX_TOKENS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(2048);
    let inject_tokens =
        (max_tokens * INJECT_TOKEN_BUDGET_FRACTION_NUM) / INJECT_TOKEN_BUDGET_FRACTION_DEN;
    let mut budget_chars = inject_tokens.saturating_mul(CHARS_PER_TOKEN);

    // Never exceed a hard upper bound to avoid massive prompts
    let hard_cap = 1_000_000usize; // 1M chars safety net
    if budget_chars > hard_cap {
        budget_chars = hard_cap;
    }

    // Build blocks, trimming to budget
    let mut blocks: Vec<String> = Vec::new();
    let mut seen = HashSet::new();

    for r in refs {
        if !seen.insert(r.clone()) {
            continue;
        }
        let block = build_block_for_ref(&r)?;
        if block.is_empty() {
            continue;
        }

        if block.len() <= budget_chars {
            blocks.push(block);
            budget_chars = budget_chars.saturating_sub(blocks.last().unwrap().len());
        } else {
            // Trim the block to remaining budget with a clear marker.
            if budget_chars > 0 {
                let mut trimmed = block;
                if budget_chars < trimmed.len() {
                    trimmed.truncate(budget_chars);
                    trimmed.push_str("\n… (injection truncated)\n");
                }
                blocks.push(trimmed);
                budget_chars = 0;
            }
            break;
        }
    }

    if blocks.is_empty() {
        return Ok(input.to_string());
    }

    let mut out =
        String::with_capacity(input.len() + blocks.iter().map(|b| b.len()).sum::<usize>() + 32);
    out.push_str(input.trim_end());
    out.push_str("\n\n");
    out.push_str("<<INJECTED_CONTEXT_BEGIN>>\n");
    for b in blocks {
        out.push_str(&b);
        if !out.ends_with('\n') {
            out.push('\n');
        }
    }
    out.push_str("<<INJECTED_CONTEXT_END>>\n");
    Ok(out)
}

/// Collect @-references from a line. Supports:
///  - @path
///  - @"path with spaces"
///  - @'path with spaces'
fn collect_at_refs(line: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'@' {
            let start = i;
            i += 1;
            if i >= bytes.len() {
                break;
            }
            let c = bytes[i] as char;
            if c == '"' || c == '\'' {
                let quote = c;
                i += 1;
                let mut j = i;
                while j < bytes.len() && (bytes[j] as char) != quote {
                    j += 1;
                }
                let path = &line[i..j];
                if !path.trim().is_empty() {
                    if let Some(p) = normalize_path_token(path) {
                        refs.push(p);
                    }
                }
                i = j.saturating_add(1);
            } else {
                // read until whitespace or another '@'
                let mut j = i;
                while j < bytes.len() {
                    let ch = bytes[j] as char;
                    if ch.is_whitespace() || ch == '@' {
                        break;
                    }
                    j += 1;
                }
                let token = &line[i..j];
                if !token.trim().is_empty() {
                    if let Some(p) = normalize_path_token(token) {
                        refs.push(p);
                    }
                }
                i = j;
            }
        } else {
            i += 1;
        }
    }
    refs
}

/// Normalize a token to a usable path string (expand ~).
fn normalize_path_token(token: &str) -> Option<String> {
    let t = token.trim();
    if t.is_empty() {
        return None;
    }
    if t.starts_with("~/") || t.starts_with("~\\") {
        if let Some(home) = dirs::home_dir() {
            let rest = &t[2..];
            let mut pb = PathBuf::from(home);
            pb.push(rest);
            return Some(pb.to_string_lossy().to_string());
        }
    }
    Some(t.to_string())
}

/// Build a single injected block for a file or directory reference.
fn build_block_for_ref(path_str: &str) -> Result<String> {
    let p = PathBuf::from(path_str);
    let target = if p.is_absolute() {
        p
    } else {
        std::env::current_dir()?.join(p)
    };
    if !target.exists() {
        return Ok(format!("<<MISSING path=\"{}\">>\n", path_str));
    }
    if target.is_dir() {
        let listing = dir_tree(&target, DIR_MAX_DEPTH, DIR_MAX_ENTRIES)?;
        return Ok(format!(
            "<<DIR path=\"{}\">>\n{}\n<<END DIR>>\n",
            target.display(),
            listing
        ));
    }
    // file
    let (content, truncated, binary) = read_text_file(&target, PER_FILE_MAX_BYTES)?;
    if binary {
        return Ok(format!(
            "<<FILE path=\"{}\" note=\"binary omitted\">>\n<<END FILE>>\n",
            target.display()
        ));
    }
    let lang = guess_lang_by_ext(target.extension().and_then(|e| e.to_str()));
    let mut block = String::new();
    block.push_str(&format!(
        "<<FILE path=\"{}\"{}>>\n",
        target.display(),
        if truncated { " truncated=\"true\"" } else { "" }
    ));
    if let Some(l) = lang {
        block.push_str(&format!("```{}\n", l));
    } else {
        block.push_str("```\n");
    }
    block.push_str(&content);
    if !content.ends_with('\n') {
        block.push('\n');
    }
    block.push_str("```\n");
    block.push_str("<<END FILE>>\n");
    Ok(block)
}

/// Read a text file safely (binary detection + size cap).
fn read_text_file(path: &Path, max_bytes: usize) -> Result<(String, bool, bool)> {
    let mut f = File::open(path).with_context(|| format!("open {}", path.display()))?;
    // Binary sniff
    let mut sniff = [0u8; 512];
    let n = f.read(&mut sniff).unwrap_or(0);
    if looks_binary(&sniff[..n]) {
        return Ok((String::new(), false, true));
    }
    // Reset and read
    let mut f = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut buf = Vec::new();
    let mut handle = f.take(max_bytes as u64);
    handle.read_to_end(&mut buf)?;
    let mut truncated = false;
    // If file is larger than cap, mark truncated
    if fs::metadata(path).map(|m| m.len() as usize).unwrap_or(0) > max_bytes {
        truncated = true;
    }
    let text = String::from_utf8_lossy(&buf).to_string();
    Ok((text, truncated, false))
}

fn looks_binary(buf: &[u8]) -> bool {
    // Simple heuristic: any NUL or >30% non-utf8ish bytes in sniff
    if buf.iter().any(|&b| b == 0) {
        return true;
    }
    let mut nonprint = 0;
    for &b in buf {
        let c = b as char;
        if !(c.is_ascii_graphic() || c.is_ascii_whitespace()) {
            nonprint += 1;
        }
    }
    nonprint * 10 > buf.len() * 3
}

/// Produce a compact directory tree (depth-limited, entry-limited).
fn dir_tree(root: &Path, max_depth: usize, max_entries: usize) -> Result<String> {
    fn helper(
        dir: &Path,
        depth: usize,
        max_depth: usize,
        left: &mut usize,
        out: &mut String,
    ) -> Result<()> {
        if depth > max_depth || *left == 0 {
            return Ok(());
        }
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };
        let mut v: Vec<PathBuf> = Vec::new();
        for e in entries {
            if *left == 0 {
                break;
            }
            if let Ok(entry) = e {
                v.push(entry.path());
            }
        }
        v.sort_by(|a, b| {
            a.file_name()
                .unwrap_or_default()
                .to_ascii_lowercase()
                .cmp(&b.file_name().unwrap_or_default().to_ascii_lowercase())
        });
        for p in v {
            if *left == 0 {
                break;
            }
            *left -= 1;
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            out.push_str(&format!(
                "{}{}\n",
                "  ".repeat(depth),
                if p.is_dir() {
                    format!("{}/", name)
                } else {
                    name.to_string()
                }
            ));
            if p.is_dir() {
                helper(&p, depth + 1, max_depth, left, out)?;
            }
        }
        Ok(())
    }

    let mut out = String::new();
    let mut left = max_entries;
    out.push_str(&format!(
        "{}/\n",
        root.file_name().and_then(|s| s.to_str()).unwrap_or("")
    ));
    if max_depth == 0 || max_entries == 0 {
        return Ok(out);
    }
    helper(root, 1, max_depth, &mut left, &mut out)?;
    if left == 0 {
        out.push_str("… (directory listing truncated)\n");
    }
    Ok(out)
}

fn guess_lang_by_ext(ext_opt: Option<&str>) -> Option<&'static str> {
    match ext_opt.map(|s| s.to_ascii_lowercase()).as_deref() {
        Some("rs") => Some("rust"),
        Some("toml") => Some("toml"),
        Some("json") => Some("json"),
        Some("md") => Some("md"),
        Some("txt") => Some("text"),
        Some("py") => Some("python"),
        Some("js") => Some("javascript"),
        Some("ts") => Some("ts"),
        Some("tsx") => Some("tsx"),
        Some("jsx") => Some("jsx"),
        Some("html") => Some("html"),
        Some("css") => Some("css"),
        Some("yml") | Some("yaml") => Some("yaml"),
        Some("go") => Some("go"),
        Some("java") => Some("java"),
        Some("kt") => Some("kotlin"),
        Some("c") => Some("c"),
        Some("h") => Some("c"),
        Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") | Some("hh") => Some("cpp"),
        _ => None,
    }
}

/* ===========================
@-PATH AUTOCOMPLETE HELPERS
=========================== */

/// Returns (start_index_in_line, token_without_at) for the @-token under cursor.
fn current_at_token(line: &str, pos: usize) -> Option<(usize, String)> {
    if pos > line.len() {
        return None;
    }
    // Find the start of the current "word"
    let mut i = pos;
    // Move left to the nearest '@' that isn't separated by whitespace
    while i > 0 {
        let ch = line.as_bytes()[i - 1] as char;
        if ch == '@' {
            let token = &line[i..pos];
            return Some((i - 1, token.to_string()));
        }
        if ch.is_whitespace() {
            break;
        }
        i -= 1;
    }
    // Also handle the case where cursor is immediately after '@'
    if i < line.len() && line.as_bytes()[i] == b'@' {
        return Some((i, String::new()));
    }
    None
}

/// Returns (start_index, end_index) of the @-span under cursor for highlighting.
fn current_at_span(line: &str, pos: usize) -> Option<(usize, usize)> {
    if pos > line.len() {
        return None;
    }
    let bytes = line.as_bytes();
    let mut start = None;
    let mut i = pos;
    while i > 0 {
        let ch = bytes[i - 1] as char;
        if ch == '@' {
            start = Some(i - 1);
            break;
        }
        if ch.is_whitespace() {
            break;
        }
        i -= 1;
    }
    let s = start?;
    let mut j = s + 1;
    while j < bytes.len() {
        let ch = bytes[j] as char;
        if ch.is_whitespace() || ch == '@' {
            break;
        }
        j += 1;
    }
    Some((s, j))
}

/// Suggest filesystem paths given a token after '@'.
/// Supports nested paths like "src/ma".
fn suggest_paths(token_after_at: &str) -> Vec<String> {
    let sep = std::path::MAIN_SEPARATOR;
    let token = token_after_at.replace(['\\', '/'], &sep.to_string());
    let (dir_part, name_prefix) = if let Some(idx) = token.rfind(sep) {
        (token[..idx].to_string(), token[idx + 1..].to_string())
    } else {
        ("".to_string(), token)
    };

    // Resolve base directory
    let base_dir = if dir_part.is_empty() {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    } else {
        let mut p = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        p.push(&dir_part);
        p
    };

    let mut out = Vec::new();
    if let Ok(rd) = fs::read_dir(&base_dir) {
        for entry in rd.flatten() {
            if let Ok(ft) = entry.file_type() {
                let name = entry.file_name().to_string_lossy().to_string();
                if starts_with_ci(&name, &name_prefix) {
                    let mut s = String::new();
                    if !dir_part.is_empty() {
                        s.push_str(&dir_part);
                        s.push(sep);
                    }
                    s.push_str(&name);
                    if ft.is_dir() {
                        s.push(sep);
                    }
                    out.push(normalize_separators_for_input(s));
                }
            }
        }
    }
    out.sort();
    out
}

fn starts_with_ci(hay: &str, needle: &str) -> bool {
    hay.to_ascii_lowercase()
        .starts_with(&needle.to_ascii_lowercase())
}

fn normalize_separators_for_input(p: String) -> String {
    // Keep user's platform style (MAIN_SEPARATOR).
    p
}
