use anyhow::{anyhow, Context, Result};

#[derive(Debug, PartialEq, Clone)]
pub enum ToolCall {
    Shell { command: String },
    ReadFile { path: String, lines: Option<(usize, usize)> },
    WriteFile { path: String, content: String, append: bool },
    ListDir { path: String },
    ChangeDir { path: String },
    WriteMemory { memory_type: String, content: String },
    ClearMemory { memory_type: String },
    ScriptTool { name: String, args: Vec<String> },
    CreateTool { name: String, desc: String, args: String, script_content: String },
}

#[derive(Debug, Default)]
pub struct ParsedResponse {
    pub natural_language: String,
    pub tool_calls: Vec<ToolCall>,
}

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

fn parse_read_args(args_str: &str) -> Result<(String, Option<(usize, usize)>)> {
    if let Some(pos) = args_str.rfind(" lines=") {
        let path = args_str[..pos].trim().to_string();
        let range_str = &args_str[pos + " lines=".len()..].trim();
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

fn find_primeactions_block(input: &str) -> (String, Vec<&str>) {
    let lines: Vec<&str> = input.lines().collect();
    let mut natural = String::new();
    let mut block_lines = Vec::new();
    let mut in_block = false;
    for line in lines {
        let trimmed = line.trim();
        if !in_block {
            if trimmed.starts_with("```primeactions") {
                in_block = true;
                continue;
            }
            natural.push_str(line);
            natural.push('\n');
        } else {
            if trimmed.starts_with("```") {
                in_block = false;
            } else {
                block_lines.push(line);
            }
        }
    }
    (natural.trim().to_string(), block_lines)
}

fn parse_create_tool_args(args_str: &str) -> Result<(String, String, String)> {
    let mut chars = args_str.chars().peekable();
    let mut name = String::new();
    let mut desc = String::new();
    let mut args_spec = String::new();
    let mut current_key = String::new();
    loop {
        while chars.peek().map_or(false, |&ch| ch.is_ascii_whitespace()) {
            chars.next();
        }
        if chars.peek().is_none() {
            break;
        }
        current_key.clear();
        if let Some(ch) = chars.next() {
            current_key.push(ch);
        }
        while chars.peek().map_or(false, |&ch| ch != '=') {
            if let Some(ch) = chars.next() {
                current_key.push(ch);
            }
        }
        if chars.peek().map_or(true, |&ch| ch != '=') {
            continue;
        }
        chars.next();
        while chars.peek().map_or(false, |&ch| ch.is_ascii_whitespace()) {
            chars.next();
        }
        if chars.peek().map_or(true, |&ch| ch != '"') {
            continue;
        }
        chars.next();
        let mut value = String::new();
        while let Some(ch) = chars.next() {
            if ch == '"' {
                break;
            }
            value.push(ch);
        }
        match current_key.trim() {
            "name" => name = value,
            "desc" => desc = value,
            "args" => args_spec = value,
            _ => {}
        }
    }
    if name.is_empty() || desc.is_empty() || args_spec.is_empty() {
        return Err(anyhow!("Invalid create_tool args: missing name, desc, or args"));
    }
    Ok((name, desc, args_spec))
}

pub fn parse_llm_response(input: &str) -> Result<ParsedResponse> {
    let mut resp = ParsedResponse::default();
    let (natural, block_lines) = find_primeactions_block(input);
    resp.natural_language = natural;
    let mut lines_iter = block_lines.into_iter().peekable();
    while let Some(line) = lines_iter.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (tool_name, args_str) = match trimmed.split_once(':') {
            Some((t, a)) => (t.trim(), a.trim()),
            None => continue,
        };
        let tool_call = match tool_name {
            "shell" => ToolCall::Shell {
                command: args_str.into(),
            },
            "list_dir" => ToolCall::ListDir {
                path: args_str.into(),
            },
            "cd" | "change_dir" => ToolCall::ChangeDir {
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
            "create_tool" => {
                let (name, desc, args_spec) = parse_create_tool_args(args_str)?;
                let mut content_lines = Vec::new();
                while let Some(cl) = lines_iter.next() {
                    if cl.trim() == "EOF_PRIME" {
                        break;
                    }
                    content_lines.push(cl);
                }
                let script_content = content_lines.join("\n");
                ToolCall::CreateTool { name, desc, args: args_spec, script_content }
            }
            _ => {
                let parts: Vec<_> = args_str.split_whitespace().map(|s| s.to_string()).collect();
                ToolCall::ScriptTool {
                    name: tool_name.to_string(),
                    args: parts,
                }
            }
        };
        resp.tool_calls.push(tool_call);
    }
    Ok(resp)
}