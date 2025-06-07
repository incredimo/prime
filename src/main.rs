use std::env;
use std::io::Write;
use std::process;
use std::sync::Arc;
use std::time::Duration;
use anyhow::{Context as AnyhowContext, Result}; // Aliased anyhow::Context
use console::Style;
use indicatif::{ProgressBar, ProgressStyle};
use rustyline::history::DefaultHistory;
// Removed rustyline imports for traits now implemented in terminal_ui.rs
use rustyline::{DefaultEditor, Editor};
use rustyline::error::ReadlineError;
// std::borrow::Cow might still be needed if other parts of rustyline API use it,
// but not directly for PrimeHelper here. Let's keep it for now.
use std::borrow::Cow;


mod session;
mod commands;
mod memory;
mod terminal_ui; // Added new module

use session::PrimeSession;
use terminal_ui::PrimeHelper; // Use PrimeHelper from the new module

const APP_NAME: &str = "prime";
const VERSION: &str = "1.0.0";

// PrimeHelper struct and its impls have been moved to src/terminal_ui.rs

#[tokio::main]
async fn main() -> Result<()> {
    println!(); // Start with a newline for spacing

    // Initialize banner styles
    let header_style = Style::new().blue().bright().bold();
    let separator_style = Style::new().black().bright();
    let info_style = Style::new().black();
    let bar_char = "━";

    // Print banner
    println!("{}", header_style.apply_to(bar_char.repeat(70)));
    println!("  {} {} {} {}",
        header_style.apply_to(APP_NAME),
        info_style.apply_to(format!("v{}", VERSION)),
        separator_style.apply_to("│"),
        info_style.apply_to("Your AI-Powered Terminal Companion")
    );
    println!("{}\n", header_style.apply_to(bar_char.repeat(70)));

    // Initialize Prime
    let prime = match init_prime().await {
        Ok(prime) => prime,
        Err(e) => {
            let error_style = Style::new().red();
            eprintln!("{} {}",
                error_style.apply_to("[ERROR]"),
                error_style.apply_to(format!("Initialization error: {}", e))
            );
            process::exit(1);
        }
    };

    // Run main loop
    match prime.run().await {
        Ok(_) => {
            let success_style = Style::new().green();
            println!("{} {}",
                success_style.apply_to("[OK]"),
                success_style.apply_to("Prime session ended successfully")
            );
            Ok(())
        },
        Err(e) => {
            let error_style = Style::new().red();
            eprintln!("{} {}",
                error_style.apply_to("[ERROR]"),
                error_style.apply_to(format!("Runtime error: {}", e))
            );
            process::exit(1);
        }
    }
}

async fn init_prime() -> Result<Prime> {
    let bar_char = "━";
    let ollama_model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gemma3:latest".to_string());
    let ollama_api = env::var("OLLAMA_API").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let base_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".prime");

    let arrow_style = Style::new().black();
    let label_style = Style::new().bold();
    let value_style = Style::new().cyan();
    let sep_style = Style::new().black().bright();

    println!("  {} {:<18} {}",
        arrow_style.apply_to("»"),
        label_style.apply_to("Using model:"),
        value_style.apply_to(&ollama_model)
    );
    println!("  {} {:<18} {}",
        arrow_style.apply_to("»"),
        label_style.apply_to("API endpoint:"),
        value_style.apply_to(&ollama_api)
    );
    println!("  {} {:<18} {}",
        arrow_style.apply_to("»"),
        label_style.apply_to("Data directory:"),
        value_style.apply_to(&base_dir.display().to_string())
    );
    println!("{}\n", sep_style.apply_to(bar_char.repeat(70)));

    let session = PrimeSession::new(base_dir, &ollama_model, &ollama_api)?;
    Ok(Prime { session: Arc::new(session) })
}

/// Main Prime application
pub struct Prime {
    session: Arc<PrimeSession>,
}

