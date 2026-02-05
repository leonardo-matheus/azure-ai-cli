use crossterm::{
    cursor,
    execute,
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{self, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write};
use std::path::Path;
use crate::i18n::{Language, Strings};

const GITHUB_URL: &str = "https://github.com/leonardo-matheus";
const VERSION: &str = "1.0.0";

pub struct UI {
    pub strings: Strings,
    term_width: usize,
    pub context_used: usize,
    pub context_max: usize,
}

impl UI {
    pub fn new(lang: Language) -> Self {
        let term_width = terminal::size().map(|(w, _)| w as usize).unwrap_or(120);
        Self {
            strings: Strings::new(lang),
            term_width: term_width.max(70),
            context_used: 0,
            context_max: 128000, // Default 128k context
        }
    }

    pub fn set_context_max(&mut self, max: usize) {
        self.context_max = max;
    }

    pub fn update_context(&mut self, used: usize) {
        self.context_used = used;
    }

    pub fn get_context_percent(&self) -> usize {
        if self.context_max == 0 { return 0; }
        (self.context_used * 100) / self.context_max
    }

    pub fn set_language(&mut self, lang: Language) {
        self.strings = Strings::new(lang);
    }

    fn hyperlink(text: &str, url: &str) -> String {
        format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text)
    }

    pub fn print_banner(&self, model: &str, model_type: &str, current_dir: &str) {
        // Responsive width: min 46, max based on terminal
        let w = self.term_width.min(80).max(46);
        let inner = w - 2;

        // Helper for centering text
        let center = |text: &str, width: usize| -> String {
            let text_len = text.chars().count();
            if text_len >= width {
                return text.to_string();
            }
            let padding = width - text_len;
            let left = padding / 2;
            let right = padding - left;
            format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
        };

        // Truncate path for display
        let max_path_len = inner - 4;
        let display_dir = Self::truncate_path(current_dir, max_path_len);

        // Context info
        let ctx_percent = self.get_context_percent();
        let ctx_color = if ctx_percent > 80 { "203" } else if ctx_percent > 50 { "220" } else { "82" };
        let ctx_text = format!("{}K/{}K tokens", self.context_used / 1000, self.context_max / 1000);

        let line = "─".repeat(inner);
        let empty = " ".repeat(inner);

        // Title line with centered "AICLI"
        let title = "─ AICLI ";
        let title_len = title.len();
        let remaining = inner - title_len;
        let title_line = format!("{}{}", title, "─".repeat(remaining));

        println!();
        println!("\x1b[38;5;75m╭{}╮\x1b[0m", title_line);
        println!("\x1b[38;5;75m│\x1b[0m{}\x1b[38;5;75m│\x1b[0m", empty);

        // Welcome message
        println!("\x1b[38;5;75m│\x1b[0m{}\x1b[38;5;75m│\x1b[0m",
            center("\x1b[1;37mWelcome back!\x1b[0m", inner + 9)); // +9 for ANSI codes

        println!("\x1b[38;5;75m│\x1b[0m{}\x1b[38;5;75m│\x1b[0m", empty);

        // Cat ASCII art - centered
        println!("\x1b[38;5;75m│\x1b[0m{}\x1b[38;5;75m│\x1b[0m",
            center("\x1b[38;5;220m/\\_/\\\x1b[0m", inner + 11));
        println!("\x1b[38;5;75m│\x1b[0m{}\x1b[38;5;75m│\x1b[0m",
            center("\x1b[38;5;220m( o.o )\x1b[0m", inner + 11));
        println!("\x1b[38;5;75m│\x1b[0m{}\x1b[38;5;75m│\x1b[0m",
            center("\x1b[38;5;220m> ^ <\x1b[0m", inner + 11));

        println!("\x1b[38;5;75m│\x1b[0m{}\x1b[38;5;75m│\x1b[0m", empty);

        // Model name - centered and bold
        let model_display = format!("\x1b[1;38;5;220m{}\x1b[0m", model);
        println!("\x1b[38;5;75m│\x1b[0m{}\x1b[38;5;75m│\x1b[0m",
            center(&model_display, inner + 12));

        // Model type - centered
        let type_display = format!("\x1b[38;5;245m{}\x1b[0m", model_type);
        println!("\x1b[38;5;75m│\x1b[0m{}\x1b[38;5;75m│\x1b[0m",
            center(&type_display, inner + 11));

        // Context - centered with color
        let ctx_display = format!("\x1b[38;5;{}m{}\x1b[0m", ctx_color, ctx_text);
        println!("\x1b[38;5;75m│\x1b[0m{}\x1b[38;5;75m│\x1b[0m",
            center(&ctx_display, inner + 11));

        // Path - centered
        let path_display = format!("\x1b[38;5;245m{}\x1b[0m", display_dir);
        println!("\x1b[38;5;75m│\x1b[0m{}\x1b[38;5;75m│\x1b[0m",
            center(&path_display, inner + 11));

        println!("\x1b[38;5;75m│\x1b[0m{}\x1b[38;5;75m│\x1b[0m", empty);
        println!("\x1b[38;5;75m╰{}╯\x1b[0m", line);
        println!();
    }

    pub fn print_welcome_line(&self) {
        let author_link = Self::hyperlink("Leonardo M. Silva", GITHUB_URL);
        println!("  \x1b[38;5;245mBy {} · v{}\x1b[0m", author_link, VERSION);
        println!();
    }

    fn visible_len(s: &str) -> usize {
        // Remove ANSI escape codes to get visible length
        let mut len = 0;
        let mut in_escape = false;
        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if c == 'm' || c == '\\' {
                    in_escape = false;
                }
            } else {
                len += 1;
            }
        }
        len
    }

    fn truncate_path(path: &str, max_len: usize) -> String {
        if path.len() <= max_len {
            path.to_string()
        } else {
            format!("...{}", &path[path.len() - max_len + 3..])
        }
    }

    pub fn print_welcome_message(&self) {
        println!("  \x1b[1;37mWelcome to AICLI\x1b[0m");
        println!();
    }

    pub fn print_separator(&self) {
        println!("\x1b[38;5;240m{}\x1b[0m", "─".repeat(self.term_width));
    }

    pub fn print_input_hint(&self) {
        self.print_separator();
        println!("\x1b[38;5;39m❯\x1b[0m Try \x1b[38;5;245m\"explain this code\"\x1b[0m or \x1b[38;5;245m\"@file what does this do?\"\x1b[0m");
        self.print_separator();
        println!("  \x1b[38;5;245m? for help · /model to switch · /exit to quit\x1b[0m");
        println!();
    }

    pub fn print_model_switch(&self, model: &str, model_type: &str) {
        println!();
        println!("  \x1b[38;5;82m✓\x1b[0m Switched to \x1b[38;5;220m{}\x1b[0m \x1b[38;5;245m({})\x1b[0m", model, model_type);
        println!();
    }

    pub fn print_lang_switch(&self, lang: &str) {
        println!();
        println!("  \x1b[38;5;82m✓\x1b[0m Language changed to \x1b[38;5;220m{}\x1b[0m", lang);
        println!();
    }

    pub fn print_thinking(&self, frame: usize) {
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let s = spinner[frame % spinner.len()];
        print!("\r\x1b[K  \x1b[38;5;220m{}\x1b[0m \x1b[38;5;245m{}\x1b[0m", s, self.strings.thinking());
        io::stdout().flush().unwrap();
    }

    pub fn print_working(&self, frame: usize, task: &str) {
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let s = spinner[frame % spinner.len()];
        print!("\r\x1b[K  \x1b[38;5;220m{}\x1b[0m \x1b[38;5;245m{}\x1b[0m", s, task);
        io::stdout().flush().unwrap();
    }

    pub fn clear_line(&self) {
        print!("\r\x1b[K");
        io::stdout().flush().unwrap();
    }

    pub fn print_assistant_prefix(&self) {
        println!();
        io::stdout().flush().unwrap();
    }

    pub fn print_token(&self, token: &str) {
        print!("{}", token);
        io::stdout().flush().unwrap();
    }

    pub fn print_newline(&self) {
        println!();
    }

    pub fn print_tool_call(&self, tool_name: &str, input: &str) {
        println!();
        println!("  \x1b[38;5;220m⚡ {}\x1b[0m", tool_name);

        // Parse and display input nicely
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(input) {
            if let Some(obj) = json.as_object() {
                for (key, value) in obj.iter().take(3) {
                    let val_str = match value {
                        serde_json::Value::String(s) => {
                            if s.len() > 50 { format!("{}...", &s[..47]) } else { s.clone() }
                        },
                        _ => {
                            let s = value.to_string();
                            if s.len() > 50 { format!("{}...", &s[..47]) } else { s }
                        }
                    };
                    println!("  \x1b[38;5;240m│\x1b[0m \x1b[38;5;245m{}:\x1b[0m {}", key, val_str);
                }
            }
        }
    }

    pub fn print_tool_result(&self, tool_name: &str, output: &str, success: bool) {
        let status = if success { "\x1b[38;5;82m✓\x1b[0m" } else { "\x1b[38;5;203m✗\x1b[0m" };
        println!("  {} \x1b[38;5;245m{}\x1b[0m", status, tool_name);

        // Show condensed output
        let lines: Vec<&str> = output.lines().collect();
        let max_lines = 4;

        for line in lines.iter().take(max_lines) {
            let truncated = if line.len() > 70 {
                format!("{}...", &line[..67])
            } else {
                line.to_string()
            };
            println!("  \x1b[38;5;240m│\x1b[0m {}", truncated);
        }

        if lines.len() > max_lines {
            println!("  \x1b[38;5;240m╰\x1b[0m \x1b[38;5;245m+{} more lines\x1b[0m", lines.len() - max_lines);
        }
        println!();
    }

    pub fn print_error(&self, message: &str) {
        println!("  \x1b[38;5;203m✗\x1b[0m {}", message);
    }

    pub fn print_info(&self, message: &str) {
        println!("  \x1b[38;5;39mℹ\x1b[0m {}", message);
    }

    pub fn print_success(&self, message: &str) {
        println!("  \x1b[38;5;82m✓\x1b[0m {}", message);
    }

    pub fn print_file_context(&self, files: &[String]) {
        if files.is_empty() {
            return;
        }
        println!();
        for file in files {
            println!("  \x1b[38;5;39m+\x1b[0m \x1b[38;5;75m{}\x1b[0m", file);
        }
        println!();
    }

    pub fn print_models_list(&self, models: &[(String, String, bool)]) {
        let s = &self.strings;
        println!();
        println!("  \x1b[1;37m{}\x1b[0m", s.title_models());
        println!();

        for (i, (name, model_type, is_active)) in models.iter().enumerate() {
            let marker = if *is_active { "\x1b[38;5;82m●\x1b[0m" } else { "\x1b[38;5;240m○\x1b[0m" };
            let name_style = if *is_active { "\x1b[1;38;5;220m" } else { "" };
            println!("    {} \x1b[38;5;245m{}.\x1b[0m {}{}\x1b[0m \x1b[38;5;245m({})\x1b[0m",
                marker, i + 1, name_style, name, model_type);
        }

        println!();
        println!("  \x1b[38;5;245mUse /model <name> to switch\x1b[0m");
        println!();
    }

    /// Interactive model selection menu
    /// Returns: Some(index) for model selection, Some(models.len()) for "Add model", None for cancel
    pub fn select_model_interactive(&self, models: &[(String, String, bool)]) -> Option<usize> {
        let total_options = models.len() + 1; // +1 for "Add model"
        let mut selected: usize = models.iter().position(|(_, _, active)| *active).unwrap_or(0);

        // Enable raw mode for keyboard input
        if enable_raw_mode().is_err() {
            return None;
        }

        // Hide cursor during selection
        let _ = execute!(io::stdout(), cursor::Hide);

        println!();
        println!("  \x1b[1;37m{}\x1b[0m", self.strings.title_models());
        println!("  \x1b[38;5;245m↑↓ navigate · Enter select · Esc cancel\x1b[0m");
        println!();

        // Initial render
        self.render_model_menu(models, selected);

        let result = loop {
            if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                match code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if selected > 0 {
                            selected -= 1;
                        } else {
                            selected = total_options - 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if selected < total_options - 1 {
                            selected += 1;
                        } else {
                            selected = 0;
                        }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        break Some(selected);
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        break None;
                    }
                    _ => continue,
                }

                // Re-render menu (move up and redraw)
                print!("\x1b[{}A", total_options + 1);
                io::stdout().flush().unwrap();
                self.render_model_menu(models, selected);
            }
        };

        // Restore terminal
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), cursor::Show);

        // Move to end and add newline
        println!();

        result
    }

    fn render_model_menu(&self, models: &[(String, String, bool)], selected: usize) {
        for (i, (name, model_type, is_active)) in models.iter().enumerate() {
            let pointer = if i == selected { "\x1b[38;5;39m❯\x1b[0m" } else { " " };
            let marker = if *is_active { "\x1b[38;5;82m●\x1b[0m" } else { "\x1b[38;5;240m○\x1b[0m" };
            let name_style = if i == selected { "\x1b[1;38;5;220m" } else if *is_active { "\x1b[38;5;220m" } else { "" };
            println!("\x1b[2K  {} {} {}{}\x1b[0m \x1b[38;5;245m({})\x1b[0m",
                pointer, marker, name_style, name, model_type);
        }

        // "Add model" option
        let add_pointer = if selected == models.len() { "\x1b[38;5;39m❯\x1b[0m" } else { " " };
        let add_style = if selected == models.len() { "\x1b[1;38;5;82m" } else { "\x1b[38;5;82m" };
        println!("\x1b[2K  {} {}+ Add model\x1b[0m", add_pointer, add_style);
        println!();

        io::stdout().flush().unwrap();
    }

    pub fn print_language_menu(&self, current_lang: Language) {
        println!();
        println!("  \x1b[1;37mLanguage\x1b[0m");
        println!();

        let en_marker = if current_lang == Language::En { "\x1b[38;5;82m●\x1b[0m" } else { "\x1b[38;5;240m○\x1b[0m" };
        let pt_marker = if current_lang == Language::Pt { "\x1b[38;5;82m●\x1b[0m" } else { "\x1b[38;5;240m○\x1b[0m" };

        println!("    {} English", en_marker);
        println!("    {} Português", pt_marker);
        println!();
        println!("  \x1b[38;5;245m/lang en · /lang pt\x1b[0m");
        println!();
    }

    pub fn print_help(&self) {
        let s = &self.strings;
        println!();
        println!("  \x1b[1;37m{}\x1b[0m", s.title_commands());
        println!();
        println!("    \x1b[38;5;220m/help\x1b[0m          {}", s.cmd_help());
        println!("    \x1b[38;5;220m/exit\x1b[0m          {}", s.cmd_exit());
        println!("    \x1b[38;5;220m/clear\x1b[0m         {}", s.cmd_clear());
        println!("    \x1b[38;5;220m/model\x1b[0m         {}", s.cmd_model());
        println!("    \x1b[38;5;220m/config\x1b[0m        {}", s.cmd_config());
        println!("    \x1b[38;5;220m/lang\x1b[0m          {}", s.cmd_lang());
        println!();
        self.print_separator();
        println!();
        println!("    \x1b[1mFile Context\x1b[0m");
        println!("    \x1b[38;5;245mUse @filename to include files as context\x1b[0m");
        println!("    \x1b[38;5;245mExample: explain @src/main.rs\x1b[0m");
        println!();
    }

    pub fn print_config(&self, endpoint: &str, deployment: &str, model_type: &str,
                        max_tokens: u32, temperature: f32, api_key_preview: &str) {
        println!();
        println!("  \x1b[1;37mConfiguration\x1b[0m");
        println!();
        println!("    Endpoint:    {}", endpoint);
        println!("    Deployment:  {}", deployment);
        println!("    Type:        {}", model_type);
        println!("    Max Tokens:  {}", max_tokens);
        println!("    Temperature: {}", temperature);
        println!("    API Key:     {}***", api_key_preview);
        println!();
    }

    pub fn clear_screen(&self) {
        execute!(
            io::stdout(),
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        ).unwrap();
    }

    /// Get recent files in current directory for suggestions
    pub fn get_recent_files(dir: &str, limit: usize) -> Vec<String> {
        let path = Path::new(dir);
        let mut files: Vec<(String, std::time::SystemTime)> = Vec::new();

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        // Skip hidden files and common ignore patterns
                        if !name.starts_with('.') && name != "node_modules" && name != "target" {
                            let display = if metadata.is_dir() {
                                format!("{}/", name)
                            } else {
                                name
                            };
                            files.push((display, modified));
                        }
                    }
                }
            }
        }

        // Sort by modification time (most recent first)
        files.sort_by(|a, b| b.1.cmp(&a.1));

        files.into_iter().take(limit).map(|(name, _)| name).collect()
    }
}

impl Default for UI {
    fn default() -> Self {
        Self::new(Language::default())
    }
}
