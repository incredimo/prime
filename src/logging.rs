use chrono;
use std::path::PathBuf;
use std::fs::{self, OpenOptions};
use std::io::Write;
use anyhow::Result;
use serde_json::Value;
use once_cell::sync::Lazy;
use crate::styling::STYLER;

pub static LOG: Lazy<Logger> = Lazy::new(|| Logger::new());

pub struct Logger {
    debug_logger: DebugLogger,
}

impl Logger {
    fn new() -> Self {
        Self {
            debug_logger: DebugLogger::new(true, None, LogLevel::Info),
        }
    }

    // Command execution logging
    pub fn executing(&self, command: impl std::fmt::Display, pwd: impl std::fmt::Display) {
        println!("{}", STYLER.executing_command_style(pwd, command));
    }

    // Success messages
    pub fn success(&self, msg: impl std::fmt::Display) {
        println!(
            "{} {}",
            STYLER.success_style("[OK]"),
            STYLER.success_style(msg)
        );
    }

    // Error messages
    pub fn error(&self, msg: impl std::fmt::Display) {
        eprintln!(
            "{} {}",
            STYLER.error_style("[ERROR]"),
            STYLER.error_style(msg)
        );
    }

    // Warning messages
    pub fn warning(&self, msg: impl std::fmt::Display) {
        println!(
            "{} {}",
            STYLER.warning_style("[WARN]"),
            STYLER.warning_style(msg)
        );
    }

    // Info messages
    pub fn info(&self, msg: impl std::fmt::Display) {
        println!(
            "{} {}",
            STYLER.info_style("[INFO]"),
            STYLER.info_style(msg)
        );
    }

    // Command output preview
    pub fn command_output(&self, output: &str) {
        if !output.trim().is_empty() {
            let preview_lines = 5;
            let lines: Vec<&str> = output.lines().collect();
            let preview = lines.iter().take(preview_lines).cloned().collect::<Vec<&str>>().join("\n");
            
            println!("{}", STYLER.dim_gray_style("Output preview:"));
            println!("{}", STYLER.dim_gray_style(&preview));
            
            if lines.len() > preview_lines {
                println!(
                    "{}",
                    STYLER.dim_gray_style(format!(
                        "... ({} more lines, full output saved in conversation)",
                        lines.len() - preview_lines
                    ))
                );
            }
        }
    }

    // Header messages
    pub fn header(&self, msg: impl std::fmt::Display) {
        println!("{}", STYLER.header_style(msg));
    }

    // Command status
    pub fn command_status(&self, exit_code: i32) {
        println!(
            "{}",
            STYLER.dim_gray_style(format!(
                "Command completed with exit code: {}",
                exit_code
            ))
        );
    }
}

pub struct DebugLogger {
    enabled: bool,
    log_file: Option<PathBuf>,
    log_level: LogLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl DebugLogger {
    pub fn new(enabled: bool, log_file: Option<PathBuf>, log_level: LogLevel) -> Self {
        Self {
            enabled,
            log_file,
            log_level,
        }
    }
    
    pub fn log_llm_interaction(&self, prompt: &str, response: &str) -> Result<()> {
        if !self.enabled || self.log_level < LogLevel::Debug {
            return Ok(());
        }
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!(
            "=== {} ===\nPROMPT:\n{}\n\nRESPONSE:\n{}\n\n",
            timestamp, prompt, response
        );
        
        self.write_log(&log_entry)
    }
    
    pub fn log_command_execution(&self, command: &str, result: &(i32, String)) -> Result<()> {
        if !self.enabled || self.log_level < LogLevel::Debug {
            return Ok(());
        }
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!(
            "=== {} ===\nCOMMAND: {}\nEXIT CODE: {}\nOUTPUT:\n{}\n\n",
            timestamp, command, result.0, result.1
        );
        
        self.write_log(&log_entry)
    }
    
    pub fn log_error(&self, error: &anyhow::Error) -> Result<()> {
        if !self.enabled || self.log_level < LogLevel::Error {
            return Ok(());
        }
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!(
            "=== {} === ERROR:\n{:?}\n\n",
            timestamp, error
        );
        
        self.write_log(&log_entry)
    }
    
    pub fn log_step(&self, step: &str, data: Option<&Value>) -> Result<()> {
        if !self.enabled || self.log_level < LogLevel::Info {
            return Ok(());
        }
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let mut log_entry = format!("=== {} === STEP: {}\n", timestamp, step);
        
        if let Some(data) = data {
            log_entry.push_str(&format!("DATA: {}\n", data));
        }
        log_entry.push_str("\n");
        
        self.write_log(&log_entry)
    }
    
    fn write_log(&self, entry: &str) -> Result<()> {
        if let Some(ref path) = self.log_file {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            
            file.write_all(entry.as_bytes())?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_debug_logger() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let logger = DebugLogger::new(
            true,
            Some(temp_file.path().to_path_buf()),
            LogLevel::Debug
        );
        
        logger.log_step("Test step", None)?;
        logger.log_error(&anyhow::anyhow!("Test error"))?;
        
        let log_content = fs::read_to_string(temp_file.path())?;
        assert!(log_content.contains("Test step"));
        assert!(log_content.contains("Test error"));
        
        Ok(())
    }
}