mod styling;
mod templates;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use anyhow::{Context as AnyhowContext, Result};
use parking_lot::Mutex;
use console::Style;
use rustyline::history::DefaultHistory;
use rustyline::{Editor};
use rustyline::error::ReadlineError;

mod session;
mod commands;
mod terminal_ui;
mod config_utils;

use session::{PrimeSession, ProcessingSessionResult}; // Import ProcessingSessionResult
use terminal_ui::PrimeHelper;
use crate::styling::STYLER;

const APP_NAME: &str = "prime";
const VERSION: &str = "1.0.0";
use terminal_ui::BANNER;
 
#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", BANNER);

    let header_style = Style::new().white().bright().bold();
    let separator_style = Style::new().white().bright();
    let version_style = Style::new().on_yellow().bold();
    let info_style = Style::new().yellow();
    let bar_char = "━";

    println!("{}", header_style.apply_to(bar_char.repeat(70)));
    println!(
        " {} {} {}",
        version_style.apply_to(format!(" v{} ", VERSION)),
        separator_style.apply_to("│"),
        info_style.apply_to("PERSONAL RESOURCE INTELLIGENCE MANAGEMENT ENGINE")
    );
    println!("{}", header_style.apply_to(bar_char.repeat(70)));

    let prime = match init_prime().await {
        Ok(prime) => prime,
        Err(e) => {
            eprintln!(
                "{} {}",
                STYLER.error_style("[ERROR]"),
                STYLER.error_style(format!("Initialization error: {}", e))
            );
            process::exit(1);
        }
    };

    match prime.run().await {
        Ok(_) => {
            println!(
                "{} {}",
                STYLER.success_style("[OK]"),
                STYLER.success_style("Prime session ended successfully")
            );
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "{} {}",
                STYLER.error_style("[ERROR]"),
                STYLER.error_style(format!("Runtime error: {}", e))
            );
            process::exit(1);
        }
    }
}

async fn init_prime() -> Result<Prime> {
    let bar_char = "━";
    let ollama_model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gemma3:latest".to_string());
    let ollama_api = env::var("OLLAMA_API").unwrap_or_else(|_| "http://localhost:11434".to_string());
    
    let workspace_dir = env::current_dir().context("Failed to get current working directory")?;

    let arrow_style = Style::new().cyan().bright();
    let label_style = Style::new().bold();
    let value_style = Style::new().cyan();
    let sep_style = Style::new().black().bright();

    println!(
        "  {} {:<18} {}",
        arrow_style.apply_to("•"),
        label_style.apply_to("model"),
        value_style.apply_to(&ollama_model)
    );
    println!(
        "  {} {:<18} {}",
        arrow_style.apply_to("•"),
        label_style.apply_to("endpoint"),
        value_style.apply_to(&ollama_api)
    );
    println!(
        "  {} {:<18} {}",
        arrow_style.apply_to("•"),
        label_style.apply_to("workspace"),
        value_style.apply_to(&workspace_dir.display().to_string())
    );
    println!("{}", sep_style.apply_to(bar_char.repeat(70)));

    let session = PrimeSession::new(workspace_dir.clone(), &ollama_model, &ollama_api)?;
    Ok(Prime { session: Arc::new(Mutex::new(session)), workspace_dir })
}

pub struct Prime {
    session: Arc<Mutex<PrimeSession>>,
    #[allow(dead_code)]
    workspace_dir: PathBuf,
}

