mod styling;
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
mod memory;
mod terminal_ui;
mod config_utils;
pub mod web_ops;

use session::{PrimeSession, ProcessedItemResult};
use terminal_ui::PrimeHelper;
use crate::styling::STYLER;

const APP_NAME: &str = "prime";
const VERSION: &str = "1.0.0";
const BANNER: &str = r#"
   [38;2;230;230;230m██[0m[38;2;230;230;230m██[0m[38;2;230;230;230m██[0m [38;2;230;230;230m██[0m[38;2;63;81;181m██[0m[38;2;33;150;243m██[0m   [38;2;3;169;244m██[0m [38;2;0;150;136m██[0m[38;2;76;175;80m██[0m[38;2;205;220;57m██[0m[38;2;255;193;7m██[0m   [38;2;255;152;0m██[0m[38;2;255;87;34m██[0m[38;2;244;67;54m██[0m 
 [38;2;33;150;243m██[0m    [38;2;3;169;244m██[0m [38;2;0;150;136m██[0m    [38;2;76;175;80m██[0m [38;2;205;220;57m██[0m [38;2;255;193;7m██[0m  [38;2;255;152;0m██[0m  [38;2;255;87;34m██[0m [38;2;244;67;54m██[0m     
 [38;2;230;230;230m██[0m[38;2;230;230;230m██[0m[38;2;63;81;181m██[0m   [38;2;33;150;243m██[0m[38;2;3;169;244m██[0m[38;2;0;150;136m██[0m   [38;2;76;175;80m██[0m [38;2;205;220;57m██[0m  [38;2;255;193;7m██[0m  [38;2;255;152;0m██[0m [38;2;255;87;34m██[0m[38;2;244;67;54m██[0m   
 [38;2;63;81;181m██[0m       [38;2;33;150;243m██[0m    [38;2;3;169;244m██[0m [38;2;0;150;136m██[0m [38;2;76;175;80m██[0m  [38;2;205;220;57m██[0m  [38;2;255;193;7m██[0m [38;2;255;152;0m██[0m[38;2;255;87;34m██[0m[38;2;244;67;54m██[0m "#;

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
    
    let prime_config_base_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".prime");

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
        label_style.apply_to("configuration"),
        value_style.apply_to(&prime_config_base_dir.display().to_string())
    );
    println!(
        "  {} {:<18} {}",
        arrow_style.apply_to("•"),
        label_style.apply_to("workspace"),
        value_style.apply_to(&workspace_dir.display().to_string())
    );
    println!("{}", sep_style.apply_to(bar_char.repeat(70)));

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
        self.session.add_user_message(initial_input)?;
        let mut current_llm_prompt = initial_input.to_string();
        let mut recursion_depth = 0;
        const MAX_RECURSION_DEPTH: usize = 3;

        loop {
            if recursion_depth >= MAX_RECURSION_DEPTH {
                let err_msg = "Max correction attempts reached for this request.";
                eprintln!("{} {}", STYLER.error_style("[ERROR]"), STYLER.error_style(err_msg));
                self.session.add_system_message("InternalError", -1, err_msg)
                    .context("Failed to log max recursion depth error")?;
                return Err(anyhow::anyhow!(err_msg));
            }

            let llm_response = match self.session.generate_prime_response_stream(
                &current_llm_prompt,
                recursion_depth > 0,
            ).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{} {}", STYLER.error_style("[ERROR]"), STYLER.error_style(format!("Failed to generate LLM response: {}", e)));
                    return Err(e);
                }
            };
            
            println!("\n{}", STYLER.separator_style("-".repeat(70)));

            match self.session.process_commands(&llm_response).await {
                Ok(processed_results) if processed_results.is_empty() => {
                    println!("{}", STYLER.info_style("LLM provided no actions. Task may be complete or require no further steps."));
                    return Ok(());
                }
                Ok(processed_results) => {
                    let mut all_succeeded = true;
                    let mut failure_details_for_llm = String::new();

                    for result_item in processed_results {
                        match result_item {
                            ProcessedItemResult::Command(cmd_res) => {
                                if !cmd_res.success {
                                    all_succeeded = false;
                                    failure_details_for_llm.push_str(&format!(
                                        "Command:\n```\n{}\n```\nFailed with exit code {}.\nOutput:\n```\n{}\n```\n\n",
                                        cmd_res.command,
                                        cmd_res.exit_code,
                                        cmd_res.output
                                    ));
                                }
                            }
                            ProcessedItemResult::FileOp(file_op_res) => {
                                if !file_op_res.success {
                                    all_succeeded = false;
                                    failure_details_for_llm.push_str(&format!(
                                        "File Operation:\nAction: {}\nPath: {}\nFailed.\nDetails:\n```\n{}\n```\n\n",
                                        file_op_res.action, file_op_res.path, file_op_res.output
                                    ));
                                }
                            }
                            ProcessedItemResult::WebOp(web_op_res) => {
                                if !web_op_res.success {
                                    all_succeeded = false;
                                    failure_details_for_llm.push_str(&format!(
                                        "Web Operation:\nAction: {}\nURL: {}\nFailed.\nDetails:\n```\n{}\n```\n\n",
                                        web_op_res.action, web_op_res.url, web_op_res.output
                                    ));
                                }
                            }
                        }
                    }

                    if all_succeeded {
                        println!("{}", STYLER.success_style("All actions executed successfully."));
                        return Ok(());
                    } else {
                        recursion_depth += 1;
                        println!(
                            "{} Some actions failed. Attempting to correct (Attempt {}/{})...",
                            STYLER.warning_style("[WARN]"), recursion_depth, MAX_RECURSION_DEPTH
                        );
                        current_llm_prompt = format!(
                            "The previous set of actions resulted in errors. Analyze the failures and provide corrected actions or an alternative approach. Ensure actions are in the correct Pandoc-attributed markdown format.\n\nThe prompt that led to the failed actions was:\n---\n{}\n---\n\nFailed action details:\n{}",
                            current_llm_prompt,
                            failure_details_for_llm
                        );
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", STYLER.error_style("[ERROR]"), STYLER.error_style(format!("Internal error during action processing: {}", e)));
                    return Err(e);
                }
            }

            if recursion_depth > 0 && recursion_depth < MAX_RECURSION_DEPTH {
                println!("\n{}", STYLER.separator_style("-".repeat(70)));
            }
        }
    }

    fn handle_special_command(&self, cmd_line: &str) -> Result<bool> {
        let parts: Vec<&str> = cmd_line.splitn(2, ' ').collect();
        let command = parts[0].to_lowercase();
        let args = parts.get(1).copied().unwrap_or("").trim();

        match command.as_str() {
            "memory" => {
                let memory_type = if args.is_empty() { None } else { Some(args) };
                match self.session.read_memory(memory_type) {
                    Ok(mem) => println!("{}\n{}", STYLER.header_style(format!("Memory ({}):", memory_type.unwrap_or("all"))), mem),
                    Err(e) => eprintln!("{} {}", STYLER.error_style("Error reading memory:"), e),
                }
                Ok(true)
            }
            "clear" | "cls" => {
                print!("\x1B[2J\x1B[1;1H");
                std::io::stdout().flush()?;
                Ok(true)
            }
            "list" => {
                match self.session.list_messages() {
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
                    match self.session.read_message(num) {
                        Ok(msg) => println!("{}", msg),
                        Err(e) => eprintln!("{} {}", STYLER.error_style(format!("Error reading message {}:", num)), e),
                    }
                } else {
                    eprintln!("{} Invalid message number: {}", STYLER.error_style("Error:"), args);
                }
                Ok(true)
            }
            "help" => {
                println!("{}", STYLER.header_style("Available Special Commands:"));
                println!("  {:<20} - Show this help message.", STYLER.command_style_alt("!help").to_string());
                println!("  {:<20} - Show memory (type: short, long, all. Default: all).", STYLER.command_style_alt("!memory [type]").to_string());
                println!("  {:<20} - List messages in the current session.", STYLER.command_style_alt("!list").to_string());
                println!("  {:<20} - Read a specific message by its number.", STYLER.command_style_alt("!read <number>").to_string());
                println!("  {:<20} - Clear the terminal screen.", STYLER.command_style_alt("!clear | !cls").to_string());
                println!("  {:<20} - Exit Prime.", STYLER.command_style_alt("!exit | !quit").to_string());
                Ok(true)
            }
            "exit" | "quit" => {
                println!("{}", STYLER.info_style("Exiting Prime..."));
                Ok(false)
            }
            _ => {
                println!("{} Unknown command: !{}. Type {} for available commands.", 
                    STYLER.error_style("Error:"), 
                    command, 
                    STYLER.command_style_alt("!help")
                );
                Ok(true)
            }
        }
    }
}
