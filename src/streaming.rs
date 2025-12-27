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

/// State machine for code block detection
#[derive(Debug, Clone, PartialEq)]
enum CodeBlockState {
    /// Not in any code block
    None,
    /// Detected partial ``` sequence
    PartialMarker(usize),
    /// Inside a code block with specified language
    InBlock(String),
}

/// Streaming response handler with intelligent buffering
pub struct StreamHandler {
    buffer: String,
    state: CodeBlockState,
    last_flush: Instant,
    flush_interval: Duration,
}

impl StreamHandler {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            state: CodeBlockState::None,
            last_flush: Instant::now(),
            flush_interval: Duration::from_millis(50), // Smooth 20 FPS display
        }
    }

    /// Process incoming token and determine if it should be displayed or buffered
    pub fn process_token(&mut self, token: &str) -> Vec<StreamToken> {
        let mut output = Vec::new();
        self.buffer.push_str(token);

        loop {
            match &self.state {
                CodeBlockState::None => {
                    // Look for start of code block
                    if let Some(idx) = self.buffer.find("```") {
                        // Check if we have a complete code block start (with newline)
                        let after_marker = &self.buffer[idx + 3..];
                        if let Some(newline_idx) = after_marker.find('\n') {
                            let lang = after_marker[..newline_idx].trim().to_string();
                            
                            // Emit text before the code block
                            let before = &self.buffer[..idx];
                            if !before.is_empty() {
                                output.push(StreamToken::Text(before.to_string()));
                            }
                            
                            // Update state
                            self.state = CodeBlockState::InBlock(lang.clone());
                            self.buffer = after_marker[newline_idx + 1..].to_string();
                            
                            // If not primeactions, emit the opening marker
                            if lang != "primeactions" {
                                output.push(StreamToken::Text(format!("```{}\n", lang)));
                            }
                            continue;
                        } else {
                            // Partial marker, wait for more input
                            break;
                        }
                    } else if self.buffer.ends_with('`') || self.buffer.ends_with("``") {
                        // Potential start of code block, wait for more
                        break;
                    } else {
                        // No code block markers, flush if enough time passed
                        if self.should_flush() && !self.buffer.is_empty() {
                            output.push(StreamToken::Text(self.buffer.clone()));
                            self.buffer.clear();
                            self.last_flush = Instant::now();
                        }
                        break;
                    }
                }
                CodeBlockState::InBlock(lang) => {
                    let lang = lang.clone();
                    // Look for end of code block
                    if let Some(idx) = self.find_closing_marker() {
                        let content = self.buffer[..idx].to_string();
                        let remaining = self.buffer[idx + 3..].to_string();
                        
                        if lang == "primeactions" {
                            // Emit as tool call
                            output.push(StreamToken::ToolCall(content));
                        } else {
                            // Emit content and closing marker
                            output.push(StreamToken::Text(content));
                            output.push(StreamToken::Text("```".to_string()));
                        }
                        
                        self.buffer = remaining;
                        self.state = CodeBlockState::None;
                        continue;
                    } else {
                        // Still in block, check for partial closing marker
                        if self.buffer.ends_with('`') || self.buffer.ends_with("``") {
                            break;
                        }
                        
                        // For non-primeactions blocks, emit content gradually
                        if lang != "primeactions" && self.should_flush() && !self.buffer.is_empty() {
                            output.push(StreamToken::Text(self.buffer.clone()));
                            self.buffer.clear();
                            self.last_flush = Instant::now();
                        }
                        break;
                    }
                }
                CodeBlockState::PartialMarker(_) => {
                    // Should not reach here in normal operation
                    break;
                }
            }
        }

        output
    }

    /// Find closing ``` marker that's on its own line or at start
    fn find_closing_marker(&self) -> Option<usize> {
        // Look for ``` preceded by newline or at start
        let mut idx = 0;
        while let Some(pos) = self.buffer[idx..].find("```") {
            let abs_pos = idx + pos;
            // Check if this is at start or preceded by newline
            if abs_pos == 0 || self.buffer.as_bytes().get(abs_pos - 1) == Some(&b'\n') {
                return Some(abs_pos);
            }
            idx = abs_pos + 3;
        }
        None
    }

    /// Check if enough time has passed for a flush
    fn should_flush(&self) -> bool {
        self.last_flush.elapsed() >= self.flush_interval
    }

    /// Flush any remaining buffered content
    pub fn flush(&mut self) -> Option<StreamToken> {
        if !self.buffer.is_empty() {
            let content = self.buffer.clone();
            self.buffer.clear();
            
            match &self.state {
                CodeBlockState::InBlock(lang) if lang == "primeactions" => {
                    // Incomplete primeactions block - emit as text
                    Some(StreamToken::Text(content))
                }
                _ => Some(StreamToken::Text(content))
            }
        } else {
            None
        }
    }

    /// Check if currently buffering a primeactions block
    pub fn is_buffering_primeactions(&self) -> bool {
        matches!(&self.state, CodeBlockState::InBlock(lang) if lang == "primeactions")
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
        assert!(!tokens.is_empty());
        
        if let StreamToken::Text(text) = &tokens[0] {
            assert!(text.contains("Hello"));
        }
    }

    #[test]
    fn test_primeactions_buffering() {
        let mut handler = StreamHandler::new();
        
        handler.process_token("```primeactions\n");
        handler.process_token("shell: ls\n");
        let tokens = handler.process_token("```");
        
        assert!(!tokens.is_empty());
        let has_tool_call = tokens.iter().any(|t| matches!(t, StreamToken::ToolCall(_)));
        assert!(has_tool_call);
    }

    #[test]
    fn test_regular_code_block() {
        let mut handler = StreamHandler::new();
        
        let tokens = handler.process_token("```python\nprint('hello')\n```");
        
        // Regular code blocks should be emitted as text
        assert!(tokens.iter().any(|t| matches!(t, StreamToken::Text(_))));
    }

    #[test]
    fn test_text_before_code_block() {
        let mut handler = StreamHandler::new();
        
        let tokens = handler.process_token("Here is some code:\n```python\nprint('hello')\n```");
        
        // Should have text before the code block
        let texts: Vec<_> = tokens.iter().filter_map(|t| {
            if let StreamToken::Text(s) = t { Some(s.as_str()) } else { None }
        }).collect();
        assert!(texts.iter().any(|t| t.contains("Here is some code")));
    }
}
