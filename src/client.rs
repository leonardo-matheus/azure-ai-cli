use crate::config::{ModelConfig, ModelType};
use crate::tools::{ToolCall, ToolResult};
use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: MessageContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

impl MessageContent {
    pub fn as_text(&self) -> String {
        match self {
            MessageContent::Text(s) => s.clone(),
            MessageContent::Parts(parts) => {
                parts.iter()
                    .filter_map(|p| {
                        if let ContentPart::Text { text } = p {
                            Some(text.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

pub struct AzureClient {
    client: Client,
    config: ModelConfig,
}

impl AzureClient {
    pub fn new(config: ModelConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn update_config(&mut self, config: ModelConfig) {
        self.config = config;
    }

    pub fn get_model_name(&self) -> &str {
        &self.config.name
    }

    pub fn get_model_type(&self) -> &ModelType {
        &self.config.model_type
    }

    pub fn get_tools_schema() -> Vec<Value> {
        vec![
            json!({
                "type": "function",
                "function": {
                    "name": "execute_command",
                    "description": "Execute a shell command on the system. Use this to run any command-line operations.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "The command to execute"
                            },
                            "working_dir": {
                                "type": "string",
                                "description": "Working directory for the command (optional)"
                            }
                        },
                        "required": ["command"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "read_file",
                    "description": "Read the contents of a file",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the file to read"
                            }
                        },
                        "required": ["path"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "write_file",
                    "description": "Write content to a file, creating it if it doesn't exist",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the file to write"
                            },
                            "content": {
                                "type": "string",
                                "description": "Content to write to the file"
                            }
                        },
                        "required": ["path", "content"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "edit_file",
                    "description": "Edit a file by replacing specific text",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the file to edit"
                            },
                            "old_text": {
                                "type": "string",
                                "description": "Text to find and replace"
                            },
                            "new_text": {
                                "type": "string",
                                "description": "Text to replace with"
                            }
                        },
                        "required": ["path", "old_text", "new_text"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "list_directory",
                    "description": "List files and directories in a path",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the directory to list"
                            }
                        },
                        "required": ["path"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "search_files",
                    "description": "Search for files matching a pattern",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "pattern": {
                                "type": "string",
                                "description": "Glob pattern to match (e.g., '*.rs', '**/*.txt')"
                            },
                            "path": {
                                "type": "string",
                                "description": "Starting directory for search"
                            }
                        },
                        "required": ["pattern"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "search_content",
                    "description": "Search for text content in files",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Text or regex pattern to search for"
                            },
                            "path": {
                                "type": "string",
                                "description": "Directory to search in"
                            },
                            "file_pattern": {
                                "type": "string",
                                "description": "File pattern to filter (e.g., '*.rs')"
                            }
                        },
                        "required": ["query"]
                    }
                }
            }),
        ]
    }

    fn get_system_prompt() -> String {
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| ".".to_string());

        format!(
            r#"You are an AI assistant with direct access to the user's computer through tools.
You can execute commands, read/write files, and perform system operations.

Current working directory: {}
Operating System: {}

IMPORTANT RULES:
1. Execute tasks IMMEDIATELY without asking for confirmation
2. Use tools proactively to accomplish tasks
3. When writing code, write complete, working solutions
4. If a task requires multiple steps, execute them all
5. Report results clearly and concisely
6. If an error occurs, try to fix it automatically

Available tools:
- execute_command: Run shell commands
- read_file: Read file contents
- write_file: Create/overwrite files
- edit_file: Modify existing files
- list_directory: List directory contents
- search_files: Find files by pattern
- search_content: Search text in files

Be efficient, precise, and helpful."#,
            cwd,
            std::env::consts::OS
        )
    }

    pub async fn chat(
        &self,
        messages: &[Message],
        mut on_token: impl FnMut(&str),
    ) -> Result<(String, Vec<ToolCall>, TokenUsage)> {
        let system_prompt = Self::get_system_prompt();
        let tools = Self::get_tools_schema();

        match self.config.model_type {
            ModelType::Claude => self.chat_claude(messages, &system_prompt, &tools, on_token).await,
            ModelType::Gpt | ModelType::DeepSeek | ModelType::Other => {
                self.chat_openai(messages, &system_prompt, &tools, on_token).await
            }
        }
    }

