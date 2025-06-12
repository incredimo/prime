mod styling;
mod logging;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use anyhow::{Context as AnyhowContext, Result};
use colored::Colorize;
use console::Style;
use rustyline::history::DefaultHistory;
use rustyline::{Editor};
use rustyline::error::ReadlineError;

mod session;
mod commands;
mod terminal_ui;
mod config_utils;
mod memory;
mod environment;

use session::{PrimeSession, ProcessingSessionResult};
use terminal_ui::PrimeHelper;
use crate::styling::STYLER;
use crate::logging::LOG;

const APP_NAME: &str = "prime";
const VERSION: &str = "2.0.0";
const MAX_ITERATIONS: usize = 5;
const MAX_FAILURES_BEFORE_HELP: usize = 2;

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
            LOG.error(format!("Initialization error: {}", e));
            process::exit(1);
        }
    };

    match prime.run().await {
        Ok(_) => {
            LOG.success("Prime session ended successfully");
            Ok(())
        }
        Err(e) => {
            LOG.error(format!("Runtime error: {}", e));
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
    
    // Initialize and display environment info
    let session = PrimeSession::new(workspace_dir.clone(), &ollama_model, &ollama_api)?;
    let env_info = session.detect_environment();
    
    println!(
        "  {} {:<18} {}",
        arrow_style.apply_to("•"),
        label_style.apply_to("python"),
        value_style.apply_to(&env_info.python_version.unwrap_or_else(|| "Not found".to_string()))
    );
    println!(
        "  {} {:<18} {}",
        arrow_style.apply_to("•"),
        label_style.apply_to("virtual env"),
        value_style.apply_to(if env_info.in_venv { "Yes" } else { "No" })
    );
    
    println!("{}", sep_style.apply_to(bar_char.repeat(70)));

    Ok(Prime { 
        session: Arc::new(session), 
        workspace_dir,
        failure_count: 0,
        current_task_iterations: 0,
    })
}

pub struct Prime {
    session: Arc<PrimeSession>,
    workspace_dir: PathBuf,
    failure_count: usize,
    current_task_iterations: usize,
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
                        LOG.info("Exiting Prime...");
                        break;
                    }
                    if input.starts_with('!') {
                        if !self.handle_special_command(&input[1..])? {
                            break;
                        }
                        continue;
                    }

                    // Reset counters for new task
                    let mut prime_mut = self.clone_for_task();
                    prime_mut.failure_count = 0;
                    prime_mut.current_task_iterations = 0;
                    
                    if let Err(e) = prime_mut.process_user_input(input).await {
                        LOG.error(format!("Error processing input: {}", e));
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    LOG.warning("Interrupted. Type 'exit' or Ctrl-D to quit.");
                }
                Err(ReadlineError::Eof) => {
                    LOG.info("End of input. Goodbye!");
                    break;
                }
                Err(err) => {
                    LOG.error(format!("Input error: {}", err));
                    break;
                }
            }
        }
        Ok(())
    }

    fn clone_for_task(&self) -> Prime {
        Prime {
            session: Arc::clone(&self.session),
            workspace_dir: self.workspace_dir.clone(),
            failure_count: 0,
            current_task_iterations: 0,
        }
    }

    async fn process_user_input(&mut self, initial_input: &str) -> Result<()> {
        if initial_input.trim().is_empty() {
            return Ok(());
        }
        
        // Add the initial user message
        self.session.add_user_message(initial_input)?;
        let mut current_prompt = initial_input.to_string();
        let mut previous_result: Option<ProcessingSessionResult> = None;
        
        loop {
            self.current_task_iterations += 1;
            
            if self.current_task_iterations > MAX_ITERATIONS {
                let error_msg = "Maximum iteration limit reached. The task might be too complex. Try breaking it down into smaller steps.";
                LOG.error(error_msg);
                self.session.add_system_message("MaxIterations", "FAILED", error_msg)?;
                break;
            }
            
            // Check if we should ask for help
            if self.failure_count >= MAX_FAILURES_BEFORE_HELP {
                println!("\n{} Multiple attempts have failed. Would you like to:", 
                    STYLER.warning_style("⚠"));
                println!("  1. Let Prime try a different approach");
                println!("  2. Provide additional context");
                println!("  3. Skip this task");
                println!("  4. Continue trying");
                
                print!("Choose (1-4): ");
                io::stdout().flush()?;
                
                let mut choice = String::new();
                io::stdin().read_line(&mut choice)?;
                
                match choice.trim() {
                    "1" => {
                        current_prompt = "The previous approaches failed. Please try a completely different method.".to_string();
                        self.failure_count = 0; // Reset failure count
                    },
                    "2" => {
                        print!("Additional context: ");
                        io::stdout().flush()?;
                        let mut context = String::new();
                        io::stdin().read_line(&mut context)?;
                        current_prompt = context.trim().to_string();
                        self.failure_count = 0;
                    },
                    "3" => {
                        println!("{}", STYLER.info_style("Skipping task."));
                        break;
                    },
                    _ => {
                        // Continue with current approach
                    }
                }
            }
            
            // Generate LLM response with context-aware prompt
            println!("\n{}", STYLER.separator_style("─".repeat(70)));
            let prompt = self.session.build_context_aware_prompt(&current_prompt, previous_result.as_ref());
            let llm_response = match self.session.generate_prime_response_stream(&prompt).await {
                Ok(response) => response,
                Err(e) => {
                    LOG.error(format!("Failed to generate response: {}", e));
                    return Err(e);
                }
            };
            
            // Process script blocks in the response
            println!("\n{}", STYLER.separator_style("─".repeat(70)));
            let processing_result = match self.session.process_commands(&llm_response).await {
                Ok(result) => result,
                Err(e) => {
                    LOG.error(format!("Error processing script blocks: {}", e));
                    return Err(e);
                }
            };
            
            // Update failure count
            let current_iteration_failed = processing_result.script_results.iter().any(|r| !r.success);
            if current_iteration_failed {
                self.failure_count += 1;
            } else {
                self.failure_count = 0; // Reset on success
            }
            
            // Determine next action based on processing results
            if processing_result.has_completed {
                // Task is marked as completed
                if let Some(final_msg) = processing_result.final_message {
                    LOG.success(format!("Task completed: {}", final_msg));
                } else {
                    LOG.success("Task completed successfully");
                }
                break;
            } else if processing_result.script_results.is_empty() {
                // No script blocks found - check if it's a question or conversation end
                if llm_response.ends_with('?') {
                    LOG.info("Waiting for your response...");
                    break; // Let user provide input
                } else {
                    LOG.info("Response complete.");
                    break;
                }
            } else {
                // Script blocks were executed - prepare feedback
                let all_successful = processing_result.script_results.iter().all(|r| r.success);
                
                if all_successful {
                    LOG.info("Sending execution results back to Prime...");
                } else {
                    LOG.warning("Some operations failed. Sending error details to Prime for correction...");
                }
                
                // Prepare focused error feedback
                current_prompt = self.create_concise_feedback(&processing_result);
                
                // Store result for next iteration
                previous_result = Some(processing_result);
                
                // Add the execution results as a user message for continuity
                self.session.add_user_message(&current_prompt)?;
            }
        }
        
        println!("\n{}", STYLER.separator_style("─".repeat(70)));
        Ok(())
    }
    
    fn create_concise_feedback(&self, result: &ProcessingSessionResult) -> String {
        let mut feedback = String::new();
        
        // Focus on failures with key information only
        for script_result in &result.script_results {
            if !script_result.success {
                if let Some(exit_code) = script_result.exit_code {
                    let key_error = self.session.extract_key_error(&script_result.output);
                    feedback.push_str(&format!(
                        "Command failed (exit {}): {}\n", 
                        exit_code, 
                        key_error
                    ));
                }
            }
        }
        
        if feedback.is_empty() {
            // All successful
            feedback.push_str("All operations completed successfully. Continue with next step.");
        } else {
            feedback.push_str("\nProvide a simple fix for the above error.");
        }
        
        feedback
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
                match self.session.list_messages() {
                    Ok(list) => {
                        LOG.header("Conversation History:");
                        if list.is_empty() {
                            LOG.info("  No messages yet.");
                        } else {
                            for item in list {
                                println!("  {}", item);
                            }
                        }
                    }
                    Err(e) => LOG.error(format!("Error listing messages: {}", e)),
                }
                Ok(true)
            }
            "read" => {
                if args.is_empty() {
                    LOG.error("Usage: !read <message_number>");
                } else if let Ok(num) = args.parse::<usize>() {
                    match self.session.read_message(num) {
                        Ok(msg) => println!("{}", msg),
                        Err(e) => LOG.error(format!("Error reading message {}: {}", num, e)),
                    }
                } else {
                    LOG.error(format!("Invalid message number: {}", args));
                }
                Ok(true)
            }
            "env" => {
                let env_info = self.session.detect_environment();
                LOG.header("Environment Information:");
                println!("  {:<20} {}", STYLER.info_style("OS:"), env_info.os);
                println!("  {:<20} {}", STYLER.info_style("Python:"), env_info.python_version.unwrap_or_else(|| "Not found".to_string()));
                println!("  {:<20} {}", STYLER.info_style("Pip:"), env_info.pip_version.unwrap_or_else(|| "Not found".to_string()));
                println!("  {:<20} {}", STYLER.info_style("Virtual Env:"), if env_info.in_venv { "Yes" } else { "No" });
                println!("  {:<20} {}", STYLER.info_style("Git:"), if env_info.has_git { "Yes" } else { "No" });
                println!("  {:<20} {}", STYLER.info_style("NPM:"), if env_info.has_npm { "Yes" } else { "No" });
                Ok(true)
            }
            "memory" => {
                if let Some(memory_type) = match args {
                    "short" => Some("short"),
                    "long" => Some("long"),
                    "all" | "" => Some("all"),
                    _ => None,
                } {
                    match self.session.memory_manager.read_memory(Some(memory_type)) {
                        Ok(content) => println!("{}", content),
                        Err(e) => LOG.error(format!("Error reading memory: {}", e)),
                    }
                } else {
                    LOG.error("Usage: !memory [short|long|all]");
                }
                Ok(true)
            }
            "help" => {
                println!("{}", STYLER.header_style("Prime Assistant v2.0 - Smart Terminal AI"));
                println!();
                println!("{}", STYLER.info_style("Prime helps you accomplish tasks by executing code and managing files."));
                println!("{}", STYLER.info_style("Simply describe what you want to do, and Prime will create and execute the necessary scripts."));
                println!();
                LOG.header("Available Special Commands:");
                println!("  {:<20} - Show this help message", STYLER.command_style_alt("!help"));
                println!("  {:<20} - List all messages in the current session", STYLER.command_style_alt("!list"));
                println!("  {:<20} - Read a specific message by its number", STYLER.command_style_alt("!read <number>"));
                println!("  {:<20} - Show environment information", STYLER.command_style_alt("!env"));
                println!("  {:<20} - View memory [short|long|all]", STYLER.command_style_alt("!memory [type]"));
                println!("  {:<20} - Clear the terminal screen", STYLER.command_style_alt("!clear | !cls"));
                println!("  {:<20} - Exit Prime", STYLER.command_style_alt("!exit | !quit"));
                println!();
                println!("{}", STYLER.header_style("Examples:"));
                println!("  {} Install a Python package", STYLER.dim_gray_style("•"));
                println!("  {} Create a new project structure", STYLER.dim_gray_style("•"));
                println!("  {} Download and process data", STYLER.dim_gray_style("•"));
                println!("  {} Set up a development environment", STYLER.dim_gray_style("•"));
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