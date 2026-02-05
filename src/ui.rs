use crossterm::{
    cursor,
    execute,
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{self, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write};
use std::path::Path;
use crate::i18n::{Language, Strings};
use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Style, Color};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

const GITHUB_URL: &str = "https://github.com/leonardo-matheus";
const VERSION: &str = "1.0.0";

// Dracula theme colors
const DRACULA_BG: &str = "236";      // #282a36
const DRACULA_FG: &str = "255";      // #f8f8f2
const DRACULA_CYAN: &str = "117";    // #8be9fd
const DRACULA_GREEN: &str = "84";    // #50fa7b
const DRACULA_ORANGE: &str = "215";  // #ffb86c
const DRACULA_PINK: &str = "205";    // #ff79c6
const DRACULA_PURPLE: &str = "141";  // #bd93f9
const DRACULA_RED: &str = "203";     // #ff5555
const DRACULA_YELLOW: &str = "228";  // #f1fa8c
const DRACULA_COMMENT: &str = "103"; // #6272a4

pub struct UI {
    pub strings: Strings,
    pub term_width: usize,
    pub context_used: usize,
    pub context_max: usize,
    pub current_model: String,
    pub current_model_type: String,
    pub current_path: String,
    in_code_block: std::cell::Cell<bool>,
    code_buffer: std::cell::RefCell<String>,
    code_lang: std::cell::RefCell<String>,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl UI {
    pub fn new(lang: Language) -> Self {
        let term_width = terminal::size().map(|(w, _)| w as usize).unwrap_or(120);
        Self {
            strings: Strings::new(lang),
            term_width: term_width.max(70),
            context_used: 0,
            context_max: 128000,
            current_model: String::new(),
            current_model_type: String::new(),
            current_path: String::new(),
            in_code_block: std::cell::Cell::new(false),
            code_buffer: std::cell::RefCell::new(String::new()),
            code_lang: std::cell::RefCell::new(String::new()),
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
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

    pub fn set_model_info(&mut self, model: &str, model_type: &str, path: &str) {
        self.current_model = model.to_string();
        self.current_model_type = model_type.to_string();
        self.current_path = path.to_string();
    }

    fn hyperlink(text: &str, url: &str) -> String {
        format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text)
    }

    /// Startup animation
    pub fn play_startup_animation(&self) {
        let frames = [
            ("▛▀▀▀▀▀▀▀▀▜", ""),
            ("▛▀▀▀▀▀▀▀▀▜", "▌        ▐"),
            ("▛▀▀▀▀▀▀▀▀▜", "▌ /\\     ▐"),
            ("▛▀▀▀▀▀▀▀▀▜", "▌ /\\_/\\  ▐"),
            ("▛▀▀▀▀▀▀▀▀▜", "▌( o.o ) ▐"),
            ("▛▀▀▀▀▀▀▀▀▜", "▌ > ^ <  ▐"),
        ];

        print!("\x1b[?25l"); // Hide cursor
        io::stdout().flush().unwrap();

        for (i, (top, mid)) in frames.iter().enumerate() {
            print!("\r\x1b[K");
            print!("\x1b[38;5;{}m{}\x1b[0m", DRACULA_PURPLE, top);
            if !mid.is_empty() {
                print!("\n\r\x1b[K\x1b[38;5;{}m{}\x1b[0m", DRACULA_CYAN, mid);
                print!("\x1b[1A"); // Move up
            }
            io::stdout().flush().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(80));
        }

        // Final frame with full logo
        print!("\r\x1b[K");
        println!();
        print!("\x1b[?25h"); // Show cursor
        io::stdout().flush().unwrap();
    }

    pub fn print_banner(&self, model: &str, model_type: &str, current_dir: &str) {
        let display_path = Self::truncate_path(current_dir, 40);

        println!();
        // Modern compact header like LOCAL-CLI
        println!("\x1b[38;5;75m▛▀▀▀▀▀▀▀▀▜\x1b[0m  \x1b[1;37mAICLI\x1b[0m \x1b[38;5;245mv{}\x1b[0m", VERSION);
        println!("\x1b[38;5;75m▌\x1b[0m \x1b[38;5;220m/\\_/\\\x1b[0m  \x1b[38;5;75m▐\x1b[0m  \x1b[38;5;82m●\x1b[0m \x1b[1;38;5;220m{}\x1b[0m \x1b[38;5;245m({})\x1b[0m", model, model_type);
        println!("\x1b[38;5;75m▙▄▄▄▄▄▄▄▄▟\x1b[0m  \x1b[38;5;245m{}\x1b[0m", display_path);
        println!();
    }

    pub fn print_welcome_line(&self) {
        let author_link = Self::hyperlink("Leonardo M. Silva", GITHUB_URL);
        println!(" \x1b[38;5;220m🎯\x1b[0m Switch models anytime! Use \x1b[38;5;75m/model\x1b[0m to select your preferred LLM.");
        println!("    \x1b[38;5;245mBy {} · Type \x1b[38;5;75m/help\x1b[0m\x1b[38;5;245m for commands\x1b[0m", author_link);
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
        // No need for extra hints - the status bar at bottom will show info
    }

    pub fn print_status_bar(&self) {
        let ctx_k = self.context_used / 1000;
        let ctx_percent = self.get_context_percent();
        let ctx_color = if ctx_percent > 80 { DRACULA_RED } else if ctx_percent > 50 { DRACULA_ORANGE } else { DRACULA_GREEN };

        let model_display = if self.current_model.len() > 25 {
            format!("{}...", &self.current_model[..22])
        } else {
            self.current_model.clone()
        };

        let path_display = Self::truncate_path(&self.current_path, 25);

        // Status line with better formatting
        println!();
        print!(" \x1b[38;5;{}m●\x1b[0m \x1b[38;5;{}m{}\x1b[0m",
            DRACULA_GREEN, DRACULA_YELLOW, model_display);
        print!(" \x1b[38;5;{}m│\x1b[0m ", DRACULA_COMMENT);
        print!("\x1b[38;5;{}mContext: {}k ({}%)\x1b[0m", ctx_color, ctx_k, ctx_percent);
        print!(" \x1b[38;5;{}m│\x1b[0m ", DRACULA_COMMENT);
        println!("\x1b[38;5;{}m{}\x1b[0m", DRACULA_COMMENT, path_display);
    }

    /// Draw the input box frame
    pub fn draw_input_box(&self) {
        let w = self.term_width.min(120);
        let border = "─".repeat(w - 2);

        println!();
        println!("\x1b[38;5;{}m┌{}┐\x1b[0m", DRACULA_COMMENT, border);
        print!("\x1b[38;5;{}m│\x1b[0m \x1b[38;5;{}m❯\x1b[0m ", DRACULA_COMMENT, DRACULA_CYAN);
    }

    /// Close the input box after reading input
    pub fn close_input_box(&self, _input: &str) {
        let w = self.term_width.min(120);
        let border = "─".repeat(w - 2);
        println!("\x1b[38;5;{}m└{}┘\x1b[0m", DRACULA_COMMENT, border);
    }

    /// Draw bottom status bar with shortcuts
    pub fn draw_shortcuts_bar(&self) {
        let ctx_k = self.context_used / 1000;
        let ctx_percent = self.get_context_percent();
        let ctx_color = if ctx_percent > 80 { DRACULA_RED } else if ctx_percent > 50 { DRACULA_ORANGE } else { DRACULA_GREEN };

        let model_short = if self.current_model.len() > 15 {
            format!("{}...", &self.current_model[..12])
        } else {
            self.current_model.clone()
        };

        print!(" \x1b[38;5;{}m[{}k/{}%]\x1b[0m", ctx_color, ctx_k, ctx_percent);
        print!(" \x1b[38;5;{}m│\x1b[0m", DRACULA_COMMENT);
        print!(" \x1b[38;5;{}m●\x1b[0m \x1b[38;5;{}m{}\x1b[0m", DRACULA_GREEN, DRACULA_YELLOW, model_short);
        print!(" \x1b[38;5;{}m│\x1b[0m", DRACULA_COMMENT);
        print!(" \x1b[38;5;{}m@\x1b[0mfiles", DRACULA_CYAN);
        print!(" \x1b[38;5;{}m/\x1b[0mcmds", DRACULA_PINK);
        print!(" \x1b[38;5;{}m│\x1b[0m", DRACULA_COMMENT);
        println!(" \x1b[38;5;{}m/help\x1b[0m", DRACULA_COMMENT);
    }

    pub fn print_model_switch(&self, model: &str, model_type: &str) {
        println!();
        println!("\x1b[38;5;82m●\x1b[0m Switched to \x1b[1;38;5;220m{}\x1b[0m \x1b[38;5;245m({})\x1b[0m", model, model_type);
        println!();
    }

    pub fn print_lang_switch(&self, lang: &str) {
        println!();
        println!("  \x1b[38;5;82m✓\x1b[0m Language changed to \x1b[38;5;220m{}\x1b[0m", lang);
        println!();
    }

    pub fn print_thinking(&self, frame: usize) {
        let spinners = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let dots = ["", ".", "..", "..."];
        let s = spinners[frame % spinners.len()];
        let d = dots[(frame / 3) % dots.len()];
        print!("\r\x1b[K\x1b[38;5;{}m{}\x1b[0m \x1b[38;5;{}m{}{}\x1b[0m",
            DRACULA_PURPLE, s, DRACULA_COMMENT, self.strings.thinking(), d);
        io::stdout().flush().unwrap();
    }

    pub fn print_working(&self, frame: usize, task: &str) {
        let spinners = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
        let s = spinners[frame % spinners.len()];
        print!("\r\x1b[K\x1b[38;5;{}m{}\x1b[0m \x1b[38;5;{}m{}\x1b[0m",
            DRACULA_ORANGE, s, DRACULA_COMMENT, task);
        io::stdout().flush().unwrap();
    }

    /// Animated typing effect for text
    pub fn print_typing(&self, text: &str, delay_ms: u64) {
        for c in text.chars() {
            print!("{}", c);
            io::stdout().flush().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
    }

    pub fn clear_line(&self) {
        print!("\r\x1b[K");
        io::stdout().flush().unwrap();
    }

    pub fn print_assistant_prefix(&self) {
        println!();
        println!("\x1b[38;5;75m●\x1b[0m \x1b[1;37mAssistant\x1b[0m");
        print!("  ");
        io::stdout().flush().unwrap();
    }

    pub fn print_token(&self, token: &str) {
        let mut buffer = self.code_buffer.borrow_mut();
        buffer.push_str(token);

        // Process the buffer looking for code block markers
        loop {
            if let Some(pos) = buffer.find("```") {
                // Print everything before the marker
                let before = &buffer[..pos];
                if !before.is_empty() {
                    if self.in_code_block.get() {
                        // Inside code block - we'll highlight when closing
                    } else {
                        // Regular text
                        print!("{}", before.replace("\n", "\n  "));
                    }
                }

                // Toggle code block state
                if self.in_code_block.get() {
                    // End of code block - highlight accumulated code
                    let code_content = before.to_string();
                    let lang = self.code_lang.borrow().clone();

                    // Print highlighted code
                    let highlighted = self.highlight_code(&code_content, &lang);
                    for (i, line) in highlighted.lines().enumerate() {
                        if i > 0 {
                            print!("\n");
                        }
                        print!("  \x1b[38;5;240m│\x1b[0m {}", line);
                    }

                    // Close the code block
                    let w = self.term_width.min(80);
                    print!("\n  \x1b[38;5;240m└{}\x1b[0m", "─".repeat(w - 4));
                    self.in_code_block.set(false);
                    self.code_lang.borrow_mut().clear();
                } else {
                    // Start of code block - find the language tag
                    let after_marker = &buffer[pos + 3..];
                    if let Some(newline_pos) = after_marker.find('\n') {
                        let lang = after_marker[..newline_pos].trim().to_string();
                        *self.code_lang.borrow_mut() = lang.clone();

                        let lang_display = if lang.is_empty() { "code".to_string() } else { lang };
                        let w = self.term_width.min(80);
                        print!("\n  \x1b[38;5;240m┌─ {} {}\x1b[0m\n",
                            lang_display,
                            "─".repeat(w.saturating_sub(8 + lang_display.len())));

                        self.in_code_block.set(true);
                        *buffer = after_marker[newline_pos + 1..].to_string();
                        continue;
                    } else {
                        // No newline yet, wait for more tokens
                        break;
                    }
                }

                *buffer = buffer[pos + 3..].to_string();
                // Remove any trailing newline after closing ```
                if buffer.starts_with('\n') {
                    *buffer = buffer[1..].to_string();
                }
            } else {
                break;
            }
        }

        // Print remaining buffer content if not in code block and no pending ```
        if !self.in_code_block.get() && !buffer.is_empty() && !buffer.contains("``") {
            let content = buffer.clone();
            buffer.clear();
            print!("{}", content.replace("\n", "\n  "));
        }

        io::stdout().flush().unwrap();
    }

    pub fn reset_code_state(&self) {
        self.in_code_block.set(false);
        self.code_buffer.borrow_mut().clear();
        self.code_lang.borrow_mut().clear();
    }

    /// Highlight code with Dracula-like theme colors
    fn highlight_code(&self, code: &str, lang: &str) -> String {
        // Map language aliases
        let syntax_name = match lang.to_lowercase().as_str() {
            "js" | "javascript" => "JavaScript",
            "ts" | "typescript" => "TypeScript",
            "rs" | "rust" => "Rust",
            "py" | "python" => "Python",
            "java" => "Java",
            "html" => "HTML",
            "css" => "CSS",
            "json" => "JSON",
            "xml" => "XML",
            "sql" => "SQL",
            "sh" | "bash" | "shell" => "Bourne Again Shell (bash)",
            "yml" | "yaml" => "YAML",
            "toml" => "TOML",
            "md" | "markdown" => "Markdown",
            "c" => "C",
            "cpp" | "c++" => "C++",
            "go" => "Go",
            "rb" | "ruby" => "Ruby",
            "php" => "PHP",
            "swift" => "Swift",
            "kt" | "kotlin" => "Kotlin",
            _ => lang,
        };

        let syntax = self.syntax_set
            .find_syntax_by_name(syntax_name)
            .or_else(|| self.syntax_set.find_syntax_by_extension(lang))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        // Use Monokai (closest to Dracula in defaults)
        let theme = &self.theme_set.themes["base16-monokai.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);

        let mut result = String::new();
        for line in LinesWithEndings::from(code) {
            match highlighter.highlight_line(line, &self.syntax_set) {
                Ok(ranges) => {
                    for (style, text) in ranges {
                        let colored = Self::style_to_ansi(&style, text);
                        result.push_str(&colored);
                    }
                }
                Err(_) => {
                    // Fallback: just use default code color
                    result.push_str(&format!("\x1b[38;5;222m{}\x1b[0m", line));
                }
            }
        }
        result
    }

    /// Convert syntect Style to ANSI escape codes (Dracula-inspired)
    fn style_to_ansi(style: &Style, text: &str) -> String {
        let fg = style.foreground;

        // Map to closest Dracula colors
        let color_code = match (fg.r, fg.g, fg.b) {
            // Pink/Magenta (keywords) - Dracula pink #ff79c6
            (r, g, b) if r > 200 && g < 150 && b > 150 => "205",
            // Purple (constants) - Dracula purple #bd93f9
            (r, g, b) if r > 150 && g < 180 && b > 200 => "141",
            // Green (strings) - Dracula green #50fa7b
            (r, g, b) if g > 200 && r < 150 => "84",
            // Yellow (classes/functions) - Dracula yellow #f1fa8c
            (r, g, b) if r > 200 && g > 200 && b < 150 => "228",
            // Cyan (support) - Dracula cyan #8be9fd
            (r, g, b) if g > 200 && b > 200 && r < 150 => "117",
            // Orange (numbers) - Dracula orange #ffb86c
            (r, g, b) if r > 200 && g > 150 && g < 200 && b < 150 => "215",
            // Red (errors/tags) - Dracula red #ff5555
            (r, _, _) if r > 220 => "203",
            // White/light gray (default text) - Dracula foreground #f8f8f2
            (r, g, b) if r > 200 && g > 200 && b > 200 => "255",
            // Gray (comments) - Dracula comment #6272a4
            (r, g, b) if r < 150 && g < 150 && b < 180 => "103",
            // Default: use actual RGB if terminal supports it
            _ => {
                return format!("\x1b[38;2;{};{};{}m{}\x1b[0m", fg.r, fg.g, fg.b, text);
            }
        };

        format!("\x1b[38;5;{}m{}\x1b[0m", color_code, text)
    }

    /// Format complete response with syntax highlighting for code blocks
    pub fn format_response(&self, content: &str) -> String {
        let mut result = String::new();
        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_buffer = String::new();

        for line in content.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    // End of code block - render it
                    result.push_str(&self.render_code_block(&code_lang, &code_buffer));
                    code_buffer.clear();
                    code_lang.clear();
                    in_code_block = false;
                } else {
                    // Start of code block
                    in_code_block = true;
                    code_lang = line.trim_start_matches('`').to_string();
                }
            } else if in_code_block {
                code_buffer.push_str(line);
                code_buffer.push('\n');
            } else {
                // Regular text - apply inline formatting
                result.push_str(&self.format_inline(line));
                result.push('\n');
            }
        }

        // Handle unclosed code block
        if in_code_block && !code_buffer.is_empty() {
            result.push_str(&self.render_code_block(&code_lang, &code_buffer));
        }

        result
    }

