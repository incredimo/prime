use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

use regex::Regex;
use ignore::WalkBuilder;

pub struct SearchResult {
    pub path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
}

pub fn search_files(pattern: &str, path: &Path) -> Result<Vec<SearchResult>> {
    let regex = Regex::new(pattern).context("Invalid regex pattern")?;
    let mut results = Vec::new();

    let walker = WalkBuilder::new(path)
        .hidden(false) // Search hidden files too if needed, but usually gitignore handles it
        .git_ignore(true) // Respect .gitignore
        .build();

    for result in walker {
        match result {
            Ok(entry) => {
                if entry.file_type().map_or(false, |ft| ft.is_file()) {
                    let path = entry.path();
                    // Skip binary files check could be added here
                    if let Ok(content) = std::fs::read_to_string(path) {
                        for (i, line) in content.lines().enumerate() {
                            if regex.is_match(line) {
                                results.push(SearchResult {
                                    path: path.to_path_buf(),
                                    line_number: i + 1,
                                    line_content: line.trim().to_string(),
                                });
                                // Limit results per file or total if needed? 
                                // For now, let's keep it simple.
                            }
                        }
                    }
                }
            }
            Err(err) => eprintln!("Error walking directory: {}", err),
        }
    }

    Ok(results)
}
