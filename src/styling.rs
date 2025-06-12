use once_cell::sync::Lazy;
use crossterm::style::{Color, Stylize};
use std::borrow::Cow;

pub static STYLER: Lazy<Styler> = Lazy::new(|| Styler::new());

pub struct Styler {
    success: Color,
    error: Color,
    warning: Color,
    info: Color,
    bold_white: Color,
    dim_gray: Color,
    command_exec: Color,
    prompt: Color,
    separator: Color,
    header: Color,
    command_alt: Color,
    llm_response: Color,
}

impl Styler {
    fn new() -> Self {
        Self {
            success: Color::Green,
            error: Color::Red,
            warning: Color::Yellow,
            info: Color::Blue,
            bold_white: Color::White,
            dim_gray: Color::DarkGrey,
            command_exec: Color::Cyan,
            prompt: Color::Magenta,
            separator: Color::DarkGrey,
            header: Color::Blue,
            command_alt: Color::Cyan,
            llm_response: Color::Green,
        }
    }

    /// Base style function that applies a color to text. Returns a formatted string with:
    /// - text in specified color
    /// output:
    /// [color]text[reset]
    pub fn style_text<'a, D: std::fmt::Display>(&self, text: D, color: Color) -> impl std::fmt::Display + 'a {
        String::from(text.to_string()).with(color)
    }

    /// Style text in success color (green). Returns a formatted string with:
    /// - text in green color
    /// output:
    /// [green]text[reset]
    pub fn success_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.success)
    }

    /// Style text in error color (red). Returns a formatted string with:
    /// - text in red color
    /// output:
    /// [red]text[reset]
    pub fn error_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.error)
    }

    /// Style text in warning color (yellow). Returns a formatted string with:
    /// - text in yellow color (can be combined with .bold())
    /// output:
    /// Basic: [yellow]text[reset]
    /// With bold: [yellow bold]text[reset]
    pub fn warning_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.warning)
    }

    /// Style text in info color (blue). Returns a formatted string with:
    /// - text in blue color
    /// output:
    /// [blue]text[reset]
    pub fn info_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.info)
    }

    /// Style text in bold white. Returns a formatted string with:
    /// - text in white color and bold
    /// output:
    /// [white bold]text[reset]
    pub fn bold_white_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        String::from(text.to_string()).with(self.bold_white).bold()
    }

    /// Style text in dim gray. Returns a formatted string with:
    /// - text in dark grey color with dim effect
    /// output:
    /// [dark grey]text[reset]
    pub fn dim_gray_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        String::from(text.to_string()).with(self.dim_gray).dim()
    }

    /// Style text in command execution color (cyan). Returns a formatted string with:
    /// - text in cyan color
    /// output:
    /// [cyan]text[reset]
    pub fn command_exec_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.command_exec)
    }

    /// Style text for executing a command. Returns a formatted string with:
    /// - "executing" in warning color and bold
    /// - separator and command in dim gray
    /// - pwd path in dim gray
    /// - long separator in dim gray
    /// output:
    /// [yellow bold]executing[reset] [dim gray]━━━━━━━━━━━━━━━━━━
    /// /home/user/projects/myapp
    /// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    /// [reset]
    pub fn executing_command_style<'a>(
        &self,
        pwd: impl std::fmt::Display,
        command: impl std::fmt::Display,
    ) -> impl std::fmt::Display + 'a {
        let separator = "━".repeat(20);
        let long_separator = "━".repeat(80);
        // Create a bold warning style string directly
        let executing = String::from("executing").with(self.warning).bold();
        format!(
            "{} {} {}\n{}\n{}",
            executing,
            self.dim_gray_style(separator),
            self.dim_gray_style(command),
            self.dim_gray_style(pwd),
            self.dim_gray_style(long_separator)
        )
    }

    /// Style text in prompt color (magenta). Returns a formatted string with:
    /// - text in magenta color
    /// output:
    /// [magenta]text[reset]
    pub fn prompt_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.prompt)
    }

    /// Style text in separator color (dark grey). Returns a formatted string with:
    /// - text in dark grey color
    /// output:
    /// [dark grey]text[reset]
    pub fn separator_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.separator)
    }

    /// Style text in header format (blue and bold). Returns a formatted string with:
    /// - text in blue color with bold effect
    /// output:
    /// [blue]text[reset]
    pub fn header_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        String::from(text.to_string()).with(self.header).bold()
    }

    /// Style text in alternative command color (cyan). Returns a formatted string with:
    /// - text in cyan color
    /// output:
    /// [cyan]text[reset]
    pub fn command_style_alt<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.command_alt)
    }

    /// Style text for LLM responses (green). Returns a formatted string with:
    /// - text in green color
    /// output:
    /// [green]text[reset]
    pub fn llm_response_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.llm_response)
    }

    /// Style text for commands (cyan). Returns a formatted string with:
    /// - text in cyan color
    /// output:
    /// [cyan]text[reset]
    pub fn command_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.command_exec)
    }
}