    fn render_code_block(&self, lang: &str, code: &str) -> String {
        let w = self.term_width.min(100);
        let border = "─".repeat(w - 6);

        let lang_display = if lang.is_empty() { "code" } else { lang };

        let mut result = String::new();
        result.push_str(&format!("\n  \x1b[38;5;240m┌─ {} {}\x1b[0m\n", lang_display, border.chars().take(w - 10 - lang_display.len()).collect::<String>()));

        for line in code.lines() {
            let truncated = if line.len() > w - 8 {
                format!("{}...", &line[..w - 11])
            } else {
                line.to_string()
            };
            result.push_str(&format!("  \x1b[38;5;240m│\x1b[0m \x1b[38;5;222m{}\x1b[0m\n", truncated));
        }

        result.push_str(&format!("  \x1b[38;5;240m└{}\x1b[0m\n", border));
        result
    }

    fn format_inline(&self, line: &str) -> String {
        let mut result = line.to_string();

        // Bold: **text** or __text__
        let bold_re = regex::Regex::new(r"\*\*(.+?)\*\*|__(.+?)__").unwrap();
        result = bold_re.replace_all(&result, "\x1b[1m$1$2\x1b[0m").to_string();

        // Inline code: `code`
        let code_re = regex::Regex::new(r"`([^`]+)`").unwrap();
        result = code_re.replace_all(&result, "\x1b[38;5;222m$1\x1b[0m").to_string();

        // Headers: ## Header
        if result.starts_with("# ") {
            result = format!("\x1b[1;38;5;75m{}\x1b[0m", &result[2..]);
        } else if result.starts_with("## ") {
            result = format!("\x1b[1;38;5;75m{}\x1b[0m", &result[3..]);
        } else if result.starts_with("### ") {
            result = format!("\x1b[1;38;5;245m{}\x1b[0m", &result[4..]);
        }

        // Bullet points
        if result.starts_with("- ") || result.starts_with("* ") {
            result = format!("\x1b[38;5;75m•\x1b[0m {}", &result[2..]);
        }

        // Numbered lists (keep as-is but add color)
        if result.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) && result.contains(". ") {
            if let Some(pos) = result.find(". ") {
                let num = &result[..pos + 1];
                let text = &result[pos + 2..];
                result = format!("\x1b[38;5;75m{}\x1b[0m {}", num, text);
            }
        }