impl Prime {
    pub async fn run(&self) -> Result<()> {
        let mut editor = Editor::<PrimeHelper, DefaultHistory>::new()?;
        let prime_helper = PrimeHelper::new(APP_NAME);
        editor.set_helper(Some(prime_helper));

        loop {
            let prompt = "» ";
            match editor.readline(&prompt) {
                Ok(line) => {
                    editor.add_history_entry(line.as_str())?;

                    let input = line.trim();
                    if input.is_empty() {
                        continue;
                    }
                    if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                        println!("{}", STYLER.info_style("Exiting Prime..."));
                        break;
                    }
                    if input.starts_with('!') {
                        if !self.handle_special_command(&input[1..])? {
                            break;
                        }
                        continue;
                    }

                    if let Err(e) = self.process_user_input(input).await {
                        eprintln!(
                            "{} {}",
                            STYLER.error_style("[ERROR]"),
                            STYLER.error_style(format!("Error processing input: {}", e))
                        );
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("{}", STYLER.warning_style("Interrupted. Type 'exit' or Ctrl-D to quit."));
                }
                Err(ReadlineError::Eof) => {
                    println!("{}", STYLER.info_style("End of input. Goodbye!"));
                    break;
                }
                Err(err) => {
                    eprintln!(
                        "{} {}",
                        STYLER.error_style("[ERROR]"),
                        STYLER.error_style(format!("Input error: {}", err))
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
        
        let mut current_prompt = initial_input.to_string();
        let mut iteration_count = 0;
        const MAX_ITERATIONS: usize = 10; // Safety limit to prevent infinite loops

        // Get initial lock and add user message
        {
            let mut session = self.session.lock();
            session.add_user_message(initial_input)?;
        }
        
        loop {
            iteration_count += 1;
            
            if iteration_count > MAX_ITERATIONS {
                let error_msg = "Maximum iteration limit reached. Please try a simpler request.";
                eprintln!("{} {}", STYLER.error_style("[ERROR]"), STYLER.error_style(error_msg));
                self.session.lock().add_system_message("MaxIterations", "FAILED", error_msg)?;
                break;
            }
            
            // Generate LLM response and process commands under a single lock
            println!("\n{}", STYLER.separator_style("─".repeat(70)));
            let mut session = self.session.lock();
            
            let llm_response = match session.generate_prime_response_stream(&current_prompt).await {
                Ok(response) => response,
                Err(e) => {
                    eprintln!(
                        "{} {}",
                        STYLER.error_style("[ERROR]"),
                        STYLER.error_style(format!("Failed to generate response: {}", e))
                    );
                    return Err(e);
                }
            };
            
            println!("\n{}", STYLER.separator_style("─".repeat(70)));
            let processing_result: ProcessingSessionResult = match session.process_commands(&llm_response).await {
                Ok(result) => result,
                Err(e) => {
                    eprintln!(
                        "{} {}",
                        STYLER.error_style("[ERROR]"),
                        STYLER.error_style(format!("Error processing script blocks: {}", e))
                    );
                    return Err(e);
                }
            };
            
            // Determine next action based on processing results
            if processing_result.has_completed {
                // Task is marked as completed
                if let Some(final_msg) = processing_result.final_message {
                    println!("\n{} {}", 
                        STYLER.success_style("✓ Task completed:"), 
                        STYLER.bold_white_style(final_msg)
                    );
                } else {
                    println!("\n{}", STYLER.success_style("✓ Task completed successfully"));
                }
                break;
            } else if processing_result.script_results.is_empty() {
                // No script blocks found - conversation ends naturally
                println!("{}", STYLER.info_style("No script blocks found. Conversation complete."));
                break;
            } else {
                // Script blocks were executed - send results back to LLM
                let all_successful = processing_result.script_results.iter().all(|r| r.success);
                
                if all_successful {
                    println!("\n{} Sending execution results back to Prime...", 
                        STYLER.info_style("→"));
                } else {
                    println!("\n{} Some operations failed. Sending error details to Prime for correction...", 
                        STYLER.warning_style("⚠"));
                }
                
                // Prepare the next prompt with execution results
                current_prompt = format!(
                    "EXECUTION RESULTS:\n\n{}\n\nPlease analyze these results and provide your next response. If the task is complete, use completed=\"true\" in your final script block.",
                    processing_result.execution_summary
                );
                
                // Add the execution results as a user message for continuity
                session.add_user_message(&current_prompt)?;
            }
        }
        
        println!("\n{}", STYLER.separator_style("─".repeat(70)));
        Ok(())
    }

    fn handle_special_command(&self, cmd_line: &str) -> Result<bool> {
        let parts: Vec<&str> = cmd_line.splitn(2, ' ').collect();
        let command = parts[0].to_lowercase();
        let args = parts.get(1).copied().unwrap_or("").trim();

        match command.as_str() {
            "clear" | "cls" => {
                print!("\x1B[2J\x1B[1;1H");
                std::io::stdout().flush()?;
                Ok(true)
            }
            "list" => {
                match self.session.lock().list_messages() {
                    Ok(list) => {
                        println!("{}", STYLER.header_style("Conversation History:"));
                        if list.is_empty() {
                            println!("{}", STYLER.info_style("  No messages yet."));
                        } else {
                            for item in list {
                                println!("  {}", item);
                            }
                        }
                    }
                    Err(e) => eprintln!("{} {}", STYLER.error_style("Error listing messages:"), e),
                }
                Ok(true)
            }
            "read" => {
                if args.is_empty() {
                    eprintln!("{}", STYLER.error_style("Usage: !read <message_number>"));
                } else if let Ok(num) = args.parse::<usize>() {
                    match self.session.lock().read_message(num) {
                        Ok(msg) => println!("{}", msg),
                        Err(e) => eprintln!("{} {}", STYLER.error_style(format!("Error reading message {}:", num)), e),
                    }
                } else {
                    eprintln!("{} Invalid message number: {}", STYLER.error_style("Error:"), args);
                }
                Ok(true)
            }
            "help" => {
                println!("{}", STYLER.header_style("Prime Assistant - Enhanced Terminal AI"));
                println!();
                println!("{}", STYLER.info_style("Prime helps you accomplish tasks by executing code and managing files."));
                println!("{}", STYLER.info_style("Simply describe what you want to do, and Prime will create and execute the necessary scripts."));
                println!();
                println!("{}", STYLER.header_style("Available Special Commands:"));
                println!("  {:<20} - Show this help message", STYLER.command_style_alt("!help"));
                println!("  {:<20} - List all messages in the current session", STYLER.command_style_alt("!list"));
                println!("  {:<20} - Read a specific message by its number", STYLER.command_style_alt("!read <number>"));
                println!("  {:<20} - Clear the terminal screen", STYLER.command_style_alt("!clear | !cls"));
                println!("  {:<20} - Exit Prime", STYLER.command_style_alt("!exit | !quit"));
                println!();
                println!("{}", STYLER.header_style("Examples:"));
                println!("  {} Create a Python script that calculates prime numbers", STYLER.dim_gray_style("•"));
                println!("  {} What files are in my current directory?", STYLER.dim_gray_style("•"));
                println!("  {} Download the latest data from my API and save it as JSON", STYLER.dim_gray_style("•"));
                println!("  {} Create a backup of my important files", STYLER.dim_gray_style("•"));
                Ok(true)
            }
            "exit" | "quit" => {
                println!("{}", STYLER.info_style("Exiting Prime..."));
                Ok(false)
            }
            _ => {
                println!(
                    "{} Unknown command: {}{}. Type {} for available commands.",
                    STYLER.error_style("Error:"),
                    STYLER.command_style_alt("!"),
                    command,
                    STYLER.command_style_alt("!help")
                );
                Ok(true)
            }
        }
    }
}