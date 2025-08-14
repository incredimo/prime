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
    // Get configuration from environment variables with defaults
    let provider = env::var("LLM_PROVIDER").unwrap_or_else(|_| "google".to_string());
    let model = env::var("LLM_MODEL").unwrap_or_else(|_| {
        match provider.as_str() {
            "google" => "gemini-2.5-flash".to_string(),
            "ollama" => "gemma3".to_string(),
            _ => "gemma3".to_string(),
        }
    });
    let temperature = env::var("LLM_TEMPERATURE")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.2);
    let max_tokens = env::var("LLM_MAX_TOKENS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(2048);

    let prime_config_base_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".prime");

    let workspace_dir = env::current_dir().context("Failed to get current working directory")?;

    // Build LLM provider based on the selected provider
    let (llm, provider_name, api_key_name) = match provider.as_str() {
        "google" => {
            let api_key = env::var("GEMINI_API_KEY").context("GEMINI_API_KEY environment variable not set")?;
            let llm = LLMBuilder::new()
                .backend(LLMBackend::Google)
                .api_key(api_key.clone())
                .model(model.clone())
                .max_tokens(max_tokens)
                .temperature(temperature)
                .build()
                .context("Failed to build LLM provider (Google)")?;
            (llm, "Google AI Platform", "GEMINI_API_KEY")
        },
        "ollama" => {
            let api_key = env::var("OLLAMA_API_KEY").unwrap_or_default(); // Ollama might not need an API key
            let llm = LLMBuilder::new()
                .backend(LLMBackend::Ollama)
                .api_key(api_key.clone())
                .model(model.clone())
                .max_tokens(max_tokens)
                .temperature(temperature)
                .build()
                .context("Failed to build LLM provider (Ollama)")?;
            (llm, "Ollama", "OLLAMA_API_KEY")
        },
        _ => {
            return Err(anyhow::anyhow!("Unsupported LLM provider: {}", provider));
        }
    };

    console::display_init_info(&model, provider_name, &prime_config_base_dir, &workspace_dir);

    let session = PrimeSession::new(prime_config_base_dir, llm)?;

    Ok(session)
}
