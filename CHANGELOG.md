# Changelog

All notable changes to AICLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-02-05

### Added

- Initial release
- Multi-model support (Claude, GPT, DeepSeek)
- Azure AI Foundry integration
- Tool execution (shell commands, file operations)
- File context with `@filename` syntax
- Streaming responses with real-time output
- TAB completion for commands and file paths
- Interactive model selection with arrow keys
- Context tracking with visual progress bar
- Auto-compact when context reaches 85%
- Internationalization (English/Portuguese)
- Responsive, centered banner UI
- Configuration via TOML file
- Environment variable support

### Commands

- `/help` - Show available commands
- `/exit`, `/quit` - Exit the CLI
- `/clear` - Clear conversation history
- `/model` - List and switch models
- `/add-model` - Add new model configuration
- `/config` - Show current configuration
- `/history` - Show conversation history
- `/lang` - Change language

### Tools

- `execute_command` - Run shell commands
- `read_file` - Read file contents
- `write_file` - Create/overwrite files
- `edit_file` - Modify existing files
- `list_directory` - List directory contents
- `search_files` - Find files by pattern
- `search_content` - Search text in files
