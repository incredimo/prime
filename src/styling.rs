use once_cell::sync::Lazy;
use crossterm::style::{Color, Stylize};
use std::borrow::Cow;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Helper;
use rustyline::Context as RustylineContext;
pub const BANNER: &str = r#"
   [38;2;230;230;230m██[0m[38;2;230;230;230m██[0m[38;2;230;230;230m██[0m [38;2;230;230;230m██[0m[38;2;63;81;181m██[0m[38;2;33;150;243m██[0m   [38;2;3;169;244m██[0m [38;2;0;150;136m██[0m[38;2;76;175;80m██[0m[38;2;205;220;57m██[0m[38;2;255;193;7m██[0m   [38;2;255;152;0m██[0m[38;2;255;87;34m██[0m[38;2;244;67;54m██[0m 
 [38;2;33;150;243m██[0m    [38;2;3;169;244m██[0m [38;2;0;150;136m██[0m    [38;2;76;175;80m██[0m [38;2;205;220;57m██[0m [38;2;255;193;7m██[0m  [38;2;255;152;0m██[0m  [38;2;255;87;34m██[0m [38;2;244;67;54m██[0m     
 [38;2;230;230;230m██[0m[38;2;230;230;230m██[0m[38;2;63;81;181m██[0m   [38;2;33;150;243m██[0m[38;2;3;169;244m██[0m[38;2;0;150;136m██[0m   [38;2;76;175;80m██[0m [38;2;205;220;57m██[0m  [38;2;255;193;7m██[0m  [38;2;255;152;0m██[0m [38;2;255;87;34m██[0m[38;2;244;67;54m██[0m   
 [38;2;63;81;181m██[0m       [38;2;33;150;243m██[0m    [38;2;3;169;244m██[0m [38;2;0;150;136m██[0m [38;2;76;175;80m██[0m  [38;2;205;220;57m██[0m  [38;2;255;193;7m██[0m [38;2;255;152;0m██[0m[38;2;255;87;34m██[0m[38;2;244;67;54m██[0m "#;

pub const APP_NAME: &str = "PRIME";
pub const BAR_CHAR: &str = "━";
pub const BANNER_WIDTH: usize = 70;

lazy_static::lazy_static! {
    pub static ref VERSION: String = env!("CARGO_PKG_VERSION").to_string();
}

pub static STYLER: Lazy<Styler> = Lazy::new(|| Styler::new());

pub struct Styler {
    success: Color,
    error: Color,
    warning: Color,
    info: Color,
    bold_white: Color,
    dim_gray: Color,
    command_exec: Color,
    prompt: Color,
    separator: Color,
    header: Color,
    command_alt: Color,
    llm_response: Color,
    version: Color,
    arrow: Color,
    label: Color,
    value: Color,
}

impl Styler {
    fn new() -> Self {
        Self {
            success: Color::Green,
            error: Color::Red,
            warning: Color::Yellow,
            info: Color::Blue,
            bold_white: Color::White,
            dim_gray: Color::DarkGrey,
            command_exec: Color::Cyan,
            prompt: Color::Magenta,
            separator: Color::DarkGrey,
            header: Color::Blue,
            command_alt: Color::Cyan,
            llm_response: Color::Green,
            version: Color::Yellow,
            arrow: Color::Cyan,
            label: Color::White,
            value: Color::Cyan,
        }
    }

    pub fn style_text<'a, D: std::fmt::Display>(&self, text: D, color: Color) -> impl std::fmt::Display + 'a {
        String::from(text.to_string()).with(color)
    }

    pub fn success_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.success)
    }

    pub fn error_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.error)
    }

    pub fn errorln<T: std::fmt::Display>(&self, msg: T) {
        eprintln!("{} {}", self.error_style("[ERROR]"), self.error_style(msg));
    }

    pub fn warning_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.warning)
    }

    pub fn info_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.info)
    }

    pub fn bold_white_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        String::from(text.to_string()).with(self.bold_white).bold()
    }

    pub fn dim_gray_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        String::from(text.to_string()).with(self.dim_gray).dim()
    }

    pub fn command_exec_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.command_exec)
    }

    pub fn prompt_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.prompt)
    }

    pub fn separator_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.separator)
    }

    pub fn header_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        String::from(text.to_string()).with(self.header).bold()
    }

    pub fn command_style_alt<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.command_alt)
    }

    pub fn llm_response_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.llm_response)
    }

    pub fn print_help(&self) {
        println!("{}", self.header_style("Prime Assistant - Enhanced Terminal AI"));
        println!();
        println!("{}", self.info_style("Prime helps you accomplish tasks by executing code and managing files."));
        println!("{}", self.info_style("Simply describe what you want to do, and Prime will create and execute the necessary scripts."));
        println!();
        println!("{}", self.header_style("Available Special Commands:"));
        println!("  {:<20} - Show this help message", self.command_style_alt("!help"));
        println!("  {:<20} - List all messages in the current session", self.command_style_alt("!list"));
        println!("  {:<20} - Read a specific message by its number", self.command_style_alt("!read <number>"));
        println!("  {:<20} - Clear the terminal screen", self.command_style_alt("!clear | !cls"));
        println!("  {:<20} - Exit Prime", self.command_style_alt("!exit | !quit"));
        println!();
        println!("{}", self.header_style("Examples:"));
        println!("  {} Create a Python script that calculates prime numbers", self.dim_gray_style("•"));
        println!("  {} What files are in my current directory?", self.dim_gray_style("•"));
        println!("  {} Download the latest data from my API and save it as JSON", self.dim_gray_style("•"));
        println!("  {} Create a backup of my important files", self.dim_gray_style("•"));
    }

    pub fn successln<T: std::fmt::Display>(&self, msg: T) {
        println!("{} {}", self.success_style("[OK]"), self.success_style(msg));
    }

    pub fn infoln<T: std::fmt::Display>(&self, msg: T) {
        println!("{} {}", self.info_style("[INFO]"), self.info_style(msg));
    }

    pub fn warningln<T: std::fmt::Display>(&self, msg: T) {
        println!("{} {}", self.warning_style("[WARN]"), self.warning_style(msg));
    }

    pub fn command_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.command_exec)
    }

    pub fn version_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        String::from(text.to_string()).with(self.version).bold()
    }

    pub fn arrow_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        // Use a brighter cyan for better visibility
        String::from(text.to_string()).with(Color::Rgb { r: 0, g: 255, b: 255 })
    }

    pub fn label_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        String::from(text.to_string()).with(self.label).bold()
    }

    pub fn value_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        String::from(text.to_string()).with(self.value)
    }

    pub fn print_separator(&self) {
        println!("{}", self.separator_style(BAR_CHAR.repeat(BANNER_WIDTH)));
    }

    pub fn print_header(&self, version: &str) {
        println!("{}", BANNER);
        self.print_separator();
        println!(
            " {} {} {}",
            self.version_style(format!(" v{} ", version)),
            self.separator_style("│"),
            self.info_style("PERSONAL RESOURCE INTELLIGENCE MANAGEMENT ENGINE")
        );
        self.print_separator();
    }

    pub fn print_config(&self, model: &str, api: &str, workspace: &impl std::fmt::Display) {
        println!(
            "  {} {:<18} {}",
            self.arrow_style("•"),
            self.label_style("model"),
            self.value_style(model)
        );
        println!(
            "  {} {:<18} {}",
            self.arrow_style("•"),
            self.label_style("endpoint"),
            self.value_style(api)
        );
        println!(
            "  {} {:<18} {}",
            self.arrow_style("•"),
            self.label_style("workspace"),
            self.value_style(workspace)
        );
        self.print_separator();
    }
}

pub struct PrimeHelper {
    app_name: String,
}

impl PrimeHelper {
    pub fn new(app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
        }
    }
}

impl Helper for PrimeHelper {}

impl Highlighter for PrimeHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&self, prompt: &'p str, default: bool) -> Cow<'b, str> {
        let _ = default;
        Cow::Owned(
            STYLER.prompt_style(format!("{}", prompt)).to_string()
        )
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(STYLER.dim_gray_style(hint).to_string())
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let mut parts = line.split_whitespace();
        
        if let Some(first_word) = parts.next() {
            if !first_word.starts_with('!') {
                return Cow::Borrowed(line);
            }

            let mut styled_line = String::with_capacity(line.len() + 20);
            styled_line.push_str(&STYLER.command_style(first_word).to_string());

            let rest = line[first_word.len()..].to_string();
            if !rest.is_empty() {
                if first_word == "!memory" || first_word == "!read" {
                    styled_line.push_str(&STYLER.success_style(rest).to_string());
                } else {
                    styled_line.push_str(&rest);
                }
            }
            return Cow::Owned(styled_line);
        }
        Cow::Borrowed(line)
    }

    fn highlight_candidate<'c>(&self, candidate: &'c str, _completion: rustyline::CompletionType) -> Cow<'c, str> {
        Cow::Borrowed(candidate)
    }
}

impl Validator for PrimeHelper {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}

impl Hinter for PrimeHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &RustylineContext) -> Option<Self::Hint> {
        if pos < line.len() {
            return None;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let first_word = parts[0];
        if first_word.starts_with('!') {
            match (first_word, parts.len()) {
                ("!", 1) => Some(" type command e.g. help, list, exit".to_string()),
                (cmd, 1) => {
                    let cmd_part = &cmd[1..];
                    let special_commands = ["help", "memory", "list", "read", "clear", "cls", "exit", "quit"];
                    special_commands.iter()
                        .find(|&template| template.starts_with(cmd_part) && cmd_part.len() < template.len())
                        .map(|template| template[cmd_part.len()..].to_string())
                },
                ("!memory", 2) => {
                    let arg = parts[1];
                    let memory_args = ["short", "long", "all"];
                    if arg.is_empty() || (!memory_args.contains(&arg) && !memory_args.iter().any(|&a| a.starts_with(arg))) {
                        Some(" [short|long|all]".to_string())
                    } else {
                        memory_args.iter()
                            .find(|&&template| template.starts_with(arg) && arg.len() < template.len())
                            .map(|&template| template[arg.len()..].to_string())
                    }
                },
                ("!read", 2) => {
                    let arg = parts[1];
                    if arg.is_empty() || arg.parse::<usize>().is_err() {
                        Some(" <message_number>".to_string())
                    } else {
                        None
                    }
                },
                _ => None
            }
        } else {
            match line {
                "ex" => Some("it".to_string()),
                "qui" => Some("t".to_string()),
                _ => None
            }
        }
    }
}

impl Completer for PrimeHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &RustylineContext) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        if !line.starts_with('!') {
            return Ok((0, Vec::new()));
        }

        let special_commands = ["!help", "!memory", "!list", "!read", "!clear", "!cls", "!exit", "!quit"];
        let memory_args = ["short", "long", "all"];
        let parts: Vec<&str> = line[..pos].split_whitespace().collect();

        let (start_pos, candidates) = match parts.as_slice() {
            [] | ["!"] => (1, special_commands.iter()
                .map(|&cmd| Pair {
                    display: cmd.to_string(),
                    replacement: cmd[1..].to_string(),
                })
                .collect()),
            [cmd] if cmd.starts_with('!') => {
                let prefix = &cmd[1..];
                (1, special_commands.iter()
                    .filter(|&&cmd| cmd[1..].starts_with(prefix))
                    .map(|&cmd| Pair {
                        display: cmd.to_string(),
                        replacement: cmd[1..].to_string(),
                    })
                    .collect())
            },
            ["!memory", arg] => {
                let start = line.rfind(arg).unwrap_or(pos);
                (start, memory_args.iter()
                    .filter(|&&template| template.starts_with(arg))
                    .map(|&arg| Pair {
                        display: arg.to_string(),
                        replacement: arg.to_string(),
                    })
                    .collect())
            },
            _ => (0, Vec::new()),
        };

        Ok((start_pos, candidates))
    }
}