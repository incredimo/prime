// src/terminal_ui.rs
// UI helpers for the terminal, like prompt styling, validation, and hinting.

use std::borrow::Cow;
use console::Style;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Helper;
use rustyline::Context as RustylineContext; // Alias rustyline::Context

// APP_NAME is a const in main.rs.
// For the helper to access it, it's best to pass it during PrimeHelper construction
// or make APP_NAME a public const in a shared module (e.g., lib.rs or a new consts.rs).
// For now, PrimeHelper will take APP_NAME as a field.

/// Helper to provide colored prompts, validation, and hints.
pub struct PrimeHelper {
 
}

impl PrimeHelper {
    pub fn new(app_name: &str) -> Self {
        Self {
 
        }
    }
}

impl Helper for PrimeHelper {}

impl Highlighter for PrimeHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&self, prompt: &'p str, default: bool) -> Cow<'b, str> {
        let _ = default; // Unused for now, but part of the trait
        // The 'prompt' received here is now the full "Prime » " string.
        // We just apply styling to it.
        Cow::Owned(Style::new().cyan().bright().bold().apply_to(prompt).to_string())
    }

    // Add other Highlighter methods if needed, with default implementations or specific logic
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        // Style the hint (e.g., make it dim)
        Cow::Owned(Style::new().dim().apply_to(hint).to_string())
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // Highlight the input line
        // Example: color special commands and their arguments
        let mut styled_line = String::new();
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            return Cow::Borrowed(line);
        }

        let first_word = parts[0];
        if first_word.starts_with('!') {
            // Special command
            let cmd_style = Style::new().yellow().bold();
            styled_line.push_str(&cmd_style.apply_to(first_word).to_string());

            if parts.len() > 1 {
                styled_line.push(' '); // Add space after command
                // Color arguments differently, e.g. !memory <arg>
                if first_word == "!memory" || first_word == "!read" {
                    let arg_style = Style::new().green();
                    styled_line.push_str(&arg_style.apply_to(parts[1..].join(" ")).to_string());
                } else {
                    // Default argument styling (or no style)
                    styled_line.push_str(&parts[1..].join(" "));
                }
            }
            // If there are trailing spaces, preserve them
            if line.ends_with(' ') && !styled_line.ends_with(' ') {
                 styled_line.push(' ');
            }
            Cow::Owned(styled_line)
        } else {
            // Regular input, no special highlighting for now
            Cow::Borrowed(line)
        }
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        _completion: rustyline::CompletionType,
    ) -> Cow<'c, str> {
        // No specific candidate highlighting for now
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
    type Hint = String; // Displayed hint

    fn hint(&self, line: &str, pos: usize, _ctx: &RustylineContext) -> Option<Self::Hint> {
        if pos < line.len() { // Only hint at the end of the line
            return None;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            return None;
        }

        let first_word = parts[0];

        if first_word.starts_with('!') {
            // Hint for commands
            if parts.len() == 1 {
                let cmd_part = &first_word[1..];
                if cmd_part.is_empty() { // Just "!"
                    return Some(" type command e.g. help, list, exit".to_string());
                }
                let special_commands = ["help", "memory", "list", "read", "clear", "cls", "exit", "quit"];
                for cmd_template in special_commands.iter() {
                    if cmd_template.starts_with(cmd_part) && cmd_part.len() < cmd_template.len() {
                        return Some(cmd_template[cmd_part.len()..].to_string());
                    }
                }
            }
            // Hint for arguments
            else if parts.len() == 2 && first_word == "!memory" {
                let arg_part = parts[1];
                let memory_args = ["short", "long", "all"];
                for mem_arg_template in memory_args.iter() {
                    if mem_arg_template.starts_with(arg_part) && arg_part.len() < mem_arg_template.len() {
                        return Some(mem_arg_template[arg_part.len()..].to_string());
                    }
                }
                 if arg_part.is_empty() || memory_args.iter().all(|&a| a != arg_part) && !memory_args.iter().any(|&a| a.starts_with(arg_part)) {
                    return Some(" [short|long|all]".to_string());
                }

            } else if parts.len() == 2 && first_word == "!read" {
                 let arg_part = parts[1];
                 if arg_part.is_empty() || arg_part.parse::<usize>().is_err() {
                    return Some(" <message_number>".to_string());
                 }
            }
        } else {
            // General input hints (could be expanded)
            if line == "ex" { return Some("it".to_string()); }
            if line == "qui" { return Some("t".to_string()); }
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
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let special_commands = [
            "!help", "!memory", "!list", "!read", "!clear", "!cls", "!exit", "!quit",
        ];
        let memory_args = ["short", "long", "all"];

        // Determine the token being completed and its start position
        // Example: line = "!memory sh", pos = 10
        // tokens = ["!memory", "sh"]
        // current_token_start = 8, current_token = "sh"

        let (start_pos, candidates) = if line.starts_with('!') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            
            // Calculate the start position of the token being completed
            let mut current_token_start = 0;
            let mut token_to_complete = "";
            let mut temp_pos = 0;
            for (i, part) in parts.iter().enumerate() {
                if pos >= temp_pos && pos <= temp_pos + part.len() {
                    current_token_start = temp_pos;
                    token_to_complete = part;
                    break;
                }
                temp_pos += part.len() + 1; // +1 for space
                 // If cursor is at a space after a token, suggest next token
                if i < parts.len() -1 && pos == temp_pos -1 {
                    current_token_start = pos;
                    token_to_complete = ""; // complete for next argument
                    break;
                }
                 // If cursor is at the end of the line after a space
                if i == parts.len() -1 && pos > temp_pos -1 {
                    current_token_start = pos;
                    token_to_complete = "";
                    break;
                }
            }
             // If pos is at the end and line ends with space, or line is just "!", or "!memory "
            if token_to_complete.is_empty() && pos > 0 && (line.ends_with(' ') || line == "!" || (parts.len() == 1 && parts[0] == "!memory")) {
                 current_token_start = pos; // Start new completion at cursor
            }


            if parts.is_empty() || (parts.len() == 1 && parts[0] == "!") { // Only "!" is typed or "! "
                let mut completions = Vec::new();
                for cmd in special_commands.iter() {
                    completions.push(Pair {
                        display: cmd.to_string(),
                        replacement: cmd.strip_prefix('!').unwrap_or(cmd).to_string(), // Replace "!" with "help"
                    });
                }
                (1, completions) // Replace from after "!"
            } else if parts.len() == 1 && parts[0].starts_with('!') { // Completing the command itself, e.g., "!mem"
                let cmd_part = &parts[0][1..]; // text after "!"
                let mut completions = Vec::new();
                for cmd_template in special_commands.iter() {
                    if cmd_template.starts_with("!") && cmd_template[1..].starts_with(cmd_part) {
                        completions.push(Pair {
                            display: cmd_template.to_string(),
                            replacement: cmd_template.strip_prefix('!').unwrap_or(cmd_template).to_string(),
                        });
                    }
                }
                (1, completions) // Replace from after "!"
            } else if parts.len() >= 2 && parts[0] == "!memory" { // Completing arguments for "!memory"
                let arg_part = if parts.len() == 2 { token_to_complete } else { "" }; // if parts.len() > 2, no arg completion for now
                let mut completions = Vec::new();
                for mem_arg in memory_args.iter() {
                    if mem_arg.starts_with(arg_part) {
                        completions.push(Pair {
                            display: mem_arg.to_string(),
                            replacement: mem_arg.to_string(),
                        });
                    }
                }
                // current_token_start should be the start of the argument
                let arg_start_pos = line.find(token_to_complete).unwrap_or(pos);
                (arg_start_pos, completions)
            }
            // Add more command argument completions here if needed for !read <number> etc.
            else {
                (0, Vec::new()) // No completions for other cases yet
            }
        } else {
            (0, Vec::new()) // Not a special command
        };

        Ok((start_pos, candidates))
    }
}