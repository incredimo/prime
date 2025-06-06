// main.rs
// Entry point for Prime terminal assistant

use std::env;
use std::process;
use std::sync::Arc;

use anyhow::{Context, Result};
use colored::*;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

mod session;
mod commands;
mod memory;

use session::PrimeSession;

const APP_NAME: &str = "Prime";
const VERSION: &str = "1.0.0";

fn main() -> Result<()> {
    println!(); // Start with a newline for spacing
    let bar_char = "─";
    let top_bar = bar_char.repeat(70).bright_blue();

    // Print banner
    println!("{}", top_bar);
    println!("  {} {} {} {}",
        APP_NAME.bright_blue().bold(),
        format!("v{}", VERSION).dimmed(),
        "|".bright_black(), // Dimmed separator
        "Your AI-Powered Terminal Companion".dimmed()
    );
    println!("{}\n", top_bar);

    // Initialize Prime
    let prime = match init_prime() {
        Ok(prime) => prime,
        Err(e) => {
            eprintln!("{} {}", "[ERROR]".red().bold(), format!("Initialization error: {}", e).red());
            process::exit(1);
        }
    };
    
    // Run Prime's main loop
    match prime.run() {
        Ok(_) => {
            println!("{} {}", "[OK]".green().bold(), "Prime session ended successfully".green());
            Ok(())
        },
        Err(e) => {
            eprintln!("{} {}", "[ERROR]".red().bold(), format!("Runtime error: {}", e).red());
            process::exit(1);
        }
    }
}

