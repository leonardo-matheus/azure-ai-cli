# Release v1.0.0

## AICLI - Azure AI Command Line Interface

A powerful CLI for interacting with Azure AI Foundry models directly from your terminal.

### Highlights

- **Multi-Model Support** — Claude, GPT, DeepSeek via Azure AI Foundry
- **Tool Execution** — Run commands, read/write files, search codebase
- **Syntax Highlighting** — Dracula-themed code blocks
- **Easy Installation** — One command global install with `/install`

### Features

- Real-time streaming responses with animated indicators
- File context inclusion with `@filename` syntax
- Interactive model selection with `/model`
- Context tracking with auto-compact at 85%
- Tab completion for commands and files
- English and Portuguese language support
- Beautiful terminal UI

### Commands

| Command | Description |
|---------|-------------|
| `/help` | Show available commands |
| `/model` | Switch between models |
| `/clear` | Clear conversation |
| `/config` | Show configuration |
| `/lang` | Change language |
| `/install` | Install globally |
| `/exit` | Exit AICLI |

### Installation

```bash
# Download and run
./aicli

# Install globally
❯ /install

# Restart terminal and use from anywhere
aicli
```

### System Requirements

- Windows 10/11, Linux, or macOS
- Azure AI Foundry account with deployed model

### Configuration

Config file: `~/.aicli/config.toml`

```toml
active_model = "claude-opus"
language = "en"

[models.claude-opus]
name = "Claude Opus 4.5"
api_key = "your-key"
endpoint = "https://your-resource.services.ai.azure.com"
deployment = "claude-opus-4-5"
model_type = "claude"
max_tokens = 8192
temperature = 0.7
```

### Links

- [Documentation](https://leonardo-matheus.github.io/azure-ai-cli/)
- [GitHub Repository](https://github.com/leonardo-matheus/azure-ai-cli)

---

Made with ❤️ by Leonardo M. Silva
