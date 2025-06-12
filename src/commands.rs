// src/commands.rs
// Enhanced command execution with fallback strategies

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use tempfile::NamedTempFile;

use crate::config_utils;
use crate::logging::LOG;

/// Handles script execution for Prime with smart fallback strategies
pub struct CommandProcessor {
    shell_command: String,
    shell_args: Vec<String>,
    ask_me_before_patterns: Vec<String>,
}

impl CommandProcessor {
    /// Create a new command processor
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        let (shell_command, shell_args) =
            ("powershell".to_string(), vec!["-Command".to_string()]);

        #[cfg(not(target_os = "windows"))]
        let (shell_command, shell_args) = ("sh".to_string(), vec!["-c".to_string()]);

        let ask_me_before_patterns = config_utils::load_ask_me_before_patterns().unwrap_or_else(|e| {
            LOG.error(format!(
                "Warning: Failed to load 'ask me before' patterns: {}. Using defaults.",
                e
            ));
            config_utils::DEFAULT_ASK_ME_BEFORE_PATTERNS
                .iter()
                .map(|s| s.to_string())
                .collect()
        });

        Self {
            shell_command,
            shell_args,
            ask_me_before_patterns,
        }
    }
    
    /// Get execution strategies for a command
    pub fn get_execution_strategies(&self, command: &str) -> Vec<String> {
        let mut strategies = vec![command.to_string()];
        let cmd_lower = command.to_lowercase();
        
        // pip install strategies
        if cmd_lower.starts_with("pip install") {
            let package = command.trim_start_matches("pip install").trim();
            
            // Try different pip strategies
            strategies.push(format!("pip install --user {}", package));
            strategies.push(format!("python -m pip install {}", package));
            strategies.push(format!("pip3 install {}", package));
            strategies.push(format!("python3 -m pip install {}", package));
            
            // Handle externally managed environments
            strategies.push(format!("pip install --break-system-packages {}", package));
            
            #[cfg(target_os = "windows")]
            {
                strategies.push(format!("py -m pip install {}", package));
                strategies.push(format!("py -m pip install --user {}", package));
            }
        }
        
        // python command strategies
        else if cmd_lower.starts_with("python ") {
            let args = command.trim_start_matches("python").trim();
            strategies.push(format!("python3 {}", args));
            
            #[cfg(target_os = "windows")]
            strategies.push(format!("py {}", args));
        }
        
        // npm install strategies
        else if cmd_lower.starts_with("npm install") {
            let package = command.trim_start_matches("npm install").trim();
            strategies.push(format!("npm install --global {}", package));
            strategies.push(format!("yarn add {}", package));
        }
        
        strategies
    }
    
    /// Execute command with automatic fallback strategies
    pub fn execute_with_fallbacks(&self, command: &str, working_dir: Option<&Path>) -> Result<(i32, String)> {
        let strategies = self.get_execution_strategies(command);
        
        // Check if we should ask before executing
        if self.is_ask_me_before_command(command) {
            LOG.warning(format!("This command might be destructive: {}", command));
            print!("Continue? (y/N): ");
            std::io::stdout().flush()?;
            
            let mut response = String::new();
            std::io::stdin().read_line(&mut response)?;
            
            if !response.trim().eq_ignore_ascii_case("y") {
                return Ok((1, "Command cancelled by user".to_string()));
            }
        }
        
        let mut last_result = (1, String::new());
        
        for (idx, strategy) in strategies.iter().enumerate() {
            if idx > 0 {
                LOG.info(format!("Trying alternative: {}", strategy));
            }
            
            let result = self.execute_command(strategy, working_dir)?;
            
            if result.0 == 0 {
                if idx > 0 {
                    LOG.success("Alternative succeeded!");
                }
                return Ok(result);
            }
            
            last_result = result;
            
            // Don't try more strategies if it's not a common error
            if !self.is_recoverable_error(&last_result.1) {
                break;
            }
        }
        
        // Return the last failure
        Ok(last_result)
    }
    
    /// Check if error is potentially recoverable with a different strategy
    fn is_recoverable_error(&self, output: &str) -> bool {
        let output_lower = output.to_lowercase();
        
        let recoverable_patterns = [
            "permission denied",
            "access is denied",
            "environmenterror",
            "externally-managed-environment",
            "not found",
            "is not recognized",
            "no module named pip",
            "cannot find module",
        ];
        
        recoverable_patterns.iter().any(|&pattern| output_lower.contains(pattern))
    }

    /// Execute a shell command and return its output
    pub fn execute_command(&self, command: &str, working_dir: Option<&Path>) -> Result<(i32, String)> {
        let effective_working_dir = working_dir.unwrap_or_else(|| Path::new("."));

        LOG.executing(&command, effective_working_dir.display());

        let mut args = self.shell_args.clone();
        args.push(command.to_string());

        let output = Command::new(&self.shell_command)
            .args(&args)
            .current_dir(effective_working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("Failed to execute command: {}", command))?;

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let combined_output = if stderr.is_empty() {
            stdout
        } else if stdout.is_empty() {
            stderr
        } else {
            format!("{}\n\nSTDERR:\n{}", stdout, stderr)
        };

        LOG.command_status(exit_code);
        
        // Show output preview
        LOG.command_output(&combined_output);
        
        Ok((exit_code, combined_output))
    }

    /// Execute a script file
    pub fn execute_script(&self, script_content: &str, working_dir: Option<&Path>) -> Result<(i32, String)> {
        #[cfg(target_os = "windows")]
        let mut script_file = NamedTempFile::new().context("Failed to create temporary PowerShell script file")?;

        #[cfg(not(target_os = "windows"))]
        let mut script_file = NamedTempFile::new().context("Failed to create temporary shell script file")?;

        script_file
            .write_all(script_content.as_bytes())
            .context("Failed to write script content")?;
        
        script_file.flush()?;

        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt;
            let file_path = script_file.path();
            let metadata = fs::metadata(file_path)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755); // rwxr-xr-x
            fs::set_permissions(file_path, permissions)?;
        }

        let script_path_str = script_file.path().to_string_lossy().to_string();
        
        #[cfg(target_os = "windows")]
        let command_to_execute = format!("& '{}'", script_path_str);
        #[cfg(not(target_os = "windows"))]
        let command_to_execute = script_path_str;

        self.execute_command(&command_to_execute, working_dir)
    }

    /// Check if a command matches any "ask me before" patterns
    pub fn is_ask_me_before_command(&self, command: &str) -> bool {
        let command_lower = command.trim().to_lowercase();
        self.ask_me_before_patterns
            .iter()
            .any(|pattern| command_lower.contains(&pattern.to_lowercase()))
    }
    
    /// Execute a simple command for environment detection (no output)
    pub fn check_command(&self, command: &str) -> Option<String> {
        match self.execute_command(command, None) {
            Ok((0, output)) => Some(output.lines().next().unwrap_or("").trim().to_string()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_command_processor_creation() {
        let processor = CommandProcessor::new();
        #[cfg(target_os = "windows")]
        {
            assert_eq!(processor.shell_command, "powershell");
            assert_eq!(processor.shell_args, vec!["-Command"]);
        }
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(processor.shell_command, "sh");
            assert_eq!(processor.shell_args, vec!["-c"]);
        }
    }

    #[test]
    fn test_execution_strategies() {
        let processor = CommandProcessor::new();
        
        // Test pip strategies
        let pip_strategies = processor.get_execution_strategies("pip install requests");
        assert!(pip_strategies.len() >= 4);
        assert!(pip_strategies.contains(&"pip install requests".to_string()));
        assert!(pip_strategies.contains(&"pip install --user requests".to_string()));
        assert!(pip_strategies.contains(&"python -m pip install requests".to_string()));
        
        // Test python strategies
        let py_strategies = processor.get_execution_strategies("python script.py");
        assert!(py_strategies.contains(&"python script.py".to_string()));
        assert!(py_strategies.contains(&"python3 script.py".to_string()));
    }

    #[test]
    fn test_simple_command_execution() {
        let processor = CommandProcessor::new();
        let temp_dir = env::temp_dir();
        
        #[cfg(target_os = "windows")]
        let test_command = "echo 'Hello World'";
        #[cfg(not(target_os = "windows"))]
        let test_command = "echo 'Hello World'";
        
        let result = processor.execute_command(test_command, Some(&temp_dir));
        assert!(result.is_ok());
        
        let (exit_code, output) = result.unwrap();
        assert_eq!(exit_code, 0);
        assert!(output.contains("Hello World"));
    }

    #[test]
    fn test_recoverable_error_detection() {
        let processor = CommandProcessor::new();
        
        assert!(processor.is_recoverable_error("Permission denied"));
        assert!(processor.is_recoverable_error("ERROR: Could not install packages due to an EnvironmentError"));
        assert!(processor.is_recoverable_error("command not found"));
        assert!(!processor.is_recoverable_error("Syntax error in script"));
    }

    #[test]
    fn test_ask_me_before_patterns() {
        let processor = CommandProcessor::new();
        
        #[cfg(target_os = "windows")]
        {
            assert!(processor.is_ask_me_before_command("Remove-Item -Recurse"));
            assert!(processor.is_ask_me_before_command("remove-item -recurse"));
            assert!(!processor.is_ask_me_before_command("Get-ChildItem"));
        }
        #[cfg(not(target_os = "windows"))]
        {
            assert!(processor.is_ask_me_before_command("rm -rf /important/folder"));
            assert!(processor.is_ask_me_before_command("RM -RF"));
            assert!(!processor.is_ask_me_before_command("ls -la"));
        }
    }
}