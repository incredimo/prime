//! Entry point for Prime CLI
//! v0.2.5: Enhanced with streaming responses and rich display

mod commands;
mod config;
mod console;
mod memory;
mod session;
mod parser;
mod streaming;
mod display;

use std::env;
use std::process;

use anyhow::{Context as AnyhowContext, Result};
use crossterm::style::Stylize;
use llm::builder::{LLMBackend, LLMBuilder};
use session::PrimeSession;
use crate::config::Config;

pub const APP_NAME: &str = "prime";

#[tokio::main]
async fn main() -> Result<()> {
    console::display_banner();

    let config = match config::load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{}", format!("[ERROR] Failed to load configuration: {}", e).red());
            process::exit(1);
        }
    };

    let session = match init_session(config).await {
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

async fn init_session(config: Config) -> Result<PrimeSession> {
    let provider = env::var("LLM_PROVIDER").unwrap_or(config.provider);
    let model_from_env = env::var("LLM_MODEL").ok();
    
    let model = model_from_env.or(config.model).unwrap_or_else(|| {
        match provider.as_str() {
            "google" => "gemini-2.5-flash-lite".to_string(),
            "ollama" => "gemma2".to_string(),
            _ => "gemma2".to_string(),
        }
    });

    let temperature = env::var("LLM_TEMPERATURE")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(config.temperature);
        
    let max_tokens = env::var("LLM_MAX_TOKENS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(config.max_tokens);

    let prime_config_base_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".prime");

    let workspace_dir = env::current_dir().context("Failed to get current working directory")?;

    let (llm, provider_name) = match provider.as_str() {
        "google" => {
            let api_key = env::var("GEMINI_API_KEY").unwrap_or(config.gemini_api_key);
            if api_key.is_empty() {
                return Err(anyhow::anyhow!("GEMINI_API_KEY not set in environment or config.toml. Please get a key from Google AI Studio."));
            }
            let llm = LLMBuilder::new()
                .backend(LLMBackend::Google)
                .api_key(api_key)
                .model(model.clone())
                .max_tokens(max_tokens)
                .temperature(temperature)
                .build()
                .context("Failed to build LLM provider (Google)")?;
            (llm, "Google AI Platform")
        },
        "ollama" => {
            let api_key = env::var("OLLAMA_API_KEY").unwrap_or(config.ollama_api_key);
            let llm = LLMBuilder::new()
                .backend(LLMBackend::Ollama)
                .api_key(api_key)
                .model(model.clone())
                .max_tokens(max_tokens)
                .temperature(temperature)
                .build()
                .context("Failed to build LLM provider (Ollama)")?;
            (llm, "Ollama")
        },
        _ => {
            return Err(anyhow::anyhow!("Unsupported LLM provider: {}", provider));
        }
    };

    console::display_init_info(&model, provider_name, &prime_config_base_dir, &workspace_dir);

    let session = PrimeSession::new(prime_config_base_dir, llm)?;

    Ok(session)
}