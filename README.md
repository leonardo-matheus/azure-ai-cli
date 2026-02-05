<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Azure-0078D4?style=for-the-badge&logo=microsoft-azure&logoColor=white" alt="Azure">
  <img src="https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white" alt="Windows">
  <img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="License">
</p>

<h1 align="center">
  <br>
  🐱 AICLI
  <br>
</h1>

<h4 align="center">A powerful CLI for Azure AI Foundry models inspired by <a href="https://github.com/anthropics/claude-code">Claude Code</a></h4>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#usage">Usage</a> •
  <a href="#commands">Commands</a> •
  <a href="#configuration">Configuration</a> •
  <a href="#building">Building</a>
</p>

---

```
╭─ AICLI ─────────────────────────────────────────────────────────╮
│                                                                  │
│                         Welcome back!                            │
│                                                                  │
│                            /\_/\                                 │
│                           ( o.o )                                │
│                            > ^ <                                 │
│                                                                  │
│                         gpt-4-turbo                              │
│                             GPT                                  │
│                        0K/128K tokens                            │
│                    ~/projects/my-app                             │
│                                                                  │
│                     By Leonardo M. Silva                         │
╰──────────────────────────────────────────────────────────────────╯
```

## ✨ Features

- **🤖 Multi-Model Support** - Claude, GPT-4, DeepSeek R1 and more via Azure AI Foundry
- **⚡ Tool Execution** - Automatically executes shell commands, reads/writes files
- **📁 File Context** - Reference files with `@filename` syntax
- **🔄 Streaming Responses** - Real-time token streaming output
- **📊 Context Tracking** - Visual progress bar showing token usage
- **🗜️ Auto-Compact** - Automatically summarizes conversation when context is full
- **⌨️ Tab Completion** - Smart completion for commands (`/`) and files (`@`)
- **🌍 Multi-language** - English and Portuguese support
- **🎨 Beautiful UI** - Responsive, centered banner with colors

## 📦 Installation

### Pre-built Binary (Windows)

Download the latest release from the [Releases](https://github.com/leonardo-matheus/aicli/releases) page.

### From Source

```bash
# Clone the repository
git clone https://github.com/leonardo-matheus/aicli.git
cd aicli

# Build release binary
cargo build --release

# Binary will be at target/release/aicli.exe
```

## 🚀 Usage

### First Run

On first run, AICLI will guide you through configuration:

```bash
aicli
```

Or configure manually:

```bash
aicli --config
```

### Environment Variables

You can also use environment variables:

```bash
export AZURE_API_KEY="your-api-key"
export AZURE_ENDPOINT="https://your-resource.services.ai.azure.com"
export AZURE_DEPLOYMENT="gpt-4-turbo"

aicli
```

### Basic Usage

```bash
# Start interactive chat
aicli

# Show help
aicli --help

# Show version
aicli --version
```

### File Context

Reference files in your prompts using `@`:

```
> explain @src/main.rs

> what's the difference between @old.txt and @new.txt?

> refactor @utils.js to use async/await
```

## 📋 Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `/help` | `/h`, `/?` | Show available commands |
| `/exit` | `/quit`, `/q` | Exit the CLI |
| `/clear` | `/c` | Clear conversation history |
| `/model` | | Interactive model selection |
| `/model <name>` | | Switch to specific model |
| `/add-model` | | Add a new model configuration |
| `/config` | | Show current configuration |
| `/history` | | Show conversation history |
| `/lang` | | Change language (en/pt) |

## ⚙️ Configuration

Configuration is stored at `~/.aicli/config.toml`:

```toml
active_model = "gpt-4-turbo"
language = "en"

[models.gpt-4-turbo]
name = "gpt-4-turbo"
api_key = "your-api-key"
endpoint = "https://your-resource.services.ai.azure.com"
deployment = "gpt-4-turbo"
model_type = "gpt"
max_tokens = 4096
temperature = 0.7

[models.claude-opus]
name = "claude-opus"
api_key = "your-api-key"
endpoint = "https://your-resource.services.ai.azure.com"
deployment = "claude-3-opus"
model_type = "claude"
max_tokens = 4096
temperature = 0.7
```

### Model Types

| Type | Models | Context Window |
|------|--------|----------------|
| `claude` | Claude 3 Opus, Sonnet, Haiku | 200K tokens |
| `gpt` | GPT-4, GPT-4 Turbo, GPT-4o | 128K tokens |
| `deepseek` | DeepSeek R1, DeepSeek Coder | 64K tokens |
| `other` | Other models | 32K tokens |

## 🛠️ Available Tools

AICLI can automatically execute these tools:

| Tool | Description |
|------|-------------|
| `execute_command` | Run shell commands |
| `read_file` | Read file contents |
| `write_file` | Create or overwrite files |
| `edit_file` | Modify existing files |
| `list_directory` | List directory contents |
| `search_files` | Find files by pattern (glob) |
| `search_content` | Search text in files (grep) |

## 🔧 Building

### Requirements

- Rust 1.70+
- Cargo

### Build Commands

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run directly
cargo run
```

### Project Structure

```
aicli/
├── Cargo.toml          # Dependencies and metadata
├── src/
│   ├── main.rs         # Entry point and CLI args
│   ├── chat.rs         # Main conversation loop
│   ├── client.rs       # Azure AI API client
│   ├── config.rs       # Configuration management
│   ├── tools.rs        # Tool implementations
│   ├── ui.rs           # Terminal UI and banner
│   ├── input.rs        # Input handling and completion
│   └── i18n.rs         # Internationalization
└── README.md
```

## 🎯 Context Management

AICLI tracks token usage and automatically manages context:

- **Visual Progress** - See current token usage in the banner
- **Color Coding** - Green (<50%), Yellow (50-80%), Red (>80%)
- **Auto-Compact** - At 85% capacity, older messages are summarized
- **Per-Response Status** - Token count shown after each response

```
  [15K/128K tokens]
```

## 🌐 Internationalization

Switch languages with `/lang`:

```
> /lang pt
✓ Language changed to Português

> /lang en
✓ Language changed to English
```

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 👨‍💻 Author

**Leonardo M. Silva**

- GitHub: [@leonardo-matheus](https://github.com/leonardo-matheus)

---

<p align="center">
  Made with ❤️ and 🦀 Rust
</p>
