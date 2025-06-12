// memory.rs
// Memory management for Prime

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{self, DateTime, Local, TimeZone, NaiveDateTime};

/// Manages Prime's memory files
pub struct MemoryManager {
    memory_dir: PathBuf,
    long_term_file: PathBuf,
    short_term_file: PathBuf,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(memory_dir: PathBuf) -> Self {
        let long_term_file = memory_dir.join("long_term.md");
        let short_term_file = memory_dir.join("short_term.md");
        
        Self {
            memory_dir,
            long_term_file,
            short_term_file,
        }
    }
    
    /// Initialize memory files if they don't exist
    pub fn initialize(&self) -> Result<()> {
        // Create memory directory if it doesn't exist
        if !self.memory_dir.exists() {
            fs::create_dir_all(&self.memory_dir)
                .context("Failed to create memory directory")?;
        }
        
        // Initialize long-term memory file if it doesn't exist
        if !self.long_term_file.exists() {
            fs::write(&self.long_term_file, "# Prime Long-term Memory\n\n")
                .context("Failed to initialize long-term memory file")?;
        }
        
        // Initialize short-term memory file if it doesn't exist
        if !self.short_term_file.exists() {
            fs::write(&self.short_term_file, "# Prime Short-term Memory\n\n")
                .context("Failed to initialize short-term memory file")?;
        }
        
        Ok(())
    }
    
    /// Add memory entry to the specified memory type
    pub fn add_memory(&self, memory_type: &str, category: &str, content: &str) -> Result<()> {
        let memory_file = match memory_type.to_lowercase().as_str() {
            "long" | "long_term" => &self.long_term_file,
            "short" | "short_term" => &self.short_term_file,
            _ => return Err(anyhow::anyhow!("Invalid memory type: {}", memory_type)),
        };
        
        // Ensure memory directory and files exist
        self.initialize()?;
        
        // Read existing memory content
        let mut memory_content = fs::read_to_string(memory_file)
            .context("Failed to read memory file")?;
        
        // Check if category exists, add it if not
        if !memory_content.contains(&format!("## {}", category)) {
            memory_content.push_str(&format!("\n## {}\n", category));
        }
        
        // Add entry with timestamp
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let entry = format!("- {} (added: {})\n", content, timestamp);
        
        // Find the category section and add the entry
        let mut updated_content = String::new();
        let mut in_category = false;
        let mut category_found = false;
        
        for line in memory_content.lines() {
            if line.starts_with(&format!("## {}", category)) {
                // Found the category
                in_category = true;
                category_found = true;
                updated_content.push_str(line);
                updated_content.push('\n');
                updated_content.push_str(&entry);
            } else if line.starts_with("## ") && in_category {
                // Found the next category, no longer in our target category
                in_category = false;
                updated_content.push_str(line);
                updated_content.push('\n');
            } else {
                // Regular line, just copy it
                updated_content.push_str(line);
                updated_content.push('\n');
            }
        }
        
        // If category wasn't found (shouldn't happen due to earlier check, but just in case)
        if !category_found {
            updated_content.push_str(&format!("\n## {}\n", category));
            updated_content.push_str(&entry);
        }
        
        // Write updated content back to file
        fs::write(memory_file, updated_content)
            .context("Failed to write updated memory file")?;
        
        Ok(())
    }
    
    /// Read memory content
    pub fn read_memory(&self, memory_type: Option<&str>) -> Result<String> {
        // Ensure memory files exist
        self.initialize()?;
        
        match memory_type {
            Some("long") | Some("long_term") => {
                fs::read_to_string(&self.long_term_file)
                    .context("Failed to read long-term memory file")
            },
            Some("short") | Some("short_term") => {
                fs::read_to_string(&self.short_term_file)
                    .context("Failed to read short-term memory file")
            },
            None | Some("all") => {
                // Combine both memory types
                let short_term = fs::read_to_string(&self.short_term_file)
                    .context("Failed to read short-term memory file")?;
                
                let long_term = fs::read_to_string(&self.long_term_file)
                    .context("Failed to read long-term memory file")?;
                
                Ok(format!("{}\n\n{}", short_term, long_term))
            },
            Some(other) => Err(anyhow::anyhow!("Invalid memory type: {}", other)),
        }
    }
    
