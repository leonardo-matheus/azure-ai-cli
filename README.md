<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Azure-0078D4?style=for-the-badge&logo=microsoft-azure&logoColor=white" alt="Azure">
  <img src="https://img.shields.io/badge/License-MIT-green.svg?style=for-the-badge" alt="License">
  <img src="https://img.shields.io/github/v/release/leonardo-matheus/azure-ai-cli?style=for-the-badge" alt="Release">
</p>

<h1 align="center">❯ AICLI</h1>

<h4 align="center">A powerful CLI for Azure AI Foundry models — Claude, GPT, DeepSeek and more.</h4>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#usage">Usage</a> •
  <a href="#configuration">Configuration</a> •
  <a href="https://leonardo-matheus.github.io/azure-ai-cli/">Documentation</a>
</p>

---

```
▛▀▀▀▀▀▀▀▀▜  AICLI v1.0.0
▌ /\_/\  ▐  ● Claude Opus 4.5 (Claude)
▙▄▄▄▄▄▄▄▄▟  ~/projects/my-app

❯ Create a REST API with Express and TypeScript

● AICLI
  Creating Express TypeScript project...
  ✓ Created src/index.ts
  ✓ Created package.json
  ✓ Installed dependencies
```

## Features

- **Multi-Model Support** — Switch between Claude, GPT, DeepSeek instantly with `/model`
- **Syntax Highlighting** — Beautiful Dracula-themed code blocks with language detection
- **Tool Execution** — Execute commands, read/write files, search codebase automatically
- **File Context** — Include files with `@filename` for context-aware responses
- **Streaming** — Real-time streaming with animated thinking indicator
- **Context Tracking** — Visual progress bar showing token usage with auto-compact
- **Tab Completion** — Smart completion for commands and file paths
- **Multilingual** — English and Portuguese interfaces
- **Easy Install** — One command global installation with `/install`

## Installation

### Quick Install

Download the latest release from [Releases](https://github.com/leonardo-matheus/azure-ai-cli/releases) and run:

```bash
./aicli
❯ /install
```

Restart your terminal, then use `aicli` from anywhere.

### Build from Source

```bash
git clone https://github.com/leonardo-matheus/azure-ai-cli.git
cd azure-ai-cli
cargo build --release
./target/release/aicli
```

## Usage

```bash
# Start AICLI
aicli

# Chat with AI
❯ Create a REST API with Express and TypeScript

# Include file context
❯ Explain this code @src/main.rs

# Switch models
❯ /model gpt-4

# Show help
❯ /help
```

## Commands

| Command | Description |
|---------|-------------|
| `/help` | Show available commands |
| `/model` | Interactive model selection |
| `/model <name>` | Switch to specific model |
| `/clear` | Clear conversation history |
| `/config` | Show current configuration |
| `/lang <en\|pt>` | Change language |
| `/install` | Install AICLI globally |
| `/uninstall` | Uninstall AICLI |
| `/exit` | Exit AICLI |
| `@file` | Include file as context |

## Configuration

Configuration is stored at `~/.aicli/config.toml`:

```toml
active_model = "claude-opus"
language = "en"

[models.claude-opus]
name = "Claude Opus 4.5"
api_key = "your-api-key"
endpoint = "https://your-resource.services.ai.azure.com"
deployment = "claude-opus-4-5"
model_type = "claude"
max_tokens = 8192
temperature = 0.7
```

### Model Types

| Type | Models | Context |
|------|--------|---------|
| `claude` | Claude 3 Opus, Sonnet, Haiku | 200K |
| `gpt` | GPT-4, GPT-4 Turbo, GPT-3.5 | 128K |
| `deepseek` | DeepSeek Coder, Chat | 64K |
| `other` | Any OpenAI-compatible | 32K |

## Tools

AICLI can automatically execute:

| Tool | Description |
|------|-------------|
| `execute_command` | Run shell commands |
| `read_file` | Read file contents |
| `write_file` | Create or overwrite files |
| `edit_file` | Modify existing files |
| `list_directory` | List directory contents |
| `search_files` | Find files by pattern |
| `search_content` | Search text in files |

## Documentation

Full documentation: [leonardo-matheus.github.io/azure-ai-cli](https://leonardo-matheus.github.io/azure-ai-cli/)

## License

MIT License — see [LICENSE](LICENSE) for details.

## Author

**Leonardo M. Silva** — [@leonardo-matheus](https://github.com/leonardo-matheus)

---

<p align="center">Built with ❤️ and Rust</p>
