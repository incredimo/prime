use chrono;
use std::path::PathBuf;
use std::fs::{self, OpenOptions};
use std::io::Write;
use anyhow::Result;
use serde_json::Value;

pub struct DebugLogger {
    enabled: bool,
    log_file: Option<PathBuf>,
    log_level: LogLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        
        // Also print to stderr for immediate visibility
        eprintln!("{}", entry);
        
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