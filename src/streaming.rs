//! Streaming response handler for real-time LLM output
//! Provides intelligent buffering for tool detection while maintaining simple protocol

use std::time::{Duration, Instant};

/// Token received from streaming LLM response
#[derive(Debug, Clone)]
pub enum StreamToken {
    /// Regular text token
    Text(String),
    /// Tool call detected (buffered and parsed)
    ToolCall(String),
    /// Stream completed
    Done,
}

/// Streaming response handler with intelligent buffering
pub struct StreamHandler {
    buffer: String,
    in_code_block: bool,
    code_block_lang: Option<String>,
    last_flush: Instant,
    flush_interval: Duration,
}

impl StreamHandler {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            in_code_block: false,
            code_block_lang: None,
            last_flush: Instant::now(),
            flush_interval: Duration::from_millis(50), // Smooth 20 FPS display
        }
    }

    /// Process incoming token and determine if it should be displayed or buffered
    pub fn process_token(&mut self, token: &str) -> Vec<StreamToken> {
        let mut output = Vec::new();
        self.buffer.push_str(token);

        // Check for code block markers
        if self.buffer.contains("```") {
            if let Some(idx) = self.buffer.rfind("```") {
                let before = &self.buffer[..idx];
                let after = &self.buffer[idx..];
                
                if !self.in_code_block {
                    // Starting code block
                    if let Some(newline_idx) = after.find('\n') {
                        let lang = after[3..newline_idx].trim().to_string();
                        self.code_block_lang = Some(lang.clone());
                        self.in_code_block = true;
                        
                        // Check if this is a primeactions block
                        if lang == "primeactions" {
                            // Buffer the entire block for tool parsing
                            return output;
                        }
                        
                        // Flush everything before the code block
                        if !before.is_empty() {
                            output.push(StreamToken::Text(before.to_string()));
                        }
                        output.push(StreamToken::Text(after[..=newline_idx].to_string()));
                        self.buffer = after[newline_idx + 1..].to_string();
                    }
                } else {
                    // Ending code block
                    self.in_code_block = false;
                    
                    // If it was a primeactions block, emit as tool call
                    if self.code_block_lang.as_deref() == Some("primeactions") {
                        output.push(StreamToken::ToolCall(before.to_string()));
                        self.buffer.clear();
                        self.code_block_lang = None;
                        return output;
                    }
                    
                    // Regular code block - flush it
                    output.push(StreamToken::Text(self.buffer.clone()));
                    self.buffer.clear();
                    self.code_block_lang = None;
                }
            }
        }

        // Flush buffer periodically for smooth display (but not during primeactions)
        if !self.in_code_block || self.code_block_lang.as_deref() != Some("primeactions") {
            if self.last_flush.elapsed() >= self.flush_interval && !self.buffer.is_empty() {
                output.push(StreamToken::Text(self.buffer.clone()));
                self.buffer.clear();
                self.last_flush = Instant::now();
            }
        }

        output
    }

    /// Flush any remaining buffered content
    pub fn flush(&mut self) -> Option<StreamToken> {
        if !self.buffer.is_empty() {
            let content = self.buffer.clone();
            self.buffer.clear();
            Some(StreamToken::Text(content))
        } else {
            None
        }
    }
}

impl Default for StreamHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regular_text_streaming() {
        let mut handler = StreamHandler::new();
        
        let tokens = handler.process_token("Hello ");
        assert!(tokens.is_empty()); // Buffered
        
        std::thread::sleep(Duration::from_millis(60));
        let tokens = handler.process_token("world");
        assert_eq!(tokens.len(), 1);
        
        if let StreamToken::Text(text) = &tokens[0] {
            assert_eq!(text, "Hello ");
        }
    }

    #[test]
    fn test_primeactions_buffering() {
        let mut handler = StreamHandler::new();
        
        handler.process_token("```primeactions\n");
        handler.process_token("shell: ls\n");
        let tokens = handler.process_token("```");
        
        assert_eq!(tokens.len(), 1);
        if let StreamToken::ToolCall(content) = &tokens[0] {
            assert!(content.contains("shell: ls"));
        }
    }

    #[test]
    fn test_regular_code_block() {
        let mut handler = StreamHandler::new();
        
        handler.process_token("```python\n");
        handler.process_token("print('hello')\n");
        let tokens = handler.process_token("```");
        
        // Regular code blocks are flushed as text
        assert!(tokens.iter().any(|t| matches!(t, StreamToken::Text(_))));
    }
}
