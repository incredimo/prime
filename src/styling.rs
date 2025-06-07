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

    pub fn style_text<'a, D: std::fmt::Display>(&self, text: D, color: Color) -> impl std::fmt::Display + 'a {
        String::from(text.to_string()).with(color)
    }

    pub fn success_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.success)
    }

    pub fn error_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.error)
    }

    pub fn warning_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.warning)
    }

    pub fn info_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.info)
    }

    pub fn bold_white_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        String::from(text.to_string()).with(self.bold_white).bold()
    }

    pub fn dim_gray_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        String::from(text.to_string()).with(self.dim_gray).dim()
    }

    pub fn command_exec_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.command_exec)
    }

    pub fn prompt_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.prompt)
    }

    pub fn separator_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.separator)
    }

    pub fn header_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        String::from(text.to_string()).with(self.header).bold()
    }

    pub fn command_style_alt<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.command_alt)
    }

    pub fn llm_response_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.llm_response)
    }

    pub fn command_style<'a, D: std::fmt::Display>(&self, text: D) -> impl std::fmt::Display + 'a {
        self.style_text(text, self.command_exec)
    }
}