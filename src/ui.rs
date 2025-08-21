//! ui.rs — helpers for clean terminal output (no emojis).
//! Uses crossterm styling; renders compact, readable sections.

use crossterm::style::Stylize;

/// Default content width for separators/panels.
pub const WIDTH: usize = 70;

/// Thin horizontal rule.
pub fn hr() -> String {
    "─".repeat(WIDTH).dark_grey().to_string()
}

/// A section title:  ▣ Title
pub fn title_line(title: &str) -> String {
    format!("█ {}", title.bold().white())
}

/// Bullet line: "  • text"
pub fn bullet_line(text: &str) -> String {
    format!("  • {}", text)
}

/// ASCII status tags (no emojis).
pub fn ok_tag(text: &str) -> String {
    format!("[OK] {}", text).green().to_string()
}
pub fn warn_tag(text: &str) -> String {
    format!("[WARN] {}", text).yellow().to_string()
}
pub fn err_tag(text: &str) -> String {
    format!("[ERR] {}", text).red().to_string()
}

/// Compact preview from large text (cap lines and characters).
pub fn preview(text: &str, max_lines: usize, max_chars: usize) -> String {
    let mut s = text.trim().to_string();
    if s.len() > max_chars {
        s.truncate(max_chars);
        s.push_str("\n… (truncated)");
    }
    let lines: Vec<&str> = s.lines().take(max_lines).collect();
    let mut joined = lines.join("\n");
    if text.lines().count() > max_lines {
        joined.push_str("\n… (output truncated)");
    }
    joined
}

/// Simple, clean panel with a title and body.
/// Layout:
/// ─────────────────────────────────────────────
/// █ TITLE
/// <body>
/// ─────────────────────────────────────────────
pub fn panel(title: &str, body: &str) -> String {
    let mut out = String::new();
    out.push_str(&hr());
    out.push('\n');
    out.push_str(&title_line(title));
    if !body.trim().is_empty() {
        out.push('\n');
        out.push_str(body);
    }
    out.push('\n');
    out.push_str(&hr());
    out
}