    /// Clear short-term memory
    pub fn clear_short_term_memory(&self) -> Result<()> {
        fs::write(&self.short_term_file, "# Prime Short-term Memory\n\n")
            .context("Failed to clear short-term memory file")
    }
    
    /// Search memory for content
    pub fn search_memory(&self, query: &str, memory_type: Option<&str>) -> Result<Vec<MemoryEntry>> {
        let memory_content = self.read_memory(memory_type)?;
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        let mut current_category = String::new();
        let mut current_type = if memory_type.is_some() {
            memory_type.unwrap().to_string()
        } else {
            "all".to_string()
        };
        
        for line in memory_content.lines() {
            if line.starts_with("# Prime ") {
                // Memory type header
                if line.contains("Long-term") {
                    current_type = "long".to_string();
                } else if line.contains("Short-term") {
                    current_type = "short".to_string();
                }
            } else if line.starts_with("## ") {
                // Category header
                current_category = line.trim_start_matches("## ").to_string();
            } else if line.starts_with("- ") && line.to_lowercase().contains(&query_lower) {
                // Entry that matches query
                results.push(MemoryEntry {
                    memory_type: current_type.clone(),
                    category: current_category.clone(),
                    content: line.trim_start_matches("- ").to_string(),
                });
            }
        }
        
        Ok(results)
    }
    
    /// Get all categories from memory
    pub fn get_categories(&self, memory_type: Option<&str>) -> Result<Vec<String>> {
        let memory_content = self.read_memory(memory_type)?;
        let mut categories = Vec::new();
        
        for line in memory_content.lines() {
            if line.starts_with("## ") {
                categories.push(line.trim_start_matches("## ").to_string());
            }
        }
        
        // Remove duplicates
        categories.sort();
        categories.dedup();
        
        Ok(categories)
    }
    
    /// Get memory summary for LLM context
    pub fn get_memory_summary(&self) -> Result<String> {
        let mut summary = String::new();
        
        // Get categories and recent entries
        if let Ok(categories) = self.get_categories(None) {
            if !categories.is_empty() {
                summary.push_str("## Known Categories:\n");
                for cat in &categories {
                    summary.push_str(&format!("- {}\n", cat));
                }
                summary.push_str("\n");
            }
        }
        
        // Get recent entries from short-term memory
        if let Ok(short_term) = self.read_memory(Some("short")) {
            let entries: Vec<&str> = short_term.lines()
                .filter(|l| l.starts_with("- "))
                .rev()
                .take(5)
                .collect();
            
            if !entries.is_empty() {
                summary.push_str("## Recent Context:\n");
                for entry in entries.into_iter().rev() {
                    summary.push_str(&format!("{}\n", entry));
                }
            }
        }
        
        Ok(summary)
    }
}

/// Represents a memory entry
pub struct MemoryEntry {
    pub memory_type: String,
    pub category: String,
    pub content: String,
}

/// Memory utilities
pub mod memory_utils {
    use super::*;
    
    /// Extract a specific category from memory
    pub fn extract_category(memory_content: &str, category: &str) -> Option<String> {
        let mut result = String::new();
        let mut in_category = false;
        
        for line in memory_content.lines() {
            if line.starts_with(&format!("## {}", category)) {
                in_category = true;
                result.push_str(line);
                result.push('\n');
            } else if line.starts_with("## ") && in_category {
                // End of category
                break;
            } else if in_category {
                result.push_str(line);
                result.push('\n');
            }
        }
        
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
    
    /// Parse timestamp from memory entry
    pub fn parse_timestamp(entry: &str) -> Option<chrono::DateTime<chrono::Local>> {
        // Look for pattern (added: YYYY-MM-DD HH:MM:SS)
        if let Some(pos) = entry.rfind("(added: ") {
            let timestamp_str = &entry[pos + 8..];
            if let Some(end_pos) = timestamp_str.find(')') {
                let timestamp = &timestamp_str[..end_pos];
                return chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S")
                    .ok()
                    .and_then(|dt| chrono::Local.from_local_datetime(&dt).single());
            }
        }
        
        None
    }
    
    /// Get the content part of a memory entry
    pub fn get_entry_content(entry: &str) -> String {
        // Strip timestamp part
        if let Some(pos) = entry.rfind(" (added:") {
            entry[..pos].trim_start_matches('-').trim().to_string()
        } else {
            entry.trim_start_matches('-').trim().to_string()
        }
    }
}