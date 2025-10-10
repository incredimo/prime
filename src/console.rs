
use std::borrow::Cow;
use std::io::{self, Write};
use std::path::PathBuf;
use anyhow::{Context, Result};
use crossterm::style::Stylize;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{Context as RustylineContext, Editor, Helper};
use crate::session::PrimeSession;
use std::env;

const BANNER: &str = r#"
 █▀█ ▄▀█ █ █▀█▀▄ █▀▀
 █▀▀ █▀▄ █ █ █ █ ██▄"#;

pub fn display_banner() {
    println!("{}", BANNER.bold().white());
    let version = env!("CARGO_PKG_VERSION");
    print!("\x1B[2A\x1B[25C");
    let vtag = format!(" V{} ", version);
    println!("{}", vtag.on_white().black().bold());
    let pwd = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .display()
        .to_string();
    print!("\x1B[25C");
    println!("{} {}", "PWD".bold().white(), pwd.cyan());
    println!("{}", "━".repeat(70).dark_grey());
}

pub fn display_init_info(
    model: &str,
    provider: &str,
    prime_config_base_dir: &PathBuf,
    workspace_dir: &PathBuf,
) {
    println!("model {}", model);
    println!("provider {}", provider);
    println!("configuration {}", prime_config_base_dir.display());
    println!("workspace {}", workspace_dir.display());
    println!("{}", "━".repeat(70).dark_grey());
}

pub async fn run_repl(mut session: PrimeSession) -> Result<()> {
    let mut editor = Editor::<PrimeHelper, DefaultHistory>::new()
        .context("Failed to initialize rustyline editor")?;
    editor.set_helper(Some(PrimeHelper {}));
   
    let prime_config_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".prime");
    let history_file = prime_config_dir.join("history.txt");
   
    if history_file.exists() {
        editor.load_history(&history_file).unwrap_or_else(|e| {
            eprintln!("{}", format!("Warning: Failed to load history: {}", e).yellow());
        });
    }
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
                if input.starts_with('!') {
                    if !handle_special_command(&input[1..], &mut session)? {
                        break;
                    }
                    continue;
                }
                if let Err(e) = session.process_input(input).await {
                    eprintln!("{}", format!("[ERROR] {}", e).red());
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("\n{}", "Interrupted. Type 'exit' or Ctrl-D to exit.".yellow());
            }
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("{}", format!("Input error: {}", err).red());
                break;
            }
        }
    }
   
    if !prime_config_dir.exists() {
        std::fs::create_dir_all(&prime_config_dir).unwrap_or_else(|e| {
            eprintln!("{}", format!("Warning: Failed to create config directory: {}", e).yellow());
        });
    }
   
    editor.save_history(&history_file).unwrap_or_else(|e| {
        eprintln!("{}", format!("Warning: Failed to save history: {}", e).yellow());
    });
   
    Ok(())
}

fn handle_special_command(cmd_line: &str, session: &mut PrimeSession) -> Result<bool> {
    let parts: Vec<&str> = cmd_line.splitn(2, ' ').collect();
    let command = parts[0].to_lowercase();
    let args = if parts.len() > 1 { parts[1] } else { "" };
    match command.as_str() {
        "clear" | "cls" => {
            print!("\x1B[2J\x1B[1;1H");
            io::stdout().flush().context("Failed to flush stdout for clear command")?;
            Ok(true)
        }
        "help" => {
            println!("{}", "Available Special Commands:".white().bold());
            println!(" {:<25} - Show this help message.", "!help".cyan());
            println!(" {:<25} - Clear the terminal screen.", "!clear | !cls".cyan());
            println!(
                " {:<25} - Show the full conversation log.",
                "!log".cyan()
            );
            println!(
                " {:<25} - Read long-term or short-term memory.",
                "!memory [long|short]".cyan()
            );
            println!(" {:<25} - List all available tools.", "!tools".cyan());
            println!(" {:<25} - Exit Prime.", "!exit | !quit".cyan());
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
        "tools" => {
            println!("{}", session.list_tools());
            Ok(true)
        }
        "exit" | "quit" => Ok(false),
        _ => {
            println!(
                "{} Unknown command: !{}. Type {} for help.",
                "Error:".red(),
                command,
                "!help".cyan()
            );
            Ok(true)
        }
    }
}

pub struct PrimeHelper {}

impl Helper for PrimeHelper {}

impl Highlighter for PrimeHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        if let Some(cmd) = line.strip_prefix('!') {
            if let Some(space_idx) = cmd.find(' ') {
                let (command, rest) = cmd.split_at(space_idx);
                let styled = format!("!{}{}", command.cyan(), rest);
                return Cow::Owned(styled);
            } else {
                return Cow::Owned(format!("!{}", cmd.cyan()));
            }
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

    fn hint(&self, line: &str, pos: usize, _ctx: &RustylineContext<'_>) -> Option<String> {
        if pos < line.len() {
            return None;
        }
        let commands = [
            "exit", "quit", "!help", "!clear", "!cls", "!log",
            "!memory", "!memory long", "!memory short", "!tools"
        ];
        for cmd in commands {
            if cmd.starts_with(line) && line.len() < cmd.len() {
                let suffix = &cmd[line.len()..];
                return Some(suffix.to_string());
            }
        }
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
        if line.starts_with('!') {
            let commands = [
                ("!help", "help"),
                ("!clear", "clear"),
                ("!cls", "cls"),
                ("!log", "log"),
                ("!memory", "memory"),
                ("!memory long", "memory long"),
                ("!memory short", "memory short"),
                ("!tools", "tools"),
                ("!exit", "exit"),
                ("!quit", "quit"),
            ];
            let mut candidates = Vec::new();
            let prefix = &line[..pos];
            for (cmd, display) in commands {
                if cmd.starts_with(prefix) {
                    candidates.push(Pair {
                        display: format!("{} ({})", cmd, display).to_string(),
                        replacement: cmd[1..].to_string(),
                    });
                }
            }
            return Ok((pos, candidates));
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
 