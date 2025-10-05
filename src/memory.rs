use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use chrono::Utc;
 

/// Manages long-term and short-term memory for the assistant
#[derive(Debug, Clone)]
pub struct MemoryManager {
    memory_dir: PathBuf,
}

impl MemoryManager {
    /// Creates a new MemoryManager, ensuring required files exist.
    pub fn new(memory_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&memory_dir)
            .with_context(|| format!("Failed to create memory directory at {}", memory_dir.display()))?;

        for mem_file in ["long_term.md", "short_term.md"] {
            let file_path = memory_dir.join(mem_file);
            if !file_path.exists() {
                let header = format!("# Prime {} Memory\n\n(This file is for notes. The AI will read this.)",
                    if mem_file == "long_term.md" { "Long-term" } else { "Short-term" });
                fs::write(&file_path, header)
                    .with_context(|| format!("Failed to create initial memory file at {}", file_path.display()))?;
            }
        }
        Ok(Self { memory_dir })
    }

    /// Reads memory content from the specified file (or both if none specified)
    pub fn read_memory(&self, memory_type: Option<&str>) -> Result<String> {
        let mut memory_content = String::new();
        match memory_type {
            Some("long_term") => {
                let content = self.read_file("long_term.md")?;
                memory_content.push_str("## Long-term Memory\n");
                memory_content.push_str(&content);
            }
            Some("short_term") => {
                let content = self.read_file("short_term.md")?;
                memory_content.push_str("## Short-term Memory\n");
                memory_content.push_str(&content);
            }
            None => {
                let long_term = self.read_file("long_term.md").unwrap_or_default();
                let short_term = self.read_file("short_term.md").unwrap_or_default();
                memory_content.push_str("\n<LONG_TERM_MEMORY>\n");
                memory_content.push_str(long_term.trim());
                memory_content.push_str("\n</LONG_TERM_MEMORY>\n");
                memory_content.push_str("\n<SHORT_TERM_MEMORY>\n");
                memory_content.push_str(short_term.trim());
                memory_content.push_str("\n</SHORT_TERM_MEMORY>\n");
            }
            Some(other) => return Err(anyhow!("Invalid memory type '{}' specified", other)),
        }
        Ok(memory_content)
    }
    
    /// Writes content to the specified memory type
    pub fn write_memory(&self, memory_type: &str, content: &str) -> Result<()> {
        let file_name = match memory_type {
            "long_term" => "long_term.md",
            "short_term" => "short_term.md",
            _ => return Err(anyhow!("Invalid memory type '{}' specified", memory_type)),
        };
        
        let file_path = self.memory_dir.join(file_name);
        let timestamp = Utc::now();
        let entry = format!("\n## Entry ({})\n{}\n", timestamp, content);
        
        fs::OpenOptions::new()
            .append(true)
            .open(&file_path)
            .with_context(|| format!("Failed to open memory file for writing: {}", file_path.display()))
            .and_then(|mut file| {
                file.write_all(entry.as_bytes())
                    .map_err(|e| anyhow::anyhow!("Failed to write to memory file: {}", e))
            })
            .with_context(|| format!("Failed to write to memory file: {}", file_path.display()))
    }
    
    /// Clears the specified memory type
    pub fn clear_memory(&self, memory_type: &str) -> Result<()> {
        let file_name = match memory_type {
            "long_term" => "long_term.md",
            "short_term" => "short_term.md",
            _ => return Err(anyhow!("Invalid memory type '{}' specified", memory_type)),
        };
        
        let file_path = self.memory_dir.join(file_name);
        let header = format!("# Prime {} Memory\n\n(This file is for notes. The AI will read this.)",
            if file_name == "long_term.md" { "Long-term" } else { "Short-term" });
        
        fs::write(&file_path, header)
            .with_context(|| format!("Failed to clear memory file: {}", file_path.display()))
    }
    
    /// Helper to read a specific memory file
    fn read_file(&self, file_name: &str) -> Result<String> {
        let file_path = self.memory_dir.join(file_name);
        fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read memory file: {}", file_path.display()))
    }
}