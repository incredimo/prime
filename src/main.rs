// main.rs
// Entry point for Prime terminal assistant

use std::env;
use std::process;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use console::Style;
use indicatif::{ProgressBar, ProgressStyle};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

mod session;
mod commands;
mod memory;

use session::PrimeSession;

const APP_NAME: &str = "Prime";
const VERSION: &str = "1.0.0";

#[tokio::main]
async fn main() -> Result<()> {
    println!(); // Start with a newline for spacing
    
    // Initialize styles
    let header_style = Style::new().blue().bright().bold();
    let separator_style = Style::new().black().bright();
    let info_style = Style::new().black();
    let bar_char = "━"; // Using a heavier bar character
    let top_bar = separator_style.apply_to(bar_char.repeat(70));

    // Print banner
    println!("{}", header_style.apply_to(bar_char.repeat(70)));
    println!("  {} {} {} {}",
        header_style.apply_to(APP_NAME),
        info_style.apply_to(format!("v{}", VERSION)),
        separator_style.apply_to("│"), // Using a vertical bar
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
    
    // Run Prime's main loop
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
    let bar_char = "━"; // Using a heavier bar character
    // Get configuration from environment variables
    let ollama_model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gemma3:latest".to_string());
    let ollama_api = env::var("OLLAMA_API").unwrap_or_else(|_| "http://localhost:11434".to_string());
    
    // Create base directory
    let base_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".prime");
    
    let arrow_style = Style::new().black();
    let label_style = Style::new().bold();
    let value_style = Style::new().cyan();
    let separator_style = Style::new().black().bright();
    
    println!("  {} {:<18} {}",
        arrow_style.apply_to("»"), // Using a different arrow
        label_style.apply_to("Using model:"),
        value_style.apply_to(&ollama_model)
    );
    println!("  {} {:<18} {}",
        arrow_style.apply_to("»"), // Using a different arrow
        label_style.apply_to("API endpoint:"),
        value_style.apply_to(&ollama_api)
    );
    println!("  {} {:<18} {}",
        arrow_style.apply_to("»"), // Using a different arrow
        label_style.apply_to("Data directory:"),
        value_style.apply_to(&base_dir.display().to_string())
    );
    println!("{}\n", separator_style.apply_to(bar_char.repeat(70)));
    
    // Initialize session
    let session = PrimeSession::new(base_dir, &ollama_model, &ollama_api)?;
    
    Ok(Prime {
        session: Arc::new(session),
    })
}

