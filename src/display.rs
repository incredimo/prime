//! Enhanced display utilities for rich terminal output
//! Maintains simple protocol while providing beautiful formatting

use crossterm::style::Stylize;
use std::io::{self, Write};
use std::time::Duration;

/// Display styles for different message types
pub struct DisplayStyle {
    pub user_prefix: String,
    pub assistant_prefix: String,
    pub tool_prefix: String,
    pub error_prefix: String,
    pub success_prefix: String,
}

impl Default for DisplayStyle {
    fn default() -> Self {
        Self {
            user_prefix: "»".to_string(),
            assistant_prefix: "┃".to_string(),
            tool_prefix: "┃".to_string(),
            error_prefix: "✗".to_string(),
            success_prefix: "✓".to_string(),
        }
    }
}

/// Simple spinner for long operations
pub struct Spinner {
    frames: Vec<&'static str>,
    current: usize,
    message: String,
}

impl Spinner {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current: 0,
            message: message.into(),
        }
    }

    pub fn tick(&mut self) -> String {
        let frame = self.frames[self.current];
        self.current = (self.current + 1) % self.frames.len();
        format!("{} {}", frame, self.message)
    }
}

/// Progress bar for operations with known duration
pub struct ProgressBar {
    width: usize,
    current: usize,
    total: usize,
    message: String,
}

impl ProgressBar {
    pub fn new(total: usize, message: impl Into<String>) -> Self {
        Self {
            width: 40,
            current: 0,
            total,
            message: message.into(),
        }
    }

    pub fn update(&mut self, current: usize) {
        self.current = current.min(self.total);
    }

    pub fn render(&self) -> String {
        let percentage = if self.total > 0 {
            (self.current as f64 / self.total as f64 * 100.0) as usize
        } else {
            0
        };

        let filled = (self.width * self.current) / self.total.max(1);
        let empty = self.width - filled;

        let bar = format!(
            "[{}{}] {}% {}",
            "=".repeat(filled),
            " ".repeat(empty),
            percentage,
            self.message
        );

        bar
    }
}

/// Format tool execution header
pub fn format_tool_header(tool_name: &str, args: &str) -> String {
    format!("┏━ {}\n┃ {}", "actions".cyan(), format!("{}: {}", tool_name, args).white())
}

/// Format tool execution footer with timing
pub fn format_tool_footer(duration: Duration, success: bool) -> String {
    let duration_str = if duration.as_secs() > 0 {
        format!("{:.1}s", duration.as_secs_f64())
    } else {
        format!("{}ms", duration.as_millis())
    };

    let status = if success {
        format!("completed in {}", duration_str).green()
    } else {
        format!("failed after {}", duration_str).red()
    };

    format!("╰────────────────────────────────────── {} ────────", status)
}

/// Format tool output with box drawing
pub fn format_tool_output(output: &str, max_lines: Option<usize>) -> String {
    let lines: Vec<String> = output.lines().map(|s| s.to_string()).collect();
    let display_lines = if let Some(max) = max_lines {
        if lines.len() > max {
            let mut result: Vec<String> = lines[..max - 1].to_vec();
            result.push(format!("... ({} more lines)", lines.len() - max + 1));
            result
        } else {
            lines
        }
    } else {
        lines
    };

    display_lines
        .iter()
        .map(|line| format!("│ {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format error message with context
pub fn format_error(error: &str, context: Option<&str>) -> String {
    let mut output = format!("{} {}", "✗".red(), error.red());
    if let Some(ctx) = context {
        output.push_str(&format!("\n  {}", ctx.dark_grey()));
    }
    output
}

/// Format success message
pub fn format_success(message: &str) -> String {
    format!("{} {}", "✓".green(), message.green())
}

/// Format streaming text with word wrapping
pub fn format_streaming_text(text: &str, width: usize) -> Vec<String> {
    textwrap::wrap(text, width)
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Display a confirmation prompt
pub fn prompt_confirmation(message: &str, default: bool) -> io::Result<bool> {
    let default_str = if default { "Y/n" } else { "y/N" };
    print!("{} [{}]: ", message.yellow(), default_str);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let input = input.trim().to_lowercase();
    Ok(match input.as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        "" => default,
        _ => default,
    })
}

/// Clear current line (for updating spinners/progress)
pub fn clear_line() {
    print!("\r\x1b[K");
    let _ = io::stdout().flush();
}

/// Move cursor up N lines
pub fn cursor_up(n: usize) {
    print!("\x1b[{}A", n);
    let _ = io::stdout().flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner() {
        let mut spinner = Spinner::new("Loading");
        let frame1 = spinner.tick();
        let frame2 = spinner.tick();
        assert_ne!(frame1, frame2);
        assert!(frame1.contains("Loading"));
    }

    #[test]
    fn test_progress_bar() {
        let mut bar = ProgressBar::new(100, "Processing");
        bar.update(50);
        let output = bar.render();
        assert!(output.contains("50%"));
        assert!(output.contains("Processing"));
    }

    #[test]
    fn test_text_wrapping() {
        let text = "This is a very long line that should be wrapped at the specified width";
        let wrapped = format_streaming_text(text, 20);
        assert!(wrapped.len() > 1);
        assert!(wrapped.iter().all(|line| line.len() <= 20));
    }
}
