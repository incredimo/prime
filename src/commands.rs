// commands.rs
// Command execution and processing for Prime

use std::process::{Command, Stdio};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use tempfile::NamedTempFile;

/// Handles command execution for Prime
pub struct CommandProcessor {
    // Options for command execution
    shell_command: String,
    shell_args: Vec<String>,
}

impl CommandProcessor {
    /// Create a new command processor
    pub fn new() -> Self {
        // Default shell configuration
        #[cfg(target_os = "windows")]
        let (shell_command, shell_args) = ("powershell".to_string(), vec!["-Command".to_string()]);
        
        #[cfg(not(target_os = "windows"))]
        let (shell_command, shell_args) = ("sh".to_string(), vec!["-c".to_string()]);
        
        Self {
            shell_command,
            shell_args,
        }
    }
    
    /// Execute a shell command and return its output
    pub fn execute_command(&self, command: &str) -> Result<(i32, String)> {
        println!("Executing: {}", command);
        
        // Create args by cloning shell_args and adding the command
        let mut args = self.shell_args.clone();
        args.push(command.to_string());
        
        // Execute the command
        let output = Command::new(&self.shell_command)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("Failed to execute command: {}", command))?;
        
        // Get exit code
        let exit_code = output.status.code().unwrap_or(-1);
        
        // Combine stdout and stderr for simplicity
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        let combined_output = if stderr.is_empty() {
            stdout
        } else {
            // Only include stderr if it's not empty
            if stdout.is_empty() {
                format!("STDERR:\n{}", stderr)
            } else {
                format!("{}\n\nSTDERR:\n{}", stdout, stderr)
            }
        };
        
        // Print a short summary of the result
        println!("Command completed with exit code: {}", exit_code);
        
        // Print the first few lines of output to provide immediate feedback
        let preview = combined_output.lines().take(5).collect::<Vec<&str>>().join("\n");
        if !preview.is_empty() {
            println!("Output preview:\n{}", preview);
            if combined_output.lines().count() > 5 {
                println!("... (output truncated, full output saved in conversation)");
            }
        }
        
        Ok((exit_code, combined_output))
    }
    
    /// Execute a script file
    pub fn execute_script(&self, script_content: &str) -> Result<(i32, String)> {
        // Create a temporary script file with appropriate extension
        #[cfg(target_os = "windows")]
        let mut script_file = NamedTempFile::with_suffix(".ps1")
            .context("Failed to create temporary PowerShell script file")?;
        
        #[cfg(not(target_os = "windows"))]
        let mut script_file = NamedTempFile::builder().prefix("prime_script_").suffix(".sh").tempfile()
            .context("Failed to create temporary script file")?;
        
        // Write script content to the file
        script_file.write_all(script_content.as_bytes())
            .context("Failed to write script content")?;
        
        // Make the script executable on Unix-like systems
        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt;
            let file_path = script_file.path();
            let metadata = std::fs::metadata(file_path)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755); // rwxr-xr-x
            std::fs::set_permissions(file_path, permissions)?;
        }
        
        // Execute the script using appropriate shell
        #[cfg(target_os = "windows")]
        let script_path = script_file.path().to_string_lossy().to_string();
        
        #[cfg(not(target_os = "windows"))]
        let script_path = script_file.path().to_string_lossy().to_string();
        
        // Execute the script
        self.execute_command(&format!("& '{}'", script_path))
    }
    
    /// Check if a command is potentially destructive
    pub fn is_destructive_command(&self, command: &str) -> bool {
        let command = command.trim().to_lowercase();
        
        // Look for dangerous commands - Windows and Unix/Linux versions
        #[cfg(target_os = "windows")]
        let dangerous_patterns = [
            "remove-item -recurse", "rmdir /s", "del /s", "format", "fdisk", 
            "clear-disk", "initialize-disk", "remove-partition", "diskpart"
        ];
        
        #[cfg(not(target_os = "windows"))]
        let dangerous_patterns = [
            "rm -rf", "rm -r", "rmdir", "mkfs", "fdisk", "format", "dd if=",
            "shred", ":(){:|:&};:", "chmod -R 777", "mv /* /dev/null"
        ];
        
        dangerous_patterns.iter().any(|pattern| command.contains(pattern))
    }
    
    /// Execute a command within a specific directory
    pub fn execute_in_directory(&self, command: &str, directory: &Path) -> Result<(i32, String)> {
        println!("Executing in {}: {}", directory.display(), command);
        
        // Clone shell args and add command
        let mut args = self.shell_args.clone();
        args.push(command.to_string());
        
        // Execute command in specified directory
        let output = Command::new(&self.shell_command)
            .args(&args)
            .current_dir(directory)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("Failed to execute command in {}: {}", directory.display(), command))?;
        
        // Get exit code
        let exit_code = output.status.code().unwrap_or(-1);
        
        // Combine output
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        let combined_output = if stderr.is_empty() {
            stdout
        } else {
            format!("{}\n\nSTDERR:\n{}", stdout, stderr)
        };
        
        println!("Command completed with exit code: {}", exit_code);
        
        Ok((exit_code, combined_output))
    }
}

/// Helper functions for command-related operations
pub mod command_utils {
    use std::fs;
    use std::path::Path;
    use anyhow::{Result, Context};
    
    /// Read a file's content
    pub fn read_file(path: &Path) -> Result<String> {
        fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))
    }
    
    /// Write content to a file
    pub fn write_file(path: &Path, content: &str) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directories for: {}", path.display()))?;
            }
        }
        
        // Write the file
        fs::write(path, content)
            .with_context(|| format!("Failed to write file: {}", path.display()))
    }
    
    /// Check if a directory exists
    pub fn directory_exists(path: &Path) -> bool {
        path.exists() && path.is_dir()
    }
    
    /// Create a directory if it doesn't exist
    pub fn ensure_directory_exists(path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .with_context(|| format!("Failed to create directory: {}", path.display()))?;
        }
        
        Ok(())
    }
    
    /// List directory contents
    pub fn list_directory(path: &Path) -> Result<Vec<String>> {
        let entries = fs::read_dir(path)
            .with_context(|| format!("Failed to read directory: {}", path.display()))?;
        
        let mut result = Vec::new();
        
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name();
            if let Some(name_str) = file_name.to_str() {
                result.push(name_str.to_owned());
            }
        }
        
        Ok(result)
    }
}