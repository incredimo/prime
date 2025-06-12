// src/environment.rs
// Environment detection and information

use crate::commands::CommandProcessor;

#[derive(Debug, Clone)]
pub struct EnvironmentInfo {
    pub os: String,
    pub python_version: Option<String>,
    pub pip_version: Option<String>,
    pub has_sudo: bool,
    pub in_venv: bool,
    pub has_git: bool,
    pub has_npm: bool,
    pub has_docker: bool,
    pub has_rust: bool,
    pub shell: Option<String>,
}

pub struct EnvironmentDetector;

impl EnvironmentDetector {
    pub fn new() -> Self {
        Self
    }
    
    pub fn detect(&self, command_processor: &CommandProcessor) -> EnvironmentInfo {
        EnvironmentInfo {
            os: std::env::consts::OS.to_string(),
            python_version: self.get_python_version(command_processor),
            pip_version: self.get_pip_version(command_processor),
            has_sudo: self.check_sudo(command_processor),
            in_venv: self.check_venv(),
            has_git: command_processor.check_command("git --version").is_some(),
            has_npm: command_processor.check_command("npm --version").is_some(),
            has_docker: command_processor.check_command("docker --version").is_some(),
            has_rust: command_processor.check_command("rustc --version").is_some(),
            shell: self.get_shell(),
        }
    }
    
    fn get_python_version(&self, command_processor: &CommandProcessor) -> Option<String> {
        // Try different python commands
        let commands = ["python --version", "python3 --version", "py --version"];
        
        for cmd in &commands {
            if let Some(output) = command_processor.check_command(cmd) {
                // Parse version from output like "Python 3.9.0"
                if let Some(version) = output.split_whitespace().nth(1) {
                    return Some(version.to_string());
                }
            }
        }
        
        None
    }
    
    fn get_pip_version(&self, command_processor: &CommandProcessor) -> Option<String> {
        // Try different pip commands
        let commands = ["pip --version", "pip3 --version", "python -m pip --version"];
        
        for cmd in &commands {
            if let Some(output) = command_processor.check_command(cmd) {
                // Parse version from output like "pip 20.2.3 from ..."
                if let Some(version_part) = output.split_whitespace().nth(1) {
                    return Some(version_part.to_string());
                }
            }
        }
        
        None
    }
    
    fn check_sudo(&self, command_processor: &CommandProcessor) -> bool {
        #[cfg(target_os = "windows")]
        {
            // Windows doesn't have sudo, check if running as admin
            false // Simplified for now
        }
        #[cfg(not(target_os = "windows"))]
        {
            command_processor.check_command("sudo -n true 2>/dev/null").is_some()
        }
    }
    
    fn check_venv(&self) -> bool {
        // Check common virtual environment variables
        std::env::var("VIRTUAL_ENV").is_ok() || 
        std::env::var("CONDA_DEFAULT_ENV").is_ok() ||
        std::env::var("PIPENV_ACTIVE").is_ok()
    }
    
    fn get_shell(&self) -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            Some("PowerShell".to_string())
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::env::var("SHELL").ok()
                .and_then(|path| {
                    path.split('/').last().map(|s| s.to_string())
                })
        }
    }
}

impl EnvironmentInfo {
    pub fn summary(&self) -> String {
        let mut parts = vec![
            format!("OS: {}", self.os),
        ];
        
        if let Some(py) = &self.python_version {
            parts.push(format!("Python: {}", py));
        }
        
        if let Some(pip) = &self.pip_version {
            parts.push(format!("Pip: {}", pip));
        }
        
        if self.in_venv {
            parts.push("Virtual Env: Active".to_string());
        }
        
        let tools: Vec<&str> = vec![
            if self.has_git { Some("git") } else { None },
            if self.has_npm { Some("npm") } else { None },
            if self.has_docker { Some("docker") } else { None },
            if self.has_rust { Some("rust") } else { None },
        ]
        .into_iter()
        .flatten()
        .collect();
        
        if !tools.is_empty() {
            parts.push(format!("Tools: {}", tools.join(", ")));
        }
        
        parts.join(" | ")
    }
}