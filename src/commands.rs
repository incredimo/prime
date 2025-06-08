// src/commands.rs
// Command execution and processing for Prime

use std::fs;
use std::io::{BufRead, BufReader, Write, Read};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use glob::Pattern; // Added for glob pattern matching
use tempfile::NamedTempFile;

use crate::config_utils; // Added for loading configurations
use crate::styling::STYLER;

const MAX_FILE_READ_LINES: usize = 1000; // Max lines to read from a file
const MAX_FILE_READ_BYTES: u64 = 1_048_576; // 1MB max bytes to read
const MAX_DIR_LISTING_CHILDREN_DISPLAY: usize = 20; // Max children to list before truncating

/// Handles command execution and file operations for Prime
pub struct CommandProcessor {
    shell_command: String,
    shell_args: Vec<String>,
    ignored_path_patterns: Vec<Pattern>,
    ask_me_before_patterns: Vec<String>,
    // Base path for resolving relative paths, typically the workspace root.
    // This needs to be set or passed in if commands/file ops can specify relative paths.
    // For now, assuming paths passed to file ops are absolute or relative to current_dir of execution.
    // If LLM provides relative paths, they should be resolved against a known base.
    // Let's assume for now that the `PrimeSession` or `Prime` struct will manage the
    // current working directory context for the LLM and resolve paths before calling these.
}

impl CommandProcessor {
    /// Create a new command processor
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        let (shell_command, shell_args) =
            ("powershell".to_string(), vec!["-Command".to_string()]);

        #[cfg(not(target_os = "windows"))]
        let (shell_command, shell_args) = ("sh".to_string(), vec!["-c".to_string()]);

        let ignored_path_patterns = config_utils::load_ignored_path_patterns().unwrap_or_else(|e| {
            eprintln!(
                "{}",
                STYLER.error_style(format!(
                    "Warning: Failed to load ignored path patterns: {}. Using defaults.",
                    e
                ))
            );
            config_utils::DEFAULT_IGNORED_PATHS
                .iter()
                .filter_map(|s| Pattern::new(s).ok())
                .collect()
        });

        let ask_me_before_patterns = config_utils::load_ask_me_before_patterns().unwrap_or_else(|e| {
            eprintln!(
                "{}",
                STYLER.error_style(format!(
                    "Warning: Failed to load 'ask me before' patterns: {}. Using defaults.",
                    e
                ))
            );
            config_utils::DEFAULT_ASK_ME_BEFORE_PATTERNS
                .iter()
                .map(|s| s.to_string())
                .collect()
        });

        Self {
            shell_command,
            shell_args,
            ignored_path_patterns,
            ask_me_before_patterns,
        }
    }

    /// Execute a shell command and return its output
    pub fn execute_command(&self, command: &str, working_dir: Option<&Path>) -> Result<(i32, String)> {
        let effective_working_dir = working_dir.unwrap_or_else(|| Path::new(".")); // Default to current dir if None

        println!(
            "{}",
            STYLER.command_exec_style(format!(
                "Executing in '{}': {}",
                effective_working_dir.display(),
                command
            ))
        );

        let mut args = self.shell_args.clone();
        args.push(command.to_string());

        let output = Command::new(&self.shell_command)
            .args(&args)
            .current_dir(effective_working_dir) // Use the specified or default working directory
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
            format!("STDERR:\n{}", stderr)
        } else {
            format!("{}\n\nSTDERR:\n{}", stdout, stderr)
        };

        println!(
            "{}",
            STYLER.dim_gray_style(format!(
                "Command completed with exit code: {}",
                exit_code
            ))
        );
        let preview = combined_output.lines().take(5).collect::<Vec<&str>>().join("\n");
        if !preview.is_empty() {
            println!(
                "{}",
                STYLER.dim_gray_style(format!("Output preview:\n{}", preview))
            );
            if combined_output.lines().count() > 5 {
                println!(
                    "{}",
                    STYLER.dim_gray_style(
                        "... (output truncated, full output saved in conversation)"
                    )
                );
            }
        }
        Ok((exit_code, combined_output))
    }

    /// Execute a script file
    pub fn execute_script(&self, script_content: &str, working_dir: Option<&Path>) -> Result<(i32, String)> {
        #[cfg(target_os = "windows")]
        let mut script_file = NamedTempFile::with_suffix(".ps1")
            .context("Failed to create temporary PowerShell script file")?;

        #[cfg(not(target_os = "windows"))]
        let mut script_file = NamedTempFile::builder()
            .prefix("prime_script_")
            .suffix(".sh")
            .tempfile()
            .context("Failed to create temporary script file")?;

        script_file
            .write_all(script_content.as_bytes())
            .context("Failed to write script content")?;

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
        
        // On Windows, PowerShell needs `& ` to execute a script if the path contains spaces or needs quoting.
        // For simplicity and robustness, always use it.
        #[cfg(target_os = "windows")]
        let command_to_execute = format!("& '{}'", script_path_str);
        #[cfg(not(target_os = "windows"))]
        let command_to_execute = script_path_str; // On Unix, just the path is usually enough for `sh -c`

        self.execute_command(&command_to_execute, working_dir)
    }


    /// Check if a command matches any "ask me before" patterns
    pub fn is_ask_me_before_command(&self, command: &str) -> bool {
        let command_lower = command.trim().to_lowercase();
        self.ask_me_before_patterns
            .iter()
            .any(|pattern| command_lower.contains(&pattern.to_lowercase()))
    }

    // --- New public file operation methods ---

    /// Read a file's content with limits to prevent context overflow.
    pub fn read_file_to_string_with_limit(&self, path: &Path) -> Result<(String, bool)> {
        command_utils::read_file_to_string_with_limit(path)
    }

    /// Write content to a file.
    pub fn write_file_to_path(&self, path: &Path, content: &str) -> Result<()> {
        command_utils::write_file(path, content)
    }

    /// List directory contents smartly (respecting ignores, truncating long lists).
    pub fn list_directory_smart(&self, path: &Path) -> Result<Vec<String>> {
        command_utils::list_directory_smart(path, &self.ignored_path_patterns)
    }
}