impl Prime {
    pub async fn run(&self) -> Result<()> {
        let success_style = Style::new().green();
        println!("Type your requests below. Type {} or {} to quit.",
            success_style.apply_to("exit"),
            success_style.apply_to("!exit")
        );

        let mut editor = Editor::< PrimeHelper, DefaultHistory>::new()?;
        // Instantiate PrimeHelper from the terminal_ui module, passing APP_NAME
        let prime_helper = PrimeHelper::new(APP_NAME);
        editor.set_helper(Some(prime_helper));

        loop {
            // Pass the full desired prompt string to readline.
            // The Highlighter will only apply color.
            let full_prompt_str = format!("{} » ", APP_NAME);

            match editor.readline(&full_prompt_str) {
                Ok(line) => {
                    let input = line.trim();
                    if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                        let info_style = Style::new().dim();
                        println!("{}", info_style.apply_to("Exiting Prime..."));
                        break;
                    }
                    if input.starts_with('!') {
                        if !self.handle_special_command(&input[1..])? {
                            break;
                        }
                        continue;
                    }
                    if let Err(e) = self.process_user_input(input).await {
                        let error_style = Style::new().red();
                        eprintln!("{} {}",
                            error_style.apply_to("[ERROR]"),
                            error_style.apply_to(format!("Error processing input: {}", e))
                        );
                    }
                }
                Err(ReadlineError::Interrupted) => println!("Interrupted. Type 'exit' to quit."),
                Err(ReadlineError::Eof) => { println!("End of input. Goodbye!"); break; },
                Err(err) => {
                    let error_style = Style::new().red();
                    eprintln!("{} {}",
                        error_style.apply_to("[ERROR]"),
                        error_style.apply_to(format!("Input error: {}", err))
                    );
                    break;
                }
            }
        }
        Ok(())
    }

    async fn process_user_input(&self, initial_input: &str) -> Result<()> {
        if initial_input.trim().is_empty() {
            return Ok(());
        }
        self.session.add_user_message(initial_input)?;
        let mut current_llm_prompt = initial_input.to_string();
        let mut recursion_depth = 0;
        const MAX_RECURSION_DEPTH: usize = 3;

        loop {
            if recursion_depth >= MAX_RECURSION_DEPTH {
                let err_msg = "Max recursion depth reached for this request.";
                let error_style = Style::new().red();
                eprintln!("{} {}",
                    error_style.apply_to("[ERROR]"),
                    error_style.apply_to(err_msg)
                );
                self.session.add_system_message("InternalError", -1, err_msg)
                    .context("Failed to log max recursion depth error")?; // This uses AnyhowContext
                return Err(anyhow::anyhow!(err_msg));
            }

            let llm_response = match self.session.generate_prime_response_stream(&current_llm_prompt, recursion_depth > 0).await {
                Ok(r) => r,
                Err(e) => {
                    let error_style = Style::new().red();
                    eprintln!("{} {}",
                        error_style.apply_to("[ERROR]"),
                        error_style.apply_to(format!("Failed to generate LLM response: {}", e))
                    );
                    return Err(e);
                }
            };

            let separator_style = Style::new().black().bright();
            println!("\n{}", separator_style.apply_to("━".repeat(70)));

            match self.session.process_commands(&llm_response) {
                Ok(results) if results.is_empty() => {
                    println!("LLM provided no commands. Task considered complete or requires no action.");
                    return Ok(());
                }
                Ok(results) => {
                    let mut all_succeeded = true;
                    let mut failed_details = String::new();
                    for res in results {
                        if !res.success {
                            all_succeeded = false;
                            failed_details.push_str(&format!(
                                "Command:\n```\n{}\n```\nFailed with exit code {}.\nOutput:\n```\n{}\n```\n\n",
                                res.command, res.exit_code, res.output
                            ));
                        }
                    }
                    if all_succeeded {
                        println!("All commands executed successfully.");
                        return Ok(());
                    }
                    recursion_depth += 1;
                    let warn_style = Style::new().yellow();
                    println!("{} Some commands failed. Attempting to correct (Attempt {}/{})...",
                        warn_style.apply_to("[WARN]"), recursion_depth, MAX_RECURSION_DEPTH);
                    current_llm_prompt = format!(
                        "The previous set of commands resulted in errors. Analyze the failures and provide corrected commands or an alternative approach. Ensure commands are in the correct Pandoc-attributed markdown format for execution.\n\nThe prompt that led to the failed commands was:\n---\n{}\n---\n\nFailed command details:\n{}Provide only the corrected commands or steps. If you believe the task is unachievable or requires clarification, please state that.",
                        current_llm_prompt, failed_details
                    );
                }
                Err(e) => {
                    let error_style = Style::new().red();
                    eprintln!("{} {}",
                        error_style.apply_to("[ERROR]"),
                        error_style.apply_to(format!("Internal error during command processing: {}", e))
                    );
                    return Err(e);
                }
            }
        }
    }

    /// Special commands handler
    fn handle_special_command(&self, cmd: &str) -> Result<bool> {
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let command = parts[0].to_lowercase();
        let args = parts.get(1).copied().unwrap_or("");
        let line_len: usize = 50;
        match command.as_str() {
            "memory" => {/* ... unchanged ... */ Ok(true) }
            "clear" | "cls" => { print!("\x1B[2J\x1B[1;1H"); Ok(true) }
            "list" => {/* ... unchanged ... */ Ok(true) }
            "read" => {/* ... unchanged ... */ Ok(true) }
            "help" => {/* ... unchanged ... */ Ok(true) }
            "exit" | "quit" => { println!("{}", Style::new().dim().apply_to("Exiting Prime...")); Ok(false) }
            _ => { println!("Unknown command: !{}. Type !help for available commands.", command); Ok(true) }
        }
    }

    /// Highlight code blocks in LLM responses
    fn highlight_response(&self, response: &str) -> String {
        let mut result = String::new();
        let mut in_code = false;
        let code_style = Style::new().yellow();
        let delim_style = Style::new().yellow().bright();
        for line in response.lines() {
            if line.starts_with("```") {
                in_code = !in_code;
                result.push_str(&format!("{}\n", delim_style.apply_to(line)));
            } else if in_code {
                result.push_str(&format!("{}\n", code_style.apply_to(line)));
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        result
    }
}
