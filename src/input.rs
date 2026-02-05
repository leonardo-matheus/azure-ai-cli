use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Editor};
use rustyline_derive::Helper;
use std::borrow::Cow;
use std::path::Path;

const COMMANDS: &[(&str, &str)] = &[
    ("/help", "Show available commands"),
    ("/exit", "Exit the CLI"),
    ("/quit", "Exit the CLI"),
    ("/clear", "Clear conversation history"),
    ("/model", "List and switch models"),
    ("/config", "Show current configuration"),
    ("/history", "Show conversation history"),
    ("/add-model", "Add a new model"),
    ("/lang", "Change language (en/pt)"),
];

#[derive(Helper)]
pub struct InputHelper {
    pub model_names: Vec<String>,
}

impl InputHelper {
    pub fn new(model_names: Vec<String>) -> Self {
        Self { model_names }
    }

    pub fn update_models(&mut self, model_names: Vec<String>) {
        self.model_names = model_names;
    }

    fn complete_command(&self, line: &str) -> Vec<Pair> {
        let mut matches = Vec::new();

        if line.starts_with('/') {
            let input = line.to_lowercase();
            for (cmd, desc) in COMMANDS {
                if cmd.starts_with(&input) {
                    matches.push(Pair {
                        display: format!("{} - {}", cmd, desc),
                        replacement: cmd.to_string(),
                    });
                }
            }
        }

        matches
    }

    fn complete_file(&self, line: &str, pos: usize) -> (usize, Vec<Pair>) {
        let before_cursor = &line[..pos];

        if let Some(at_pos) = before_cursor.rfind('@') {
            let partial_path = &before_cursor[at_pos + 1..];

            // Determine directory and prefix
            let (dir, prefix) = if partial_path.contains('/') || partial_path.contains('\\') {
                let path = Path::new(partial_path);
                if let Some(parent) = path.parent() {
                    let file_prefix = path.file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or("");
                    (parent.to_string_lossy().to_string(), file_prefix.to_string())
                } else {
                    (".".to_string(), partial_path.to_string())
                }
            } else {
                (".".to_string(), partial_path.to_string())
            };

            let mut matches = Vec::new();
            let search_dir = if dir.is_empty() { "." } else { &dir };

            // Collect files with metadata for sorting
            let mut files_with_time: Vec<(String, String, bool, std::time::SystemTime)> = Vec::new();

            if let Ok(entries) = std::fs::read_dir(search_dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        let name = entry.file_name().to_string_lossy().to_string();

                        // Skip hidden files unless searching for them
                        if name.starts_with('.') && !prefix.starts_with('.') {
                            continue;
                        }

                        // Skip common ignored directories
                        if name == "node_modules" || name == "target" || name == ".git" {
                            continue;
                        }

                        // Filter by prefix (case insensitive)
                        if prefix.is_empty() || name.to_lowercase().starts_with(&prefix.to_lowercase()) {
                            let is_dir = metadata.is_dir();
                            let full_path = if dir == "." {
                                name.clone()
                            } else {
                                format!("{}/{}", dir, name)
                            };

                            let modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                            files_with_time.push((name, full_path, is_dir, modified));
                        }
                    }
                }
            }

            // Sort: directories first, then by modification time (most recent first)
            files_with_time.sort_by(|a, b| {
                match (a.2, b.2) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => b.3.cmp(&a.3),
                }
            });

            // Take top 6 results
            for (name, full_path, is_dir, _) in files_with_time.into_iter().take(6) {
                let display = if is_dir {
                    format!("+ {}/", name)
                } else {
                    format!("+ {}", name)
                };

                let replacement = if is_dir {
                    format!("@{}/", full_path)
                } else {
                    format!("@{}", full_path)
                };

                matches.push(Pair {
                    display,
                    replacement,
                });
            }

            (at_pos, matches)
        } else {
            (0, Vec::new())
        }
    }

    fn complete_model(&self, line: &str) -> Vec<Pair> {
        let mut matches = Vec::new();
        let lower_line = line.to_lowercase();

        if lower_line.starts_with("/model ") || lower_line == "/model" {
            let partial = if lower_line.len() > 7 {
                &line[7..]
            } else {
                ""
            };

            for model_name in &self.model_names {
                if model_name.to_lowercase().starts_with(&partial.to_lowercase()) {
                    matches.push(Pair {
                        display: format!("● {}", model_name),
                        replacement: format!("/model {}", model_name),
                    });
                }
            }
        }

        matches
    }
}