/// Helper functions for command-related operations
pub mod command_utils {
    use super::{MAX_DIR_LISTING_CHILDREN_DISPLAY, MAX_FILE_READ_BYTES, MAX_FILE_READ_LINES};
    use anyhow::{Context, Result};
    use glob::Pattern;
    use std::fs;
    use std::io::{BufRead, BufReader, Read};
    use std::path::Path;

    /// Read a file's content with limits.
    /// Returns the content and a boolean indicating if it was truncated.
    pub fn read_file_to_string_with_limit(path: &Path) -> Result<(String, bool)> {
        let file = fs::File::open(path)
            .with_context(|| format!("Failed to open file: {}", path.display()))?;
        let metadata = file.metadata()
            .with_context(|| format!("Failed to read metadata for file: {}", path.display()))?;

        let mut truncated = false;
        let mut content = String::new();

        if metadata.len() > MAX_FILE_READ_BYTES {
            truncated = true;
            let mut reader = BufReader::new(file);
            let mut buffer = vec![0; MAX_FILE_READ_BYTES as usize];
            let bytes_read = reader.read(&mut buffer)
                .with_context(|| format!("Failed to read initial chunk of large file: {}", path.display()))?;
            content = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
            // Further truncate by lines if needed
            let lines: Vec<&str> = content.lines().take(MAX_FILE_READ_LINES).collect();
            content = lines.join("\n");
            if content.lines().count() >= MAX_FILE_READ_LINES && metadata.len() > MAX_FILE_READ_BYTES {
                 // ensure it's marked truncated if either limit is hit.
                truncated = true;
            }

        } else {
            let mut reader = BufReader::new(file);
            let mut line_count = 0;
            for line_result in reader.lines() {
                let line = line_result.with_context(|| format!("Failed to read line from file: {}", path.display()))?;
                if line_count >= MAX_FILE_READ_LINES {
                    truncated = true;
                    break;
                }
                content.push_str(&line);
                content.push('\n');
                line_count += 1;
            }
        }
        if truncated {
            content.push_str("\n... (file truncated due to size or line limit)");
        }
        Ok((content, truncated))
    }