    pub fn get_max_context(&self) -> usize {
        // Return context window size based on model type
        match self.config.model_type {
            ModelType::Claude => 200000,  // Claude 3 Opus: 200K
            ModelType::Gpt => 128000,     // GPT-4 Turbo: 128K
            ModelType::DeepSeek => 64000, // DeepSeek: 64K
            ModelType::Other => 32000,    // Default: 32K
        }
    }

    async fn chat_openai(
        &self,
        messages: &[Message],
        system_prompt: &str,
        tools: &[Value],
        mut on_token: impl FnMut(&str),
    ) -> Result<(String, Vec<ToolCall>, TokenUsage)> {
        let mut api_messages: Vec<Value> = vec![json!({
            "role": "system",
            "content": system_prompt
        })];

        // Estimate prompt tokens (rough: 1 token ≈ 4 chars)
        let mut prompt_chars = system_prompt.len();
        for msg in messages {
            prompt_chars += msg.content.as_text().len();
            api_messages.push(json!({
                "role": msg.role,
                "content": msg.content.as_text()
            }));
        }

        // Support both Azure OpenAI and Azure AI Foundry formats
        let endpoint = if self.config.endpoint.contains("/models") || self.config.endpoint.contains("services.ai.azure.com") {
            // Azure AI Foundry format
            format!(
                "{}/models/chat/completions?api-version=2024-05-01-preview",
                self.config.endpoint.trim_end_matches('/')
            )
        } else {
            // Classic Azure OpenAI format
            format!(
                "{}/openai/deployments/{}/chat/completions?api-version=2024-02-15-preview",
                self.config.endpoint.trim_end_matches('/'),
                self.config.deployment
            )
        };

        let body = json!({
            "model": self.config.deployment,
            "messages": api_messages,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "tools": tools,
            "stream": true
        });

        let response = self.client
            .post(&endpoint)
            .header("api-key", &self.config.api_key)
            .header("Authorization", format!("Bearer {}", &self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("API error: {}", error_text));
        }

        let mut full_content = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut current_tool_call: Option<(String, String, String)> = None;

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);

            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        continue;
                    }

                    if let Ok(json) = serde_json::from_str::<Value>(data) {
                        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                            for choice in choices {
                                if let Some(delta) = choice.get("delta") {
                                    // Handle content
                                    if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                        full_content.push_str(content);
                                        on_token(content);
                                    }

                                    // Handle tool calls
                                    if let Some(tcs) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                                        for tc in tcs {
                                            if let Some(func) = tc.get("function") {
                                                if let Some(name) = func.get("name").and_then(|n| n.as_str()) {
                                                    let id = tc.get("id")
                                                        .and_then(|i| i.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    current_tool_call = Some((id, name.to_string(), String::new()));
                                                }
                                                if let Some(args) = func.get("arguments").and_then(|a| a.as_str()) {
                                                    if let Some((_, _, ref mut existing_args)) = current_tool_call.as_mut() {
                                                        existing_args.push_str(args);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Check if we should finalize tool call
                                if let Some(finish_reason) = choice.get("finish_reason").and_then(|f| f.as_str()) {
                                    if finish_reason == "tool_calls" || finish_reason == "stop" {
                                        if let Some((id, name, args)) = current_tool_call.take() {
                                            if !name.is_empty() {
                                                let input: Value = serde_json::from_str(&args).unwrap_or(json!({}));
                                                tool_calls.push(ToolCall { id, name, input });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Finalize any remaining tool call
        if let Some((id, name, args)) = current_tool_call {
            if !name.is_empty() {
                let input: Value = serde_json::from_str(&args).unwrap_or(json!({}));
                tool_calls.push(ToolCall { id, name, input });
            }
        }

        // Estimate token usage (1 token ≈ 4 characters)
        let prompt_tokens = prompt_chars / 4;
        let completion_tokens = full_content.len() / 4;
        let usage = TokenUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        };

        Ok((full_content, tool_calls, usage))
    }

    async fn chat_claude(
        &self,
        messages: &[Message],
        system_prompt: &str,
        tools: &[Value],
        mut on_token: impl FnMut(&str),
    ) -> Result<(String, Vec<ToolCall>, TokenUsage)> {
        let mut api_messages: Vec<Value> = Vec::new();

        // Estimate prompt tokens (rough: 1 token ≈ 4 chars)
        let mut prompt_chars = system_prompt.len();
        for msg in messages {
            prompt_chars += msg.content.as_text().len();
            api_messages.push(json!({
                "role": msg.role,
                "content": msg.content.as_text()
            }));
        }

        // Convert tools to Claude format
        let claude_tools: Vec<Value> = tools.iter().map(|t| {
            let func = t.get("function").unwrap();
            json!({
                "name": func.get("name"),
                "description": func.get("description"),
                "input_schema": func.get("parameters")
            })
        }).collect();

        // Support both direct Anthropic API and Azure AI Foundry
        let endpoint = if self.config.endpoint.contains("services.ai.azure.com") {
            // Azure AI Foundry format
            format!(
                "{}/anthropic/v1/messages",
                self.config.endpoint.trim_end_matches('/')
            )
        } else {
            // Direct Anthropic API
            format!(
                "{}/v1/messages",
                self.config.endpoint.trim_end_matches('/')
            )
        };

        let body = json!({
            "model": self.config.deployment,
            "max_tokens": self.config.max_tokens,
            "system": system_prompt,
            "messages": api_messages,
            "tools": claude_tools,
            "stream": true
        });

        let response = self.client
            .post(&endpoint)
            .header("api-key", &self.config.api_key)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("API error: {}", error_text));
        }

        let mut full_content = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut current_tool: Option<(String, String, String)> = None;

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);

            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];

                    if let Ok(json) = serde_json::from_str::<Value>(data) {
                        let event_type = json.get("type").and_then(|t| t.as_str()).unwrap_or("");

                        match event_type {
                            "content_block_start" => {
                                if let Some(content_block) = json.get("content_block") {
                                    if content_block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                                        let id = content_block.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string();
                                        let name = content_block.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                                        current_tool = Some((id, name, String::new()));
                                    }
                                }
                            }
                            "content_block_delta" => {
                                if let Some(delta) = json.get("delta") {
                                    if let Some(text_delta) = delta.get("text").and_then(|t| t.as_str()) {
                                        full_content.push_str(text_delta);
                                        on_token(text_delta);
                                    }
                                    if let Some(partial_json) = delta.get("partial_json").and_then(|p| p.as_str()) {
                                        if let Some((_, _, ref mut args)) = current_tool.as_mut() {
                                            args.push_str(partial_json);
                                        }
                                    }
                                }
                            }
                            "content_block_stop" => {
                                if let Some((id, name, args)) = current_tool.take() {
                                    if !name.is_empty() {
                                        let input: Value = serde_json::from_str(&args).unwrap_or(json!({}));
                                        tool_calls.push(ToolCall { id, name, input });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Estimate token usage (1 token ≈ 4 characters)
        let prompt_tokens = prompt_chars / 4;
        let completion_tokens = full_content.len() / 4;
        let usage = TokenUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        };

        Ok((full_content, tool_calls, usage))
    }

    #[allow(dead_code)]
    pub async fn chat_with_tool_results(
        &self,
        messages: &[Message],
        tool_results: &[ToolResult],
        on_token: impl FnMut(&str),
    ) -> Result<(String, Vec<ToolCall>, TokenUsage)> {
        let mut all_messages = messages.to_vec();

        // Add tool results as assistant context
        let results_text = tool_results
            .iter()
            .map(|r| format!("[Tool: {}]\n{}", r.tool_name, r.output))
            .collect::<Vec<_>>()
            .join("\n\n");

        all_messages.push(Message {
            role: "assistant".to_string(),
            content: MessageContent::Text(format!("Tool results:\n{}", results_text)),
        });

        all_messages.push(Message {
            role: "user".to_string(),
            content: MessageContent::Text("Continue based on the tool results above.".to_string()),
        });

        self.chat(&all_messages, on_token).await
    }
}
