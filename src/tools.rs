use anyhow::Result;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: Value,
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub tool_name: String,
    pub output: String,
    pub success: bool,
}

pub trait Tool {
    fn name(&self) -> &str;
    fn execute(&self, input: &Value) -> Result<String>;
}

pub struct ToolExecutor;

impl ToolExecutor {
    pub fn execute(tool_call: &ToolCall) -> ToolResult {
        let result = match tool_call.name.as_str() {
            "execute_command" => Self::execute_command(&tool_call.input),
            "read_file" => Self::read_file(&tool_call.input),
            "write_file" => Self::write_file(&tool_call.input),
            "edit_file" => Self::edit_file(&tool_call.input),
            "list_directory" => Self::list_directory(&tool_call.input),
            "search_files" => Self::search_files(&tool_call.input),
            "search_content" => Self::search_content(&tool_call.input),
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_call.name)),
        };

        match result {
            Ok(output) => ToolResult {
                tool_call_id: tool_call.id.clone(),
                tool_name: tool_call.name.clone(),
                output,
                success: true,
            },
            Err(e) => ToolResult {
                tool_call_id: tool_call.id.clone(),
                tool_name: tool_call.name.clone(),
                output: format!("Error: {}", e),
                success: false,
            },
        }
    }

    fn execute_command(input: &Value) -> Result<String> {
        let command = input
            .get("command")
            .and_then(|c| c.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter"))?;

        let working_dir = input
            .get("working_dir")
            .and_then(|w| w.as_str())
            .map(PathBuf::from);

        let output = if cfg!(windows) {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", command]);
            if let Some(dir) = working_dir {
                cmd.current_dir(dir);
            }
            cmd.output()?
        } else {
            let mut cmd = Command::new("sh");
            cmd.args(["-c", command]);
            if let Some(dir) = working_dir {
                cmd.current_dir(dir);
            }
            cmd.output()?
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = String::new();
        if !stdout.is_empty() {
            result.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push_str("\n");
            }
            result.push_str("[stderr]\n");
            result.push_str(&stderr);
        }

        if result.is_empty() {
            result = format!("Command completed with exit code: {}", output.status.code().unwrap_or(-1));
        }

        Ok(result)
    }

    fn read_file(input: &Value) -> Result<String> {
        let path = input
            .get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

        let content = std::fs::read_to_string(path)?;

        // Add line numbers
        let numbered: String = content
            .lines()
            .enumerate()
            .map(|(i, line)| format!("{:4} │ {}", i + 1, line))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(numbered)
    }

    fn write_file(input: &Value) -> Result<String> {
        let path = input
            .get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

        let content = input
            .get("content")
            .and_then(|c| c.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;

        // Create parent directories if needed
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content)?;

        Ok(format!("Successfully wrote {} bytes to {}", content.len(), path))
    }

    fn edit_file(input: &Value) -> Result<String> {
        let path = input
            .get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

        let old_text = input
            .get("old_text")
            .and_then(|o| o.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'old_text' parameter"))?;

        let new_text = input
            .get("new_text")
            .and_then(|n| n.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'new_text' parameter"))?;

        let content = std::fs::read_to_string(path)?;

        if !content.contains(old_text) {
            return Err(anyhow::anyhow!(
                "Could not find the specified text to replace in {}",
                path
            ));
        }

        let new_content = content.replace(old_text, new_text);
        std::fs::write(path, &new_content)?;

        Ok(format!(
            "Successfully edited {}. Replaced {} occurrences.",
            path,
            content.matches(old_text).count()
        ))
    }

    fn list_directory(input: &Value) -> Result<String> {
        let path = input
            .get("path")
            .and_then(|p| p.as_str())
            .unwrap_or(".");

        let entries = std::fs::read_dir(path)?;

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();

            if metadata.is_dir() {
                dirs.push(format!("📁 {}/", name));
            } else {
                let size = metadata.len();
                let size_str = if size < 1024 {
                    format!("{} B", size)
                } else if size < 1024 * 1024 {
                    format!("{:.1} KB", size as f64 / 1024.0)
                } else {
                    format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
                };
                files.push(format!("📄 {} ({})", name, size_str));
            }
        }

        dirs.sort();
        files.sort();

        let mut result = format!("Contents of {}:\n\n", path);
        for dir in dirs {
            result.push_str(&dir);
            result.push('\n');
        }
        for file in files {
            result.push_str(&file);
            result.push('\n');
        }

        Ok(result)
    }

    fn search_files(input: &Value) -> Result<String> {
        let pattern = input
            .get("pattern")
            .and_then(|p| p.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'pattern' parameter"))?;

        let base_path = input
            .get("path")
            .and_then(|p| p.as_str())
            .unwrap_or(".");

        let mut matches = Vec::new();
        Self::search_files_recursive(Path::new(base_path), pattern, &mut matches)?;

        if matches.is_empty() {
            Ok(format!("No files matching '{}' found in {}", pattern, base_path))
        } else {
            Ok(format!(
                "Found {} files matching '{}':\n{}",
                matches.len(),
                pattern,
                matches.join("\n")
            ))
        }
    }

    fn search_files_recursive(dir: &Path, pattern: &str, matches: &mut Vec<String>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        let glob_pattern = glob::Pattern::new(pattern)?;

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Skip hidden directories and common non-essential dirs
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "node_modules" && name != "target" {
                    Self::search_files_recursive(&path, pattern, matches)?;
                }
            } else {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if glob_pattern.matches(file_name) {
                    matches.push(path.display().to_string());
                }
            }
        }

        Ok(())
    }

    fn search_content(input: &Value) -> Result<String> {
        let query = input
            .get("query")
            .and_then(|q| q.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'query' parameter"))?;

        let base_path = input
            .get("path")
            .and_then(|p| p.as_str())
            .unwrap_or(".");

        let file_pattern = input
            .get("file_pattern")
            .and_then(|f| f.as_str());

        let regex = regex::Regex::new(query)?;
        let mut results = Vec::new();

        Self::search_content_recursive(
            Path::new(base_path),
            &regex,
            file_pattern,
            &mut results,
        )?;

        if results.is_empty() {
            Ok(format!("No matches for '{}' found", query))
        } else {
            Ok(format!("Found {} matches:\n\n{}", results.len(), results.join("\n\n")))
        }
    }

    fn search_content_recursive(
        dir: &Path,
        regex: &regex::Regex,
        file_pattern: Option<&str>,
        results: &mut Vec<String>,
    ) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        let glob_pattern = file_pattern.map(|p| glob::Pattern::new(p).ok()).flatten();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "node_modules" && name != "target" {
                    Self::search_content_recursive(&path, regex, file_pattern, results)?;
                }
            } else {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Check file pattern
                if let Some(ref pattern) = glob_pattern {
                    if !pattern.matches(file_name) {
                        continue;
                    }
                }

                // Try to read file (skip binary files)
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for (line_num, line) in content.lines().enumerate() {
                        if regex.is_match(line) {
                            results.push(format!(
                                "{}:{}: {}",
                                path.display(),
                                line_num + 1,
                                line.trim()
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