/// The main Prime application
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
        
        let mut editor = DefaultEditor::new()?;
        
        loop {
            // Display prompt with consistent styling
            let prompt_style = Style::new().cyan().bright().bold();
            let arrow_style = Style::new().black().bright();
            let prompt = format!("{} {} ",
                prompt_style.apply_to(APP_NAME),
                arrow_style.apply_to("»") // Using a different arrow
            );
            
            // Read user input
            match editor.readline(&prompt) {
                Ok(line) => {
                    let input = line.trim();
                    
                    // Check for exit command
                    if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                        let info_style = Style::new().dim();
                        println!("{}", info_style.apply_to("Exiting Prime..."));
                        break;
                    }
                    
                    // Special commands
                    if input.starts_with("!") {
                        match self.handle_special_command(&input[1..]) {
                            Ok(should_continue) => {
                                if !should_continue {
                                    break;
                                }
                                continue;
                            },
                            Err(e) => {
                                let error_style = Style::new().red();
                                eprintln!("{} {}",
                                    error_style.apply_to("[ERROR]"),
                                    error_style.apply_to(format!("Command error: {}", e))
                                );
                                continue;
                            }
                        }
                    }
                    
                    // Process user input
                    if let Err(e) = self.process_user_input(input).await {
                        let error_style = Style::new().red();
                        eprintln!("{} {}",
                            error_style.apply_to("[ERROR]"),
                            error_style.apply_to(format!("Error processing input: {}", e))
                        );
                    }
                },
                Err(ReadlineError::Interrupted) => {
                    println!("Interrupted. Type 'exit' to quit.");
                },
                Err(ReadlineError::Eof) => {
                    println!("End of input. Goodbye!");
                    break;
                },
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
        // Skip empty input
        if initial_input.trim().is_empty() {
            return Ok(());
        }
        
        // Save user input as message
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
                    .context("Failed to log max recursion depth error")?;
                return Err(anyhow::anyhow!(err_msg));
            }

            // Show thinking indicator
            // Spinner is now handled within session.generate_prime_response_stream
            
            // Generate AI response using the current prompt (which might include error context)
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
    
            // Response is already displayed in real-time by generate_prime_response_stream
            // Just add footer separator
            let separator_style = Style::new().black().bright();
            println!("\n{}", separator_style.apply_to("━".repeat(70))); // Using a heavier bar character
            
            // Process commands from the latest LLM response
            // process_commands already saves system messages for each command output.
            match self.session.process_commands(&llm_response) {
                Ok(execution_results) => {
                    if execution_results.is_empty() {
                        println!("LLM provided no commands. Task considered complete or requires no action.");
                        return Ok(());
                    }

                    let mut failed_commands_details = String::new();
                    let mut all_succeeded = true;

                    for result in execution_results {
                        if !result.success {
                            all_succeeded = false;
                            failed_commands_details.push_str(&format!(
                                "Command:\n```\n{}\n```\nFailed with exit code {}.\nOutput:\n```\n{}\n```\n\n",
                                result.command, result.exit_code, result.output
                            ));
                        }
                    }

                    if all_succeeded {
                        println!("All commands executed successfully.");
                        return Ok(());
                    } else {
                        recursion_depth += 1;
                        let warn_style = Style::new().yellow();
                        println!("{} Some commands failed. Attempting to correct (Attempt {}/{})...",
                            warn_style.apply_to("[WARN]"),
                            recursion_depth,
                            MAX_RECURSION_DEPTH
                        );
                        
                        let previous_llm_prompt_that_failed = current_llm_prompt.clone();
                        current_llm_prompt = format!(
                            "The previous set of commands resulted in errors. \
                            Analyze the failures and provide corrected commands or an alternative approach. \
                            Ensure commands are in the correct Pandoc-attributed markdown format for execution.\n\n\
                            The prompt that led to the failed commands was:\n---\n{}\n---\n\n\
                            Failed command details:\n\
                            {}\
                            Provide only the corrected commands or steps. If you believe the task is unachievable or requires clarification, please state that.",
                            previous_llm_prompt_that_failed,
                            failed_commands_details
                        );
                        // Loop continues
                    }
                },
                Err(e) => {
                    // This is an error in the command *processing* logic itself, not command execution.
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
    
    // Helper method to highlight parts of the response
    fn highlight_response(&self, response: &str) -> String {
        let mut result = String::new();
        let mut in_code_block = false;
        let code_style = Style::new().yellow();
        let code_delim_style = Style::new().yellow().bright();

        for line in response.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    // End of code block
                    in_code_block = false;
                    result.push_str(&format!("{}\n", code_delim_style.apply_to(line)));
                } else {
                    // Start of code block
                    in_code_block = true;
                    result.push_str(&format!("{}\n", code_delim_style.apply_to(line)));
                }
            } else if in_code_block {
                // Inside code block
                result.push_str(&format!("{}\n", code_style.apply_to(line)));
            } else {
                // Regular text
                result.push_str(line);
                result.push('\n');
            }
        }
        result
    }
    
    fn handle_special_command(&self, cmd: &str) -> Result<bool> {
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let command = parts[0].to_lowercase();
        let args = parts.get(1).map_or("", |s| s.trim());
        let line_len: usize = 50;
        
        match command.as_str() {
            "memory" => {
                // !memory [short|long|all]
                let memory_type = if args.is_empty() { "all" } else { args };
                let title = format!("Memory Content: {}", memory_type);
                let title_style = Style::new().magenta().bold();
                let title_bar = format!("{} {}",
                    title_style.apply_to(&title),
                    title_style.apply_to("━".repeat(line_len.saturating_sub(title.len() + 1))) // Using a heavier bar character
                );
                
                match self.session.read_memory(Some(memory_type)) {
                    Ok(content) => {
                        println!("\n{}", title_bar);
                        println!("{}", content);
                        let separator_style = Style::new().dim();
                        println!("{}", separator_style.apply_to("━".repeat(line_len))); // Using a heavier bar character
                    },
                    Err(e) => {
                        let error_style = Style::new().red();
                        eprintln!("{}", error_style.apply_to(format!("Error reading memory: {}", e)));
                    },
                }
                
                Ok(true)
            },
            "clear" | "cls" => {
                // Clear screen using ANSI escape code
                print!("\x1B[2J\x1B[1;1H");
                Ok(true)
            },
            "list" => {
                // List current session messages
                let title = "Session Messages";
                let title_style = Style::new().blue().bright().bold();
                let title_bar = format!("{} {}",
                    title_style.apply_to(&title),
                    title_style.apply_to("━".repeat(line_len.saturating_sub(title.len() + 1))) // Using a heavier bar character
                );
                match self.session.list_messages() {
                    Ok(messages) => {
                        println!("\n{}", title_bar);
                        
                        for msg in messages {
                            println!("{}", msg);
                        }
                        
                        let separator_style = Style::new().dim();
                        println!("{}", separator_style.apply_to("━".repeat(line_len))); // Using a heavier bar character
                    },
                    Err(e) => {
                        let error_style = Style::new().red();
                        eprintln!("{}", error_style.apply_to(format!("Error listing messages: {}", e)));
                    },
                }
                
                Ok(true)
            },
            "read" => {
                // Read a specific message
                if args.is_empty() {
                    println!("Usage: !read <message_number>");
                    return Ok(true);
                }
                
                if let Ok(msg_num) = args.parse::<usize>() {
                    let title = format!("Message #{}", msg_num);
                    let title_style = Style::new().blue().bright().bold();
                    let title_bar = format!("{} {}",
                        title_style.apply_to(&title),
                        title_style.apply_to("━".repeat(line_len.saturating_sub(title.len() + 1))) // Using a heavier bar character
                    );
                    match self.session.read_message(msg_num) {
                        Ok(content) => {
                            println!("\n{}", title_bar);
                            
                            println!("{}", content);
                            
                            let separator_style = Style::new().dim();
                            println!("{}", separator_style.apply_to("━".repeat(line_len))); // Using a heavier bar character
                        },
                        Err(e) => {
                            let error_style = Style::new().red();
                            eprintln!("{}", error_style.apply_to(format!("Error reading message: {}", e)));
                        },
                    }
                } else {
                    println!("Invalid message number: {}", args);
                }
                
                Ok(true)
            },
            "help" => {
                let title = "Prime Help";
                let title_style = Style::new().cyan().bright().bold();
                let success_style = Style::new().green();
                let title_bar = format!("{} {}",
                    title_style.apply_to(&title),
                    title_style.apply_to("━".repeat(line_len.saturating_sub(title.len() + 1))) // Using a heavier bar character
                );
                println!("\n{}", title_bar);
                
                println!("Regular input: Send a request to Prime");
                println!("\nSpecial commands:");
                println!("  {:<28} {}", success_style.apply_to("!memory [short|long|all]"), "View memory content");
                println!("  {:<28} {}", success_style.apply_to("!list"), "List session messages");
                println!("  {:<28} {}", success_style.apply_to("!read <number>"), "Read a specific message");
                println!("  {:<28} {}", success_style.apply_to("!clear, !cls"), "Clear the screen");
                println!("  {:<28} {}", success_style.apply_to("!help"), "Show this help");
                println!("  {:<28} {}", success_style.apply_to("!exit, !quit"), "Exit Prime");
                
                let separator_style = Style::new().black();
                println!("{}", separator_style.apply_to("━".repeat(line_len))); // Using a heavier bar character
                
                Ok(true)
            },
            "exit" | "quit" => {
                let info_style = Style::new().dim();
                println!("{}", info_style.apply_to("Exiting Prime..."));
                Ok(false) // Signal to exit
            },
            _ => {
                println!("Unknown command: !{}. Type !help for available commands.", command);
                Ok(true)
            }
        }
    }
}