use std::collections::HashMap;
use anyhow::{Result, anyhow};

use crate::session::ScriptBlock;

pub struct MarkdownParser;

impl MarkdownParser {
    /// Parse markdown content and extract script blocks with validation
    pub fn parse_script_blocks(&self, markdown: &str) -> Result<Vec<ScriptBlock>> {
        let lines: Vec<&str> = markdown.lines().collect();
        let mut script_blocks = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();
            
            // Look for code block start with script class
            if line.starts_with("```{.script") || line.starts_with("```{.text") || line.starts_with("```{.powershell") {
                if let Some(script_block) = Self::parse_single_block(&lines, i) {
                    // Validate the block before adding it
                    Self::validate_script_block(&script_block.0)?;
                    script_blocks.push(script_block.0);
                    i = script_block.1; // Jump to end of this block
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        Ok(script_blocks)
    }

    /// Parse a single script block starting at the given line index
    fn parse_single_block(lines: &[&str], start_idx: usize) -> Option<(ScriptBlock, usize)> {
        let start_line = lines[start_idx].trim();
        
        // Parse attributes from the opening line
        let attributes = Self::parse_attributes(start_line);
        
        // Find the content between ``` markers
        let mut content_lines = Vec::new();
        let mut end_idx = start_idx + 1;
        
        while end_idx < lines.len() {
            let line = lines[end_idx];
            if line.trim() == "```" {
                break;
            }
            content_lines.push(line);
            end_idx += 1;
        }
        
        if end_idx >= lines.len() {
            // No closing ``` found
            return None;
        }
        
        let content = content_lines.join("\n");
        
        Some((ScriptBlock {
            attributes,
            content,
            step_number: None,
            depends_on: None,
        }, end_idx + 1))
    }

    /// Parse attributes from opening line like ```{.script attr1="value1" attr2="value2"}
    fn parse_attributes(line: &str) -> HashMap<String, String> {
        let mut attributes = HashMap::new();
        
        // Find the content between { and }
        if let Some(start) = line.find('{') {
            if let Some(end) = line.rfind('}') {
                let attr_str = &line[start + 1..end];
                
                // Remove the class part (e.g., ".script", ".text", ".powershell")
                let attr_str = attr_str.split_whitespace()
                    .skip_while(|s| s.starts_with('.'))
                    .collect::<Vec<&str>>()
                    .join(" ");
                
                let mut chars: Vec<char> = attr_str.chars().collect();
                let mut i = 0;
                
                while i < chars.len() {
                    // Skip whitespace
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    
                    if i >= chars.len() {
                        break;
                    }
                    
                    // Read key
                    let mut key = String::new();
                    while i < chars.len() && chars[i] != '=' && !chars[i].is_whitespace() {
                        key.push(chars[i]);
                        i += 1;
                    }
                    
                    // Skip whitespace and '='
                    while i < chars.len() && (chars[i].is_whitespace() || chars[i] == '=') {
                        i += 1;
                    }
                    
                    if i >= chars.len() {
                        break;
                    }
                    
                    // Read value
                    let mut value = String::new();
                    if chars[i] == '"' {
                        // Quoted value - handle escaped quotes
                        i += 1; // Skip opening quote
                        while i < chars.len() && chars[i] != '"' {
                            if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == '"' {
                                value.push('"');
                                i += 2; // Skip escaped quote
                            } else {
                                value.push(chars[i]);
                                i += 1;
                            }
                        }
                        if i < chars.len() {
                            i += 1; // Skip closing quote
                        }
                    } else {
                        // Unquoted value
                        while i < chars.len() && !chars[i].is_whitespace() {
                            value.push(chars[i]);
                            i += 1;
                        }
                    }
                    
                    if !key.is_empty() && !value.is_empty() {
                        attributes.insert(key, value);
                    }
                }
            }
        }
        
        attributes
    }

    /// Validate a script block's attributes and content
    fn validate_script_block(block: &ScriptBlock) -> Result<()> {
        // Check for conflicting attributes
        if block.attributes.contains_key("save") && block.attributes.contains_key("patch") {
            return Err(anyhow!("Cannot use both 'save' and 'patch' in same block"));
        }
        
        // Validate paths in attributes
        for (key, value) in &block.attributes {
            match key.as_str() {
                "save" | "path" => {
                    if value.contains("..") || value.starts_with("/") || value.starts_with("\\") {
                        return Err(anyhow!("Invalid path '{}' in {} attribute. Use relative paths only.", value, key));
                    }
                }
                "execute" => {
                    if value.trim().is_empty() {
                        return Err(anyhow!("Execute command cannot be empty"));
                    }
                    // Check for dangerous commands
                    let dangerous_patterns = vec!["rm -rf", "deltree", "format"];
                    for pattern in dangerous_patterns {
                        if value.contains(pattern) {
                            return Err(anyhow!("Potentially dangerous command detected: {}", pattern));
                        }
                    }
                }
                "replace" => {
                    if !block.attributes.contains_key("find") {
                        return Err(anyhow!("'replace' attribute requires 'find' attribute"));
                    }
                }
                "step" => {
                    if let Err(_) = value.parse::<usize>() {
                        return Err(anyhow!("Step number must be a positive integer"));
                    }
                }
                "depends_on" => {
                    if let Err(_) = value.parse::<usize>() {
                        return Err(anyhow!("Dependency must be a valid step number"));
                    }
                }
                _ => {}
            }
        }
        
        // Validate content based on block type
        if block.attributes.contains_key("save") {
            if block.content.trim().is_empty() {
                return Err(anyhow!("Content cannot be empty for save blocks"));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_script_block_validation() {
        let parser = MarkdownParser;
        
        // Test valid block
        let mut block = ScriptBlock {
            attributes: [("save".to_string(), "test.txt".to_string())]
                .iter().cloned().collect(),
            content: "test content".to_string(),
            step_number: None,
            depends_on: None,
        };
        assert!(MarkdownParser::validate_script_block(&block).is_ok());
        
        // Test invalid path
        block.attributes.insert("save".to_string(), "../test.txt".to_string());
        assert!(MarkdownParser::validate_script_block(&block).is_err());
        
        // Test conflicting attributes
        block.attributes.insert("patch".to_string(), "test.txt".to_string());
        assert!(MarkdownParser::validate_script_block(&block).is_err());
    }

    #[test]
    fn test_parse_script_blocks() {
        let parser = MarkdownParser;
        let markdown = r#"
Some text

```{.script save="test.txt"}
test content
```

```{.script execute="echo hello"}
echo hello
```
"#;
        let blocks = parser.parse_script_blocks(markdown).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].content, "test content");
        assert_eq!(blocks[1].attributes.get("execute").unwrap(), "echo hello");
    }
}