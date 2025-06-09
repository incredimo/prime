use std::borrow::Cow;
use crate::styling::STYLER;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Helper;
use rustyline::Context as RustylineContext;
pub const BANNER: &str = r#"
   [38;2;230;230;230m██[0m[38;2;230;230;230m██[0m[38;2;230;230;230m██[0m [38;2;230;230;230m██[0m[38;2;63;81;181m██[0m[38;2;33;150;243m██[0m   [38;2;3;169;244m██[0m [38;2;0;150;136m██[0m[38;2;76;175;80m██[0m[38;2;205;220;57m██[0m[38;2;255;193;7m██[0m   [38;2;255;152;0m██[0m[38;2;255;87;34m██[0m[38;2;244;67;54m██[0m 
 [38;2;33;150;243m██[0m    [38;2;3;169;244m██[0m [38;2;0;150;136m██[0m    [38;2;76;175;80m██[0m [38;2;205;220;57m██[0m [38;2;255;193;7m██[0m  [38;2;255;152;0m██[0m  [38;2;255;87;34m██[0m [38;2;244;67;54m██[0m     
 [38;2;230;230;230m██[0m[38;2;230;230;230m██[0m[38;2;63;81;181m██[0m   [38;2;33;150;243m██[0m[38;2;3;169;244m██[0m[38;2;0;150;136m██[0m   [38;2;76;175;80m██[0m [38;2;205;220;57m██[0m  [38;2;255;193;7m██[0m  [38;2;255;152;0m██[0m [38;2;255;87;34m██[0m[38;2;244;67;54m██[0m   
 [38;2;63;81;181m██[0m       [38;2;33;150;243m██[0m    [38;2;3;169;244m██[0m [38;2;0;150;136m██[0m [38;2;76;175;80m██[0m  [38;2;205;220;57m██[0m  [38;2;255;193;7m██[0m [38;2;255;152;0m██[0m[38;2;255;87;34m██[0m[38;2;244;67;54m██[0m "#;

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