    /// Write content to a file
    pub fn write_file(path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directories for: {}", path.display()))?;
            }
        }
        fs::write(path, content)
            .with_context(|| format!("Failed to write file: {}", path.display()))
    }

    /// List directory contents smartly.
    pub fn list_directory_smart(path: &Path, ignored_patterns: &[Pattern]) -> Result<Vec<String>> {
        if !path.is_dir() {
            return Err(anyhow::anyhow!("Path is not a directory: {}", path.display()));
        }

        let entries = fs::read_dir(path)
            .with_context(|| format!("Failed to read directory: {}", path.display()))?;

        let mut items = Vec::new();
        for entry_result in entries {
            let entry = entry_result.with_context(|| format!("Error reading directory entry in {}", path.display()))?;
            let entry_path = entry.path();
            let file_name_os = entry.file_name();
            let file_name = file_name_os.to_string_lossy();

            // Check against ignored patterns
            if ignored_patterns.iter().any(|p| p.matches_path(&entry_path) || p.matches(&file_name)) {
                continue;
            }

            let display_name = if entry_path.is_dir() {
                format!("{}/", file_name)
            } else {
                file_name.into_owned()
            };
            items.push(display_name);
        }

        items.sort_by(|a, b| { // Sort alphabetically, directories first
            let a_is_dir = a.ends_with('/');
            let b_is_dir = b.ends_with('/');
            if a_is_dir && !b_is_dir {
                std::cmp::Ordering::Less
            } else if !a_is_dir && b_is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.to_lowercase().cmp(&b.to_lowercase())
            }
        });
        
        if items.len() > MAX_DIR_LISTING_CHILDREN_DISPLAY {
            let mut truncated_items: Vec<String> = items.iter().take(MAX_DIR_LISTING_CHILDREN_DISPLAY).cloned().collect();
            truncated_items.push(format!("... (and {} more items)", items.len() - MAX_DIR_LISTING_CHILDREN_DISPLAY));
            Ok(truncated_items)
        } else {
            Ok(items)
        }
    }

    /// Check if a directory exists (remains for internal use if needed, but list_directory_smart implies existence)
    #[allow(dead_code)]
    pub fn directory_exists(path: &Path) -> bool {
        path.exists() && path.is_dir()
    }

    /// Create a directory if it doesn't exist (remains for internal use if needed)
    #[allow(dead_code)]
    pub fn ensure_directory_exists(path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .with_context(|| format!("Failed to create directory: {}", path.display()))?;
        }
        Ok(())
    }

    // --- File Operation Handlers ---

    /// Handles creating a file.
    pub fn handle_create_file(path: &Path, content: &str, overwrite: bool) -> Result<String> {
        if path.exists() && !overwrite {
            return Err(anyhow::anyhow!("File already exists and overwrite is false."));
        }
        write_file(path, content)
            .with_context(|| format!("Failed to create file: {}", path.display()))?;
        Ok("File created successfully.".to_string())
    }

    /// Handles reading a file.
    pub fn handle_read_file(path: &Path) -> Result<String> {
        if !path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", path.display()));
        }
        // The boolean for truncation can be ignored here or logged.
        // The content string itself will indicate truncation if it happened.
        read_file_to_string_with_limit(path)
            .map(|(content, _)| content)
            .with_context(|| format!("Failed to read file: {}", path.display()))
    }

    /// Handles updating an existing file.
    pub fn handle_update_file(path: &Path, content: &str) -> Result<String> {
        if !path.exists() {
            return Err(anyhow::anyhow!("File not found for update: {}", path.display()));
        }
        write_file(path, content) // write_file overwrites by default
            .with_context(|| format!("Failed to update file: {}", path.display()))?;
        Ok("File updated successfully.".to_string())
    }

    /// Handles deleting a file or directory.
    pub fn handle_delete_file(path: &Path, recursive: bool) -> Result<String> {
        if !path.exists() {
            return Err(anyhow::anyhow!("File or directory not found for deletion: {}", path.display()));
        }

        if path.is_dir() {
            if recursive {
                fs::remove_dir_all(path)
                    .with_context(|| format!("Failed to recursively delete directory: {}", path.display()))?;
            } else {
                // Check if directory is empty before attempting non-recursive delete
                if fs::read_dir(path)?.next().is_some() {
                    return Err(anyhow::anyhow!(
                        "Path is a non-empty directory and recursive delete is false: {}",
                        path.display()
                    ));
                }
                fs::remove_dir(path)
                    .with_context(|| format!("Failed to delete empty directory: {}. Ensure it is empty or use recursive delete.", path.display()))?;
            }
        } else {
            fs::remove_file(path)
                .with_context(|| format!("Failed to delete file: {}", path.display()))?;
        }
        Ok("File/directory deleted successfully.".to_string())
    }

    /// Handles patching a file using a diff.
    pub fn handle_patch_file(path: &Path, diff_content: &str, _diff_format: &str) -> Result<String> {
        // Assuming _diff_format is "unified" for now.
        // This function depends on the `patch` crate.
        if !path.exists() {
            return Err(anyhow::anyhow!("File not found for patching: {}", path.display()));
        }

        let original_content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read original file for patching: {}", path.display()))?;

        let patch_obj = patch::Patch::from_str(diff_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse diff content: {}", e))
            .with_context(|| "Error parsing patch object from diff string")?;

        let patched_content_cow = patch::apply(&original_content, &patch_obj)
            .map_err(|e| anyhow::anyhow!("Failed to apply patch: {}", e))
            .with_context(|| format!("Error applying patch to file: {}", path.display()))?;

        let patched_content_string = patched_content_cow.into_owned();

        fs::write(path, patched_content_string)
            .with_context(|| format!("Failed to write patched content to file: {}", path.display()))?;

        Ok("File patched successfully.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::command_utils::*; // Imports handlers from command_utils
    use std::fs;
    use tempfile::tempdir; // For creating temporary directories
    use std::path::PathBuf;

    // Helper to create a temp file with content
    fn create_temp_file(dir: &tempfile::TempDir, name: &str, content: &str) -> PathBuf {
        let file_path = dir.path().join(name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directory for temp file");
        }
        fs::write(&file_path, content).expect("Failed to create temp file for test");
        file_path
    }

    // Tests for handle_create_file
    #[test]
    fn test_handle_create_file_success() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("new_file.txt");
        let content = "Hello, world!";

        let result = handle_create_file(&file_path, content, false).unwrap();
        assert!(result.contains("successfully"));
        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), content);
    }

    #[test]
    fn test_handle_create_file_overwrite_true() {
        let dir = tempdir().unwrap();
        let file_path = create_temp_file(&dir, "existing.txt", "Old content");
        let new_content = "New content";

        let result = handle_create_file(&file_path, new_content, true).unwrap();
        assert!(result.contains("successfully"));
        assert_eq!(fs::read_to_string(&file_path).unwrap(), new_content);
    }

    #[test]
    fn test_handle_create_file_overwrite_false_error() {
        let dir = tempdir().unwrap();
        let file_path = create_temp_file(&dir, "existing.txt", "Old content");

        let result = handle_create_file(&file_path, "Attempt to overwrite", false);
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("File already exists"));
    }

    // Tests for handle_read_file
    #[test]
    fn test_handle_read_file_success() {
        let dir = tempdir().unwrap();
        let content = "Content to be read\nWith multiple lines\nAnd special chars!@#$%^&*()";
        // The read_file_to_string_with_limit function in command_utils adds a newline if not present
        // and the original content does not end with one and is within limits.
        // For exact match, ensure test content aligns or trim the result.
        // Here, we'll test with content that will be read as is.
        let file_path = create_temp_file(&dir, "readable.txt", content);

        let result = handle_read_file(&file_path).unwrap();
        // handle_read_file uses read_file_to_string_with_limit, which might add a trailing newline
        // if the original content doesn't have one and line limits are not hit.
        // The provided content for `create_temp_file` will be written as is.
        // `read_file_to_string_with_limit` reconstructs content line by line, adding `\n`.
        // So, if `content` doesn't end with `\n`, the read version will have one more.
        let expected_content = if content.ends_with('\n') || content.is_empty() {
            content.to_string()
        } else {
            format!("{}\n", content)
        };
        // If the file is truncated, it will have a specific message.
        // This test assumes no truncation.
        let expected_content_final = if expected_content.contains("\n... (file truncated due to size or line limit)") {
             expected_content
        } else {
            expected_content.trim_end_matches('\n').to_string()
        };
        // The current implementation of `read_file_to_string_with_limit` adds a newline to each line.
        // and then `handle_read_file` returns that. If the original content doesn't end with newline,
        // the read version will.
        // Let's adjust expectation based on `read_file_to_string_with_limit` behavior.
        // It reads lines and appends '\n'. So "text" becomes "text\n".
        // "text\n" becomes "text\n\n" effectively if not careful.
        // The current `read_file_to_string_with_limit` adds a `\n` after each line it reads.
        // And if truncated, it adds a specific message.
        // The `handle_read_file` simply returns this.
        // Let's simplify: the test content has newlines.
        let content_for_test = "Content to be read\nWith multiple lines\nAnd special chars!@#$%^&*()\n";
        let file_path_for_test = create_temp_file(&dir, "readable_complex.txt", content_for_test);
        let result_complex = handle_read_file(&file_path_for_test).unwrap();
        assert_eq!(result_complex.trim_end(), content_for_test.trim_end());


        let simple_content = "Simple";
        let file_path_simple = create_temp_file(&dir, "readable_simple.txt", simple_content);
        let result_simple = handle_read_file(&file_path_simple).unwrap();
        assert_eq!(result_simple, format!("{}\n",simple_content));


    }

    #[test]
    fn test_handle_read_file_not_found_error() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("non_existent.txt");

        let result = handle_read_file(&file_path);
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("File not found"));
    }

    // Tests for handle_update_file
    #[test]
    fn test_handle_update_file_success() {
        let dir = tempdir().unwrap();
        let file_path = create_temp_file(&dir, "updatable.txt", "Initial content");
        let new_content = "Updated content";

        let result = handle_update_file(&file_path, new_content).unwrap();
        assert!(result.contains("successfully"));
        assert_eq!(fs::read_to_string(&file_path).unwrap(), new_content);
    }

    #[test]
    fn test_handle_update_file_not_found_error() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("non_existent_for_update.txt");

        let result = handle_update_file(&file_path, "data");
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("File not found for update"));
    }

    // Tests for handle_delete_file
    #[test]
    fn test_handle_delete_file_success() {
        let dir = tempdir().unwrap();
        let file_path = create_temp_file(&dir, "deletable_file.txt", "content");

        let result = handle_delete_file(&file_path, false).unwrap();
        assert!(result.contains("successfully"));
        assert!(!file_path.exists());
    }

    #[test]
    fn test_handle_delete_empty_dir_success() {
        let dir = tempdir().unwrap();
        let sub_dir_path = dir.path().join("empty_sub_dir");
        fs::create_dir(&sub_dir_path).unwrap();

        let result = handle_delete_file(&sub_dir_path, false).unwrap();
        assert!(result.contains("successfully"));
        assert!(!sub_dir_path.exists());
    }

    #[test]
    fn test_handle_delete_non_empty_dir_recursive_true_success() {
        let dir = tempdir().unwrap();
        let sub_dir_path = dir.path().join("sub_dir_with_content");
        fs::create_dir(&sub_dir_path).unwrap();
        // create_temp_file helper needs to know the full path relative to tempdir's root for nested files.
        create_temp_file(&dir, "sub_dir_with_content/some_file.txt", "content");

        let result = handle_delete_file(&sub_dir_path, true).unwrap();
        assert!(result.contains("successfully"));
        assert!(!sub_dir_path.exists());
    }

    #[test]
    fn test_handle_delete_non_empty_dir_recursive_false_error() {
        let dir = tempdir().unwrap();
        let sub_dir_path = dir.path().join("sub_dir_error");
        fs::create_dir(&sub_dir_path).unwrap();
        create_temp_file(&dir, "sub_dir_error/some_file.txt", "content");

        let result = handle_delete_file(&sub_dir_path, false);
        assert!(result.is_err());
        let error_message = result.err().unwrap().to_string();
        // Error message for non-empty directory without recursive delete
        assert!(error_message.contains("Path is a non-empty directory and recursive delete is false"));
    }

    #[test]
    fn test_handle_delete_file_not_found_error() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("non_existent_for_delete.txt");
        let result = handle_delete_file(&file_path, false);
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("not found for deletion"));
    }

    // Tests for handle_patch_file
    #[test]
    fn test_handle_patch_file_success() {
        let dir = tempdir().unwrap();
        let original_content = "Line 1\nLine 2\nLine 3\n";
        let file_path = create_temp_file(&dir, "patch_target.txt", original_content);

        let diff_content = "--- a/patch_target.txt\n+++ b/patch_target.txt\n@@ -1,3 +1,4 @@\n Line 1\n-Line 2\n+Line Two\n Line 3\n+Line 4\n";
        let expected_content = "Line 1\nLine Two\nLine 3\nLine 4\n";

        let result = handle_patch_file(&file_path, diff_content, "unified").unwrap();
        assert!(result.contains("successfully"));
        assert_eq!(fs::read_to_string(&file_path).unwrap(), expected_content);
    }

    #[test]
    fn test_handle_patch_file_not_found_error() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("non_existent_for_patch.txt");
        let diff_content = "--- a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-a\n+b\n";

        let result = handle_patch_file(&file_path, diff_content, "unified");
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("File not found for patching"));
    }

    #[test]
    fn test_handle_patch_file_invalid_diff_error() {
        let dir = tempdir().unwrap();
        let file_path = create_temp_file(&dir, "patch_invalid_diff.txt", "content");
        let diff_content = "this is not a valid diff";

        let result = handle_patch_file(&file_path, diff_content, "unified");
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("Failed to parse diff content"));
    }
}