mod styling;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use anyhow::{Context as AnyhowContext, Result};
use rustyline::history::DefaultHistory;
use rustyline::{Editor};
use rustyline::error::ReadlineError;

mod session;
mod commands;

use session::{PrimeSession, ProcessingSessionResult};
use styling::{PrimeHelper, STYLER, BANNER, APP_NAME};

#[tokio::main]
async fn main() -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    STYLER.print_header(&version);

    let prime = match init_prime().await {
        Ok(prime) => prime,
        Err(e) => {
            STYLER.errorln(format!("Initialization error: {}", e));
            process::exit(1);
        }
    };

    match prime.run().await {
        Ok(_) => {
            STYLER.successln("Prime session ended successfully");
            Ok(())
        }
        Err(e) => {
            STYLER.errorln(format!("Runtime error: {}", e));
            process::exit(1);
        }
    }
}

async fn init_prime() -> Result<Prime> {
    let ollama_model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gemma3:latest".to_string());
    let ollama_api = env::var("OLLAMA_API").unwrap_or_else(|_| "http://localhost:11434".to_string());
    
    let workspace_dir = env::current_dir().context("Failed to get current working directory")?;

    STYLER.print_config(&ollama_model, &ollama_api, &workspace_dir.display());

    let session = PrimeSession::new(workspace_dir.clone(), &ollama_model, &ollama_api)?;
    Ok(Prime { session: Arc::new(session), workspace_dir })
}

pub struct Prime {
    session: Arc<PrimeSession>,
    #[allow(dead_code)]
    workspace_dir: PathBuf,
}

impl Prime {
    pub async fn run(&self) -> Result<()> {
        let mut editor = Editor::<PrimeHelper, DefaultHistory>::new()?;
        let prime_helper = PrimeHelper::new(&styling::APP_NAME);
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
                        STYLER.infoln("Exiting Prime...");
                        break;
                    }
                    if input.starts_with('!') {
                        if !self.handle_special_command(&input[1..])? {
                            break;
                        }
                        continue;
                    }

                    if let Err(e) = self.process_user_input(input).await {
                        STYLER.errorln(format!("Error processing input: {}", e));
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    STYLER.warningln("Interrupted. Type 'exit' or Ctrl-D to quit.");
                }
                Err(ReadlineError::Eof) => {
                    STYLER.infoln("End of input. Goodbye!");
                    break;
                }
                Err(err) => {
                    STYLER.errorln(format!("Input error: {}", err));
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
        
        // Add the initial user message
        self.session.add_user_message(initial_input)?;
        let mut current_prompt = initial_input.to_string();
        let mut iteration_count = 0;
        const MAX_ITERATIONS: usize = 10; // Safety limit to prevent infinite loops
        
        loop {
            iteration_count += 1;
            
            if iteration_count > MAX_ITERATIONS {
                let error_msg = "Maximum iteration limit reached. Please try a simpler request.";
                STYLER.errorln(error_msg);
                self.session.add_system_message("MaxIterations", "FAILED", error_msg)?;
                break;
            }
            
            // Generate LLM response
            STYLER.print_separator();
            let llm_response = match self.session.generate_prime_response_stream(&current_prompt).await {
                Ok(response) => response,
                Err(e) => {
                    STYLER.errorln(format!("Failed to generate response: {}", e));
                    return Err(e);
                }
            };
            
            // Process script blocks in the response
            STYLER.print_separator();
            let processing_result: ProcessingSessionResult = match self.session.process_commands(&llm_response).await {
                Ok(result) => result,
                Err(e) => {
                    STYLER.errorln(format!("Error processing script blocks: {}", e));
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
                STYLER.infoln("No script blocks found. Conversation complete.");
                break;
            } else {
                // Script blocks were executed - send results back to LLM
                let all_successful = processing_result.script_results.iter().all(|r| r.success);
                
                if all_successful {
                    STYLER.infoln("Sending execution results back to Prime...");
                } else {
                    STYLER.warningln("Some operations failed. Sending error details to Prime for correction...");
                }
                
                // Prepare the next prompt with execution results
                current_prompt = format!(
                    "EXECUTION RESULTS:\n\n{}\n\nPlease analyze these results and provide your next response. If the task is complete, use completed=\"true\" in your final script block.",
                    processing_result.execution_summary
                );
                
                // Add the execution results as a user message for continuity
                self.session.add_user_message(&current_prompt)?;
            }
        }
        
        STYLER.print_separator();
        Ok(())
    }

    fn handle_special_command(&self, cmd_line: &str) -> Result<bool> {
        crate::commands::handle_special_command(self.session.as_ref(), cmd_line)
    }
}