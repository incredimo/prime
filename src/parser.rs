//! parser.rs — Unified Command Markdown (UCM) parser
//! -------------------------------------------------
//! Parses fenced code blocks that use Pandoc-like attributes.
//! Supported blocks: ```get …```, ```set …```, ```run …```
//
//! Examples:
//! ```get {#f1 lines=10-40}
//! file:Cargo.toml
//! glob:src/**/*.rs
//! mem:long
//! ```
//
//! ```set { target="file:NOTES.md" append=true }
//! New line
//! ```
//
//! ```run { lang=python timeout=30 }
//! print("hi")
//! ```
//
//! ```run { mode=http method=GET url="https://api.github.com" }
//! Accept: application/json
//! ```

use anyhow::{anyhow, Context, Result};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub enum ToolCall {
    Get(GetSpec),
    Set(SetSpec),
    Run(RunSpec),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedResponse {
    pub natural_language: String,
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attrs {
    pub id: Option<String>,
    pub classes: BTreeSet<String>,
    pub kv: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetSpec {
    pub id: Option<String>,
    pub cwd: Option<String>,
    pub limit_bytes: Option<usize>,
    pub lines: Option<(usize, usize)>,
    /// Targets: file:, dir:, glob:, http(s):, mem:*, input:, confirm:
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetSpec {
    pub id: Option<String>,
    pub cwd: Option<String>,
    pub target: String,     // e.g. file:PATH | mem:long | mkdir:PATH | rm:PATH
    pub append: bool,
    pub confirm: bool,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunSpec {
    pub id: Option<String>,
    pub cwd: Option<String>,
    pub timeout_secs: Option<u64>,
    // Mode A: script
    pub lang: Option<String>, // python|node|bash|pwsh|ruby|php
    pub args: Option<String>,
    pub code: Option<String>,
    pub sh_one_liner: bool,   // treat body as shell command
    // Mode B: HTTP
    pub http: Option<HttpSpec>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HttpSpec {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

pub fn parse_llm_response(input: &str) -> Result<ParsedResponse> {
    let mut blocks = extract_fences(input)?;
    let mut calls = Vec::new();

    let natural = trim_outside_text(input, &blocks);

    for b in blocks.drain(..) {
        match b.kind.as_str() {
            "get" => {
                let attrs = b.attrs;
                let mut targets = Vec::new();
                for line in b.body.lines() {
                    let t = line.trim();
                    if t.is_empty() || t.starts_with('#') { continue; }
                    targets.push(t.to_string());
                }
                let get = GetSpec {
                    id: attrs.id,
                    cwd: attrs.kv.get("cwd").cloned(),
                    limit_bytes: attrs.kv.get("limit_bytes").and_then(|v| v.parse().ok()),
                    lines: parse_lines_opt(attrs.kv.get("lines"))?,
                    targets,
                };
                calls.push(ToolCall::Get(get));
            }
            "set" => {
                let attrs = b.attrs;
                let target = attrs.kv.get("target")
                    .cloned()
                    .or_else(|| first_nonempty_line(&b.body))
                    .ok_or_else(|| anyhow!("set: missing 'target'"))?;
                let append = attrs.kv.get("append").map(|v| v == "true").unwrap_or(false);
                let confirm = attrs.kv.get("confirm").map(|v| v == "true").unwrap_or(false);
                let set = SetSpec {
                    id: attrs.id,
                    cwd: attrs.kv.get("cwd").cloned(),
                    target,
                    append,
                    confirm,
                    body: b.body,
                };
                calls.push(ToolCall::Set(set));
            }
            "run" => {
                let attrs = b.attrs;
                let timeout_secs = attrs.kv.get("timeout")
                    .or_else(|| attrs.kv.get("timeout_secs"))
                    .and_then(|v| v.parse::<u64>().ok());
                // HTTP mode?
                let http_mode = attrs.kv.get("mode").map(|s| s == "http").unwrap_or(false);
                if http_mode {
                    let method = attrs.kv.get("method").cloned().unwrap_or_else(|| "GET".into());
                    let url = attrs.kv.get("url")
                        .cloned()
                        .ok_or_else(|| anyhow!("run{{mode=http}}: missing url"))?;
                    let (headers, body) = split_headers_body(&b.body);
                    let http = HttpSpec { method, url, headers, body };
                    calls.push(ToolCall::Run(RunSpec {
                        id: attrs.id,
                        cwd: attrs.kv.get("cwd").cloned(),
                        timeout_secs,
                        lang: None,
                        args: None,
                        code: None,
                        sh_one_liner: false,
                        http: Some(http),
                    }));
                } else {
                    // Script or shell
                    let sh_one = attrs.kv.get("sh").map(|v| v == "true").unwrap_or(false);
                    let lang = attrs.kv.get("lang").cloned();
                    let args = attrs.kv.get("args").cloned().map(|s| strip_quotes(&s));
                    let code = if sh_one { None } else { Some(b.body) };
                    let run = RunSpec {
                        id: attrs.id,
                        cwd: attrs.kv.get("cwd").cloned(),
                        timeout_secs,
                        lang,
                        args,
                        code,
                        sh_one_liner: sh_one,
                        http: None,
                    };
                    calls.push(ToolCall::Run(run));
                }
            }
            _ => { /* ignore unknown blocks */ }
        }
    }

    Ok(ParsedResponse {
        natural_language: natural,
        tool_calls: calls,
    })
}

/* ---------------- internals ---------------- */

#[derive(Debug, Clone)]
struct Fence {
    kind: String, // get|set|run
    attrs: Attrs,
    body: String,
    start: usize,
    end: usize,
}

fn trim_outside_text(all: &str, blocks: &[Fence]) -> String {
    if let Some(first) = blocks.first() {
        let head = all[..first.start].trim();
        return head.to_string();
    }
    all.trim().to_string()
}

fn parse_lines_opt(v: Option<&String>) -> Result<Option<(usize, usize)>> {
    if let Some(s) = v {
        let parts: Vec<_> = s.split('-').collect();
        if parts.len() != 2 { return Err(anyhow!("lines must be START-END")); }
        let a = parts[0].parse::<usize>()?;
        let b = parts[1].parse::<usize>()?;
        if a == 0 || a > b { return Err(anyhow!("invalid lines range")); }
        return Ok(Some((a, b)));
    }
    Ok(None)
}

fn first_nonempty_line(s: &str) -> Option<String> {
    for l in s.lines() {
        let t = l.trim();
        if !t.is_empty() { return Some(t.to_string()); }
    }
    None
}

fn strip_quotes(s: &str) -> String {
    let t = s.trim();
    if (t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')) {
        return t[1..t.len()-1].to_string();
    }
    t.to_string()
}

fn split_headers_body(body: &str) -> (Vec<(String, String)>, Option<String>) {
    let mut headers = Vec::new();
    let mut lines = body.lines().peekable();

    while let Some(line) = lines.peek() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            lines.next(); // consume the blank line
            break;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
        lines.next();
    }

    let remaining: Vec<_> = lines.collect();
    let body_opt = if remaining.is_empty() {
        None
    } else {
        Some(remaining.join("\n"))
    };

    (headers, body_opt)
}

fn extract_fences(input: &str) -> Result<Vec<Fence>> {
    let mut out = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0usize;

    while i + 3 <= bytes.len() {
        // find a fence start ```
        if bytes[i] == b'`' {
            let tick_start = i;
            let mut ticks = 0;
            while i < bytes.len() && bytes[i] == b'`' { ticks += 1; i += 1; }
            if ticks < 3 { continue; }

            // read to end of line to get lang/attrs
            let line_end = input[i..].find('\n').map(|o| i + o).unwrap_or(bytes.len());
            let header = input[i..line_end].trim();

            // header can be: "get {#id key=val}" OR "{.get #id key=val}" OR "run { … }"
            let (kind, attr_str) = parse_header(header)?;
            if !(kind == "get" || kind == "set" || kind == "run") {
                i = line_end + 1;
                continue;
            }

            // body until matching fence of same length
            let mut j = line_end + 1;
            let fence_pat = "`".repeat(ticks);
            let mut end = None;
            while j < bytes.len() {
                if input[j..].starts_with(&fence_pat) {
                    // make sure fence is alone on the line (or just followed by spaces)
                    let after = j + ticks;
                    let line_tail = input[after..].split_once('\n').map(|(h, _)| h).unwrap_or("");
                    if line_tail.trim().is_empty() {
                        end = Some(j);
                        break;
                    }
                }
                // advance to next newline
                if let Some(nl) = input[j..].find('\n') {
                    j += nl + 1;
                } else {
                    break;
                }
            }
            let Some(body_end) = end else { return Err(anyhow!("Unclosed fence for {}", kind)); };

            let body = &input[line_end + 1 .. body_end];
            let attrs = parse_attrs(attr_str)?;
            out.push(Fence {
                kind,
                attrs,
                body: body.to_string(),
                start: tick_start,
                end: body_end + ticks, // end of closing backticks
            });

            // move i to after the closing fence line
            if let Some(nl) = input[body_end + ticks ..].find('\n') {
                i = body_end + ticks + nl + 1;
            } else {
                i = bytes.len();
            }
        } else {
            i += 1;
        }
    }
    Ok(out)
}

/// Parses header like:
///   "get {#id .x key=val key2=\"v\"}"
///   "{.get #abc lines=1-4}"
fn parse_header(header: &str) -> Result<(String, String)> {
    let h = header.trim();
    if h.starts_with('{') {
        // pure attrs; must include .get|.set|.run
        let attrs = h;
        let classes = sniff_classes(attrs);
        let k = classes
            .into_iter()
            .find(|c| c == "get" || c == "set" || c == "run")
            .ok_or_else(|| anyhow!("attrs fence missing class .get/.set/.run"))?;
        Ok((k, attrs.to_string()))
    } else {
        // language + optional attrs
        let (lang, rest) = if let Some(sp) = h.find('{') {
            (h[..sp].trim(), h[sp..].trim())
        } else {
            (h, "")
        };
        let k = lang.to_lowercase();
        Ok((k, rest.to_string()))
    }
}

fn sniff_classes(attrs: &str) -> Vec<String> {
    let inside = attrs.trim().trim_start_matches('{').trim_end_matches('}');
    let mut out = Vec::new();
    let mut i = 0usize;
    let b = inside.as_bytes();
    while i < b.len() {
        while i < b.len() && b[i].is_ascii_whitespace() { i += 1; }
        if i >= b.len() { break; }
        if b[i] == b'.' {
            i += 1;
            let start = i;
            while i < b.len() && !b[i].is_ascii_whitespace() && b[i] != b'.' && b[i] != b'#' && b[i] != b'}' {
                i += 1;
            }
            out.push(inside[start..i].to_string());
        } else {
            // skip token
            while i < b.len() && !b[i].is_ascii_whitespace() { i += 1; }
        }
    }
    out
}

fn parse_attrs(attrs: String) -> Result<Attrs> {
    let mut id = None;
    let mut classes = BTreeSet::new();
    let mut kv = BTreeMap::new();

    let s = attrs.trim();
    if s.is_empty() { return Ok(Attrs { id, classes, kv }); }
    let inner = s.trim_start_matches('{').trim_end_matches('}').trim();

    // simple tokenizer: tokens separated by whitespace, but key="value with spaces" kept together
    let mut i = 0usize;
    let bytes = inner.as_bytes();
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() { i += 1; }
        if i >= bytes.len() { break; }

        match bytes[i] as char {
            '.' => {
                i += 1;
                let start = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'.' && bytes[i] != b'#' {
                    i += 1;
                }
                classes.insert(inner[start..i].to_string());
            }
            '#' => {
                i += 1;
                let start = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'.' && bytes[i] != b'#' {
                    i += 1;
                }
                id = Some(inner[start..i].to_string());
            }
            _ => {
                // key=val or lone word
                let start = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'=' {
                    i += 1;
                }
                let key = inner[start..i].to_string();
                while i < bytes.len() && bytes[i].is_ascii_whitespace() { i += 1; }
                if i < bytes.len() && bytes[i] == b'=' {
                    i += 1;
                    while i < bytes.len() && bytes[i].is_ascii_whitespace() { i += 1; }
                    // value can be quoted
                    let val = if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                        let q = bytes[i];
                        i += 1;
                        let start = i;
                        while i < bytes.len() && bytes[i] != q { i += 1; }
                        let v = inner[start..i].to_string();
                        i += 1; // skip closing quote
                        v
                    } else {
                        let start = i;
                        while i < bytes.len() && !bytes[i].is_ascii_whitespace() { i += 1; }
                        inner[start..i].to_string()
                    };
                    kv.insert(key, val);
                } else {
                    // bare token -> treat as boolean true
                    kv.insert(key, "true".into());
                }
            }
        }
    }

    Ok(Attrs { id, classes, kv })
}