        result
    }

    pub fn print_newline(&self) {
        println!();
    }

    pub fn print_context_status(&self) {
        self.print_status_bar();
    }

    pub fn print_tool_call(&self, tool_name: &str, input: &str) {
        println!();
        println!("  \x1b[38;5;220m⚡\x1b[0m \x1b[38;5;75m{}\x1b[0m", tool_name);

        // Parse and display input nicely
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(input) {
            if let Some(obj) = json.as_object() {
                for (key, value) in obj.iter().take(3) {
                    let val_str = match value {
                        serde_json::Value::String(s) => {
                            if s.len() > 60 { format!("{}...", &s[..57]) } else { s.clone() }
                        },
                        _ => {
                            let s = value.to_string();
                            if s.len() > 60 { format!("{}...", &s[..57]) } else { s }
                        }
                    };
                    println!("     \x1b[38;5;245m{}:\x1b[0m {}", key, val_str);
                }
            }
        }
    }

    pub fn print_tool_result(&self, tool_name: &str, output: &str, success: bool) {
        let status = if success { "\x1b[38;5;82m✓\x1b[0m" } else { "\x1b[38;5;203m✗\x1b[0m" };
        println!("  {} \x1b[38;5;245m{}\x1b[0m", status, tool_name);

        // Show condensed output
        let lines: Vec<&str> = output.lines().collect();
        let max_lines = 5;

        for line in lines.iter().take(max_lines) {
            let truncated = if line.len() > 80 {
                format!("{}...", &line[..77])
            } else {
                line.to_string()
            };
            println!("     \x1b[38;5;240m{}\x1b[0m", truncated);
        }

        if lines.len() > max_lines {
            println!("     \x1b[38;5;245m... +{} more lines\x1b[0m", lines.len() - max_lines);
        }
    }

    pub fn print_error(&self, message: &str) {
        println!("\x1b[38;5;203m✗\x1b[0m {}", message);
    }

    pub fn print_info(&self, message: &str) {
        println!("\x1b[38;5;75mℹ\x1b[0m {}", message);
    }

    pub fn print_success(&self, message: &str) {
        println!("\x1b[38;5;82m✓\x1b[0m {}", message);
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
