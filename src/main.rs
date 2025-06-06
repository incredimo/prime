// main.rs
// Entry point for Prime terminal assistant

use std::env;
use std::path::PathBuf;
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
    // Print banner
    println!("\n{}", format!("┌──────────────────────────────┐").bright_blue());
    println!("{}", format!("│ {} v{} │", APP_NAME, VERSION).bright_blue());
    println!("{}", format!("│ Terminal Assistant            │").bright_blue());
    println!("{}", format!("└──────────────────────────────┘").bright_blue());
    
    // Initialize Prime
    let prime = match init_prime() {
        Ok(prime) => prime,
        Err(e) => {
            eprintln!("{} {}", "✗".red(), format!("Initialization error: {}", e).red());
            process::exit(1);
        }
    };
    
    // Run Prime's main loop
    match prime.run() {
        Ok(_) => {
            println!("{} {}", "✓".green(), "Prime session ended successfully".green());
            Ok(())
        },
        Err(e) => {
            eprintln!("{} {}", "✗".red(), format!("Runtime error: {}", e).red());
            process::exit(1);
        }
    }
}

fn init_prime() -> Result<Prime> {
    // Get configuration from environment variables
    let ollama_model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gemma3:latest".to_string());
    let ollama_api = env::var("OLLAMA_API").unwrap_or_else(|_| "http://localhost:11434".to_string());
    
    // Create base directory
    let base_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".prime");
    
    println!("{} Using model: {}", "•".blue(), ollama_model);
    println!("{} API endpoint: {}", "•".blue(), ollama_api);
    println!("{} Data directory: {}", "•".blue(), base_dir.display());
    
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
        println!("\n{} Type your requests below. Type 'exit' to quit.", "•".blue());
        
        let mut editor = DefaultEditor::new()?;
        
        loop {
            // Display prompt
            let prompt = format!("{} ", "prime>".bright_cyan());
            
            // Read user input
            match editor.readline(&prompt) {
                Ok(line) => {
                    let input = line.trim();
                    
                    // Check for exit command
                    if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                        println!("Goodbye!");
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
                                eprintln!("{} {}", "✗".red(), format!("Command error: {}", e).red());
                                continue;
                            }
                        }
                    }
                    
                    // Process user input
                    if let Err(e) = self.process_user_input(input) {
                        eprintln!("{} {}", "✗".red(), format!("Error: {}", e).red());
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
                    eprintln!("{} {}", "✗".red(), format!("Input error: {}", err).red());
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    fn process_user_input(&self, input: &str) -> Result<()> {
        // Skip empty input
        if input.trim().is_empty() {
            return Ok(());
        }
        
        // Save user input as message
        self.session.add_user_message(input)?;
        
        // Show thinking indicator
        println!("{} {}", "⋯".bright_yellow(), "Thinking...".bright_yellow());
        
        // Generate AI response
        match self.session.generate_prime_response() {
            Ok(response) => {
                // Print the AI response with a clear separator
                println!("\n{}", format!("{}{}{}", "┌".bright_cyan(), "─".repeat(78).bright_cyan(), "┘".bright_cyan()));
                println!("{} {}", "│".bright_cyan(), "Prime Response:".bright_cyan().bold());
                println!("{}", format!("{}{}{}", "└".bright_cyan(), "─".repeat(78).bright_cyan(), "┘".bright_cyan()));
                
                // Print the response with syntax highlighting
                let highlighted_response = self.highlight_response(&response);
                print!("{}", highlighted_response);
                
                println!("\n{}", "─".repeat(80).bright_cyan());
                
                // Now process and execute any commands in the response
                if let Err(e) = self.session.process_commands(&response) {
                    eprintln!("{} {}", "✗".red(), format!("Command execution error: {}", e).red());
                }
                
                Ok(())
            },
            Err(e) => Err(anyhow::anyhow!("Failed to generate response: {}", e))
        }
    }
    
    // Helper method to highlight parts of the response
    fn highlight_response(&self, response: &str) -> String {
        let mut result = String::new();
        let mut in_code_block = false;
        let mut code_block_content = String::new();
        
        for line in response.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    // End of code block
                    in_code_block = false;
                    
                    // Add highlighted code block
                    result.push_str(&format!("{}\n", line.bright_yellow()));
                    
                    // Reset code block content
                    code_block_content.clear();
                } else {
                    // Start of code block
                    in_code_block = true;
                    result.push_str(&format!("{}\n", line.bright_yellow()));
                }
            } else if in_code_block {
                // Inside code block - collect content for highlighting
                code_block_content.push_str(line);
                code_block_content.push('\n');
                
                // Highlight command content
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
        
        match command.as_str() {
            "memory" => {
                // !memory [short|long|all]
                let memory_type = if args.is_empty() { "all" } else { args };
                
                match self.session.read_memory(Some(memory_type)) {
                    Ok(content) => {
                        println!("\n{}", format!("{}{}{}", "┌".bright_magenta(), "─".repeat(20).bright_magenta(), "┐".bright_magenta()));
                        println!("{} {} {}", "│".bright_magenta(), format!("Memory: {}", memory_type).bold(), "│".bright_magenta());
                        println!("{}", format!("{}{}{}", "└".bright_magenta(), "─".repeat(20).bright_magenta(), "┘".bright_magenta()));
                        println!("{}", content);
                        println!("{}", "─".repeat(40).bright_magenta());
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
                match self.session.list_messages() {
                    Ok(messages) => {
                        println!("\n{}", format!("{}{}{}", "┌".bright_blue(), "─".repeat(20).bright_blue(), "┐".bright_blue()));
                        println!("{} {} {}", "│".bright_blue(), "Session Messages".bold(), "│".bright_blue());
                        println!("{}", format!("{}{}{}", "└".bright_blue(), "─".repeat(20).bright_blue(), "┘".bright_blue()));
                        
                        for msg in messages {
                            println!("{}", msg);
                        }
                        
                        println!("{}", "─".repeat(40).bright_blue());
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
                    match self.session.read_message(msg_num) {
                        Ok(content) => {
                            println!("\n{}", format!("{}{}{}", "┌".bright_blue(), "─".repeat(20).bright_blue(), "┐".bright_blue()));
                            println!("{} {} {} {}", "│".bright_blue(), "Message".bold(), msg_num.to_string().bold(), "│".bright_blue());
                            println!("{}", format!("{}{}{}", "└".bright_blue(), "─".repeat(20).bright_blue(), "┘".bright_blue()));
                            
                            println!("{}", content);
                            
                            println!("{}", "─".repeat(40).bright_blue());
                        },
                        Err(e) => eprintln!("Error reading message: {}", e),
                    }
                } else {
                    println!("Invalid message number: {}", args);
                }
                
                Ok(true)
            },
            "help" => {
                println!("\n{}", "┌".bright_cyan().to_string() + &"─".bright_cyan().to_string().repeat(20) + &"┐".bright_cyan().to_string());
                println!("{} {} {}", "│".bright_cyan(), "Prime Help".bold(), "│".bright_cyan());
                println!("{}", format!("{}{}{}", "└".bright_cyan(), "─".repeat(20).bright_cyan(), "┘".bright_cyan()));
                
                println!("Regular input: Send a request to Prime");
                println!("\nSpecial commands:");
                println!("  {}: View memory content", "!memory [short|long|all]".bright_white().bold());
                println!("  {}: List session messages", "!list".bright_white().bold());
                println!("  {}: Read a specific message", "!read <number>".bright_white().bold());
                println!("  {}: Clear the screen", "!clear, !cls".bright_white().bold());
                println!("  {}: Show this help", "!help".bright_white().bold());
                println!("  {}: Exit Prime", "!exit, !quit".bright_white().bold());
                
                println!("\n{}", "─".repeat(40).bright_cyan());
                
                Ok(true)
            },
            "exit" | "quit" => {
                println!("Goodbye!");
                Ok(false) // Signal to exit
            },
            _ => {
                println!("Unknown command: !{}. Type !help for available commands.", command);
                Ok(true)
            }
        }
    }
}