fn init_prime() -> Result<Prime> {
    let bar_char = "─";
    // Get configuration from environment variables
    let ollama_model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gemma3:latest".to_string());
    let ollama_api = env::var("OLLAMA_API").unwrap_or_else(|_| "http://localhost:11434".to_string());
    
    // Create base directory
    let base_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".prime");
    
    println!("  {} {:<18} {}", ">".dimmed(), "Using model:".bold(), ollama_model.cyan());
    println!("  {} {:<18} {}", ">".dimmed(), "API endpoint:".bold(), ollama_api.cyan());
    println!("  {} {:<18} {}", ">".dimmed(), "Data directory:".bold(), base_dir.display().to_string().cyan());
    println!("{}\n", bar_char.repeat(70).bright_black()); // Separator
    
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
    pub fn run(&self) -> Result<()> {
        println!("Type your requests below. Type {} or {} to quit.", "exit".bold().green(), "!exit".bold().green());
        
        let mut editor = DefaultEditor::new()?;
        
        loop {
            // Display prompt
            let prompt = format!("{} {} ", APP_NAME.bright_cyan().bold(), ">".bright_black());
            
            // Read user input
            match editor.readline(&prompt) {
                Ok(line) => {
                    let input = line.trim();
                    
                    // Check for exit command
                    if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                        println!("{}", "Exiting Prime...".dimmed());
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
                                eprintln!("{} {}", "[ERROR]".red().bold(), format!("Command error: {}", e).red());
                                continue;
                            }
                        }
                    }
                    
                    // Process user input
                    if let Err(e) = self.process_user_input(input) {
                        eprintln!("{} {}", "[ERROR]".red().bold(), format!("Error processing input: {}", e).red());
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
                    eprintln!("{} {}", "[ERROR]".red().bold(), format!("Input error: {}", err).red());
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    fn process_user_input(&self, initial_input: &str) -> Result<()> {
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
                eprintln!("{} {}", "[ERROR]".red().bold(), err_msg.red());
                self.session.add_system_message("InternalError", -1, err_msg)
                    .context("Failed to log max recursion depth error")?;
                return Err(anyhow::anyhow!(err_msg));
            }

            // Show thinking indicator
            println!("{} Processing...", "[~]".dimmed());
            
            // Generate AI response using the current prompt (which might include error context)
            let llm_response = match self.session.generate_prime_response(&current_llm_prompt, recursion_depth > 0) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{} {}", "[ERROR]".red().bold(), format!("Failed to generate LLM response: {}", e).red());
                    return Err(e);
                }
            };
    
            // Print the AI response with a clear separator
            // Note: The response is already saved to session history by generate_prime_response
            let line_len: usize = 70;
            let title = "Prime Response";
            let title_bar = format!("{} {}", title.bright_cyan().bold(), "─".repeat(line_len.saturating_sub(title.len() + 1)).bright_cyan());
            println!("\n{}", title_bar);
            
            // Print the response with syntax highlighting
            let highlighted_response = self.highlight_response(&llm_response);
            print!("{}", highlighted_response);
            
            println!("{}", "─".repeat(line_len).dimmed()); // Light footer
            
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
                        println!("{} Some commands failed. Attempting to correct (Attempt {}/{})...", "[WARN]".yellow().bold(), recursion_depth, MAX_RECURSION_DEPTH);
                        
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
                    eprintln!("{} {}", "[ERROR]".red().bold(), format!("Internal error during command processing: {}", e).red());
                    return Err(e);
                }
            }
        }
    }
    
    // Helper method to highlight parts of the response
    fn highlight_response(&self, response: &str) -> String {
        let mut result = String::new();
        let mut in_code_block = false;
        // let mut code_block_content = String::new(); // code_block_content is unused

        for line in response.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    // End of code block
                    in_code_block = false;
                    result.push_str(&format!("{}\n", line.bright_yellow()));
                    // code_block_content.clear();
                } else {
                    // Start of code block
                    in_code_block = true;
                    result.push_str(&format!("{}\n", line.bright_yellow()));
                }
            } else if in_code_block {
                // Inside code block
                // code_block_content.push_str(line);
                // code_block_content.push('\n');
                result.push_str(&format!("{}\n", line.yellow()));
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
                let title_bar = format!("{} {}", title.bright_magenta().bold(), "─".repeat(line_len.saturating_sub(title.len() + 1)).bright_magenta());
                
                match self.session.read_memory(Some(memory_type)) {
                    Ok(content) => {
                        println!("\n{}", title_bar);
                        println!("{}", content);
                        println!("{}", "─".repeat(line_len).dimmed());
                    },
                    Err(e) => eprintln!("Error reading memory: {}", e),
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
                let title_bar = format!("{} {}", title.bright_blue().bold(), "─".repeat(line_len.saturating_sub(title.len() + 1)).bright_blue());
                match self.session.list_messages() {
                    Ok(messages) => {
                        println!("\n{}", title_bar);
                        
                        for msg in messages {
                            println!("{}", msg);
                        }
                        
                        println!("{}", "─".repeat(line_len).dimmed());
                    },
                    Err(e) => eprintln!("Error listing messages: {}", e),
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
                    let title_bar = format!("{} {}", title.bright_blue().bold(), "─".repeat(line_len.saturating_sub(title.len() + 1)).bright_blue());
                    match self.session.read_message(msg_num) {
                        Ok(content) => {
                            println!("\n{}", title_bar);
                            
                            println!("{}", content);
                            
                            println!("{}", "─".repeat(line_len).dimmed());
                        },
                        Err(e) => eprintln!("Error reading message: {}", e),
                    }
                } else {
                    println!("Invalid message number: {}", args);
                }
                
                Ok(true)
            },
            "help" => {
                let title = "Prime Help";
                let title_bar = format!("{} {}", title.bright_cyan().bold(), "─".repeat(line_len.saturating_sub(title.len() + 1)).bright_cyan());
                println!("\n{}", title_bar);
                
                println!("Regular input: Send a request to Prime");
                println!("\nSpecial commands:");
                println!("  {:<28} {}", "!memory [short|long|all]".green(), "View memory content");
                println!("  {:<28} {}", "!list".green(), "List session messages");
                println!("  {:<28} {}", "!read <number>".green(), "Read a specific message");
                println!("  {:<28} {}", "!clear, !cls".green(), "Clear the screen");
                println!("  {:<28} {}", "!help".green(), "Show this help");
                println!("  {:<28} {}", "!exit, !quit".green(), "Exit Prime");
                
                println!("{}", "─".repeat(line_len).dimmed());
                
                Ok(true)
            },
            "exit" | "quit" => {
                println!("{}", "Exiting Prime...".dimmed());
                Ok(false) // Signal to exit
            },
            _ => {
                println!("Unknown command: !{}. Type !help for available commands.", command);
                Ok(true)
            }
        }
    }
}