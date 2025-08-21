//! parser.rs – extract Plan & Execution and Action Block from LLM messages.
//! ----------------------------------------------------------------------------
//! v0.1.9 fixes:
//! * Corrected regex – we accidentally looked for the **literal** "\\s". Now we
//!   match real whitespace so fenced ````primeactions` blocks are detected again.
//! * Still supports `<end_of_tool_output/>` sentinel, unchanged public API.
//!
//! Grammar recap:
//!
//! ```text
//! Natural language (plan / status / rationale)...
//!
//! ```primeactions
//! shell: ls -la
//! read_file: Cargo.toml lines=1-20
//! write_file: path/to/file append=true
//! <content terminated by the line: EOF_PRIME>
//! EOF_PRIME
//! ```
//! ```

use anyhow::{anyhow, Context, Result};
use regex::Regex;

#[derive(Debug, PartialEq, Clone)]
pub enum ToolCall {
    Shell { command: String },
    ReadFile { path: String, lines: Option<(usize, usize)> },
    WriteFile { path: String, content: String, append: bool },
    ListDir { path: String },
    WriteMemory { memory_type: String, content: String },
    ClearMemory { memory_type: String },
    RunScript { lang: String, args: Option<String>, code: String, timeout_secs: Option<u64> },
}

#[derive(Debug, Default)]
pub struct ParsedResponse {
    pub natural_language: String,
    pub tool_calls: Vec<ToolCall>,
}

/// Helper to parse write_file arguments like "path/to/file append=true"
fn parse_write_args(args_str: &str) -> (String, bool) {
    let mut append = false;
    let mut path = args_str.to_string();

    if let Some(pos) = path.rfind(" append=true") {
        if pos + " append=true".len() == path.len() {
            append = true;
            path.truncate(pos);
        }
    }
    (path.trim().to_string(), append)
}

/// Helper to parse read_file arguments like "path/to/file lines=10-20"
fn parse_read_args(args_str: &str) -> Result<(String, Option<(usize, usize)>)> {
    if let Some(pos) = args_str.rfind(" lines=") {
        let path = args_str[..pos].trim().to_string();
        let range_str = args_str[pos + " lines=".len()..].trim();
        let parts: Vec<&str> = range_str.split('-').collect();
        if parts.len() == 2 {
            let start = parts[0]
                .parse::<usize>()
                .context(format!("Invalid start line number: {}", parts[0]))?;
            let end = parts[1]
                .parse::<usize>()
                .context(format!("Invalid end line number: {}", parts[1]))?;
            return Ok((path, Some((start, end))));
        } else {
            return Err(anyhow!("Invalid lines format. Expected start-end"));
        }
    }
    Ok((args_str.trim().to_string(), None))
}

pub fn parse_llm_response(input: &str) -> Result<ParsedResponse> {
    let mut resp = ParsedResponse::default();

    // Explicit done hint: if present, treat as completion with natural text only.
    if input.contains("<done/>") {
        resp.natural_language = input.trim().to_string();
        return Ok(resp);
    }

    // 1️⃣ Extract fenced action block – (?s) makes . match newlines
    let fence_re = Regex::new(r"(?s)```[ \t]*primeactions[ \t]*\n(.*?)```")
        .map_err(|e| anyhow::anyhow!("Failed to compile regex for parsing primeactions block: {}", e))?;
    let Some(caps) = fence_re.captures(input) else {
        // No action block – treat whole message as natural text
        resp.natural_language = input.trim().to_string();
        return Ok(resp);
    };

    let actions_block = caps.get(1).unwrap().as_str();

    // text before first fence = natural language
    let start_idx = caps.get(0).unwrap().start();
    resp.natural_language = input[..start_idx].trim().to_string();

    // 2️⃣ Parse lines inside the fence
    let mut lines_iter = actions_block.lines().peekable();

    while let Some(line) = lines_iter.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (tool_name, args_str) = match trimmed.split_once(':') {
            Some((t, a)) => (t.trim(), a.trim()),
            None => continue, // skip malformed lines
        };

        let tool_call = match tool_name {
            "shell" => ToolCall::Shell {
                command: args_str.into(),
            },
            "list_dir" => ToolCall::ListDir {
                path: args_str.into(),
            },
            "read_file" => {
                let (path, lines) = parse_read_args(args_str)?;
                ToolCall::ReadFile { path, lines }
            }
            "write_memory" => {
                let mut parts = args_str.splitn(2, ' ');
                let memory_type = parts.next().unwrap_or("").to_string();
                let mut content_lines = Vec::new();
                while let Some(cl) = lines_iter.next() {
                    if cl.trim() == "EOF_PRIME" {
                        break;
                    }
                    content_lines.push(cl);
                }
                ToolCall::WriteMemory {
                    memory_type,
                    content: content_lines.join("\n"),
                }
            }
            "clear_memory" => {
                ToolCall::ClearMemory {
                    memory_type: args_str.to_string(),
                }
            }
            "write_file" => {
                let (path, append) = parse_write_args(args_str);
                let mut content_lines = Vec::new();
                while let Some(cl) = lines_iter.next() {
                    if cl.trim() == "EOF_PRIME" {
                        break;
                    }
                    content_lines.push(cl);
                }
                ToolCall::WriteFile {
                    path,
                    content: content_lines.join("\n"),
                    append,
                }
            }
            "run_script" => {
                // e.g. run_script: lang=python args="--flag" timeout=30
                let mut lang = String::new();
                let mut args = None;
                let mut timeout_secs = None;

                for part in args_str.split_whitespace() {
                    if let Some(v) = part.strip_prefix("lang=") {
                        lang = v.to_string();
                    } else if let Some(v) = part.strip_prefix("args=") {
                        args = Some(v.trim_matches('"').to_string());
                    } else if let Some(v) = part.strip_prefix("timeout=") {
                        timeout_secs = v.parse().ok();
                    }
                }
                let mut code_lines = Vec::new();
                while let Some(cl) = lines_iter.next() {
                    if cl.trim() == "EOF_PRIME" {
                        break;
                    }
                    code_lines.push(cl);
                }
                ToolCall::RunScript {
                    lang,
                    args,
                    code: code_lines.join("\n"),
                    timeout_secs,
                }
            }
            _ => continue, // ignore unknown tools
        };
        resp.tool_calls.push(tool_call);
    }

    Ok(resp)
}
