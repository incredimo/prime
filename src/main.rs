#![allow(warnings)]
//! Entry point for Prime CLI
//! -------------------------
//! Adjustments in v0.1.8:
//! * LLM temperature lowered to `0.2` for more deterministic action plans.
//! * Cargo version bump is reflected in the banner.
//! * Pulled `serde_json` into crate graph implicitly via `session.rs`.

mod commands;
mod config;
mod console;
mod memory;
mod session;
mod parser;

use std::env;
use std::path::PathBuf;
use std::process;

use anyhow::{Context as AnyhowContext, Result};
use crossterm::style::Stylize;
use llm::builder::{LLMBackend, LLMBuilder};
use session::PrimeSession;

pub const APP_NAME: &str = "prime";

#[tokio::main]
async fn main() -> Result<()> {
    console::display_banner();

    let session = match init_session().await {
        Ok(session) => session,
        Err(e) => {
            eprintln!("{}", format!("[ERROR] Initialization error: {}", e).red());
            process::exit(1);
        }
    };

    if let Err(e) = console::run_repl(session).await {
        eprintln!("{}", format!("[ERROR] Session ended with an error: {}", e).red());
        process::exit(1);
    }

    Ok(())
}

async fn init_session() -> Result<PrimeSession> {
    // Pull GEMINI key from env – could be extended to support multiple providers.
    let google_api_key = env::var("GEMINI_API_KEY").context("GEMINI_API_KEY environment variable not set")?;
    let google_model = "gemini-1.5-flash-latest";

    let prime_config_base_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".prime");

    let workspace_dir = env::current_dir().context("Failed to get current working directory")?;

    console::display_init_info(google_model, "Google AI Platform", &prime_config_base_dir, &workspace_dir);

    let llm = LLMBuilder::new()
        .backend(LLMBackend::Google)
        .api_key(google_api_key)
        .model(google_model)
        .max_tokens(2048)
        .temperature(0.2) // lower temp = fewer hallucinated loops
        .build()
        .context("Failed to build LLM provider (Google)")?;

    let session = PrimeSession::new(prime_config_base_dir, llm)?;

    Ok(session)
}