impl Completer for InputHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        // Check for @ file completion
        if line.contains('@') {
            let (start, matches) = self.complete_file(line, pos);
            if !matches.is_empty() {
                return Ok((start, matches));
            }
        }

        // Check for /model completion
        if line.to_lowercase().starts_with("/model") {
            let matches = self.complete_model(line);
            if !matches.is_empty() {
                return Ok((0, matches));
            }
        }

        // Check for / command completion
        if line.starts_with('/') {
            let matches = self.complete_command(line);
            return Ok((0, matches));
        }

        Ok((0, Vec::new()))
    }
}

impl Hinter for InputHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        if pos < line.len() {
            return None;
        }

        // Hint for commands
        if line.starts_with('/') && !line.contains(' ') {
            let input = line.to_lowercase();
            for (cmd, desc) in COMMANDS {
                if cmd.starts_with(&input) && *cmd != input {
                    let hint = &cmd[line.len()..];
                    return Some(format!("{} \x1b[38;5;245m({})\x1b[0m", hint, desc));
                }
            }
        }

        None
    }
}

impl Highlighter for InputHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, _default: bool) -> Cow<'b, str> {
        // Add cyan color to the prompt
        Cow::Owned(format!("\x1b[38;5;117m{}\x1b[0m", prompt))
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let mut result = String::new();
        let mut chars = line.chars().peekable();
        let mut in_at_path = false;

        while let Some(c) = chars.next() {
            match c {
                '/' if result.is_empty() => {
                    // Command highlighting
                    result.push_str("\x1b[38;5;220m/");
                    while let Some(&next) = chars.peek() {
                        if next.is_whitespace() {
                            result.push_str("\x1b[0m");
                            break;
                        }
                        result.push(chars.next().unwrap());
                    }
                    if !result.ends_with("\x1b[0m") {
                        result.push_str("\x1b[0m");
                    }
                }
                '@' => {
                    // File path highlighting
                    in_at_path = true;
                    result.push_str("\x1b[38;5;39m@");
                }
                ' ' | '\t' if in_at_path => {
                    result.push_str("\x1b[0m");
                    result.push(c);
                    in_at_path = false;
                }
                _ => {
                    result.push(c);
                }
            }
        }

        if in_at_path {
            result.push_str("\x1b[0m");
        }

        Cow::Owned(result)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[38;5;245m{}\x1b[0m", hint))
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        true
    }
}

impl Validator for InputHelper {}

pub struct InputReader {
    editor: Editor<InputHelper, rustyline::history::DefaultHistory>,
}

impl InputReader {
    pub fn new(model_names: Vec<String>) -> Self {
        let helper = InputHelper::new(model_names);
        let config = rustyline::Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .edit_mode(rustyline::EditMode::Emacs)
            .auto_add_history(true)
            .build();

        let mut editor = Editor::with_config(config).expect("Failed to create editor");
        editor.set_helper(Some(helper));

        Self { editor }
    }

    pub fn update_models(&mut self, model_names: Vec<String>) {
        if let Some(helper) = self.editor.helper_mut() {
            helper.update_models(model_names);
        }
    }

    pub fn readline(&mut self, prompt: &str) -> Result<String, ReadlineError> {
        self.editor.readline(prompt)
    }

    pub fn add_history_entry(&mut self, line: &str) {
        let _ = self.editor.add_history_entry(line);
    }
}

/// Parse file references from input (e.g., @path/to/file.txt)
pub fn parse_file_references(input: &str) -> Vec<String> {
    let mut files = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '@' {
            let mut path = String::new();
            while let Some(&next) = chars.peek() {
                if next.is_whitespace() {
                    break;
                }
                path.push(chars.next().unwrap());
            }
            if !path.is_empty() {
                files.push(path);
            }
        }
    }

    files
}

/// Remove file references from input and return clean text
pub fn strip_file_references(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '@' {
            // Skip until whitespace
            while let Some(&next) = chars.peek() {
                if next.is_whitespace() {
                    break;
                }
                chars.next();
            }
        } else {
            result.push(c);
        }
    }

    result.trim().to_string()
}

/// Read file contents for context
pub fn read_file_context(files: &[String]) -> String {
    let mut context = String::new();

    for file_path in files {
        match std::fs::read_to_string(file_path) {
            Ok(content) => {
                context.push_str(&format!("\n--- File: {} ---\n", file_path));
                context.push_str(&content);
                context.push_str("\n--- End of file ---\n");
            }
            Err(e) => {
                context.push_str(&format!("\n[Error reading {}: {}]\n", file_path, e));
            }
        }
    }

    context
}
