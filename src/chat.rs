use crate::client::{AzureClient, Message, MessageContent};
use crate::config::{AppConfig, add_model_interactive, save_config};
use crate::i18n::Language;
use crate::input::{InputReader, parse_file_references, strip_file_references, read_file_context};
use crate::tools::{ToolCall, ToolExecutor, ToolResult};
use crate::ui::UI;
use anyhow::Result;
use rustyline::error::ReadlineError;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const COMPACT_THRESHOLD: f32 = 0.85; // Compact when context reaches 85%

/// Animated spinner that runs until stopped
fn start_thinking_animation(ui: &UI) -> Arc<AtomicBool> {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_clone = stop_flag.clone();

    let thinking_text = ui.strings.thinking().to_string();

    std::thread::spawn(move || {
        let mut frame = 0;
        let spinners = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let dots = ["", ".", "..", "..."];

        while !stop_clone.load(Ordering::Relaxed) {
            let s = spinners[frame % spinners.len()];
            let d = dots[(frame / 3) % dots.len()];
            print!("\r\x1b[K\x1b[38;5;141m{}\x1b[0m \x1b[38;5;103m{}{}\x1b[0m", s, thinking_text, d);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            std::thread::sleep(Duration::from_millis(80));
            frame += 1;
        }
    });

    stop_flag
}

pub async fn run(mut config: AppConfig) -> Result<()> {
    let mut ui = UI::new(config.language);

    let active_model = config.get_active_model()
        .ok_or_else(|| anyhow::anyhow!("No active model configured"))?
        .clone();

    let mut client = AzureClient::new(active_model.clone());

    // Set context max from client
    ui.set_context_max(client.get_max_context());

    let model_names: Vec<String> = config.models.keys().cloned().collect();
    let mut input_reader = InputReader::new(model_names);

    let current_dir = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    ui.set_model_info(&active_model.name, &active_model.model_type.to_string(), &current_dir);

    // Startup animation
    ui.play_startup_animation();

    ui.print_banner(&active_model.name, &active_model.model_type.to_string(), &current_dir);
    ui.print_welcome_line();

    let mut messages: Vec<Message> = Vec::new();
    let mut total_tokens: usize = 0;

    loop {
        // Draw input prompt
        ui.draw_input_box();

        let input = match input_reader.readline("") {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                println!();
                ui.print_info(&ui.strings.ctrl_c_hint().to_string());
                continue;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!();
                ui.print_error(&format!("Input error: {}", err));
                continue;
            }
        };

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        input_reader.add_history_entry(input);

        // Handle commands
        if input.starts_with('/') {
            match handle_command(input, &mut ui, &mut config, &mut client, &mut messages, &mut input_reader, &mut total_tokens) {
                CommandResult::Continue => continue,
                CommandResult::Exit => break,
                CommandResult::Processed => continue,
            }
        }

        // Parse file references
        let file_refs = parse_file_references(input);
        let clean_input = strip_file_references(input);

        let mut full_message = clean_input.clone();

        if !file_refs.is_empty() {
            ui.print_file_context(&file_refs);
            let context = read_file_context(&file_refs);
            full_message = format!("{}\n\nFile context:{}", clean_input, context);
        }

        messages.push(Message {
            role: "user".to_string(),
            content: MessageContent::Text(full_message),
        });

        // Check if we need to auto-compact before the API call
        let context_percent = (total_tokens as f32) / (ui.context_max as f32);
        if context_percent > COMPACT_THRESHOLD && messages.len() > 4 {
            ui.print_info(&format!("Context {}% full. Auto-compacting...", (context_percent * 100.0) as usize));
            messages = compact_messages(&messages, &client, &ui).await;
            total_tokens = estimate_tokens(&messages);
            ui.update_context(total_tokens);
            ui.print_success("Conversation compacted. Continuing...");
        }

        let mut response_started = false;
        ui.reset_code_state();

        // Start animated thinking spinner
        let stop_animation = start_thinking_animation(&ui);

        let result = client
            .chat(&messages, |token| {
                if !response_started {
                    // Stop animation and clear line
                    stop_animation.store(true, Ordering::Relaxed);
                    std::thread::sleep(Duration::from_millis(100)); // Wait for animation to stop
                    ui.clear_line();
                    ui.print_assistant_prefix();
                    response_started = true;
                }
                ui.print_token(token);
            })
            .await;

        // Make sure animation is stopped
        stop_animation.store(true, Ordering::Relaxed);

        match result {
            Ok((content, tool_calls, usage)) => {
                // Update token usage
                total_tokens = usage.total_tokens;
                ui.update_context(total_tokens);
                if !response_started && !content.is_empty() {
                    ui.clear_line();
                    ui.print_assistant_prefix();
                    ui.print_token(&content);
                }

                if !content.is_empty() {
                    ui.print_newline();
                    messages.push(Message {
                        role: "assistant".to_string(),
                        content: MessageContent::Text(content.clone()),
                    });
                }

                // Execute tools with animation
                if !tool_calls.is_empty() {
                    if !response_started {
                        ui.clear_line();
                    }

                    let tool_results = execute_tools_animated(&ui, &tool_calls);

                    let mut iterations = 0;
                    let max_iterations = 10;
                    let mut pending_results = tool_results;

                    while !pending_results.is_empty() && iterations < max_iterations {
                        iterations += 1;

                        let results_text = pending_results
                            .iter()
                            .map(|r| format!("[Tool: {} | Success: {}]\n{}", r.tool_name, r.success, r.output))
                            .collect::<Vec<_>>()
                            .join("\n\n---\n\n");

                        messages.push(Message {
                            role: "user".to_string(),
                            content: MessageContent::Text(format!(
                                "Tool execution results:\n\n{}\n\nContinue with the task.",
                                results_text
                            )),
                        });

                        // Show thinking for follow-up
                        ui.print_thinking(iterations);

                        response_started = false;
                        ui.reset_code_state();

                        // Start animated thinking spinner for follow-up
                        let stop_animation = start_thinking_animation(&ui);

                        let follow_up = client
                            .chat(&messages, |token| {
                                if !response_started {
                                    stop_animation.store(true, Ordering::Relaxed);
                                    std::thread::sleep(Duration::from_millis(100));
                                    ui.clear_line();
                                    ui.print_assistant_prefix();
                                    response_started = true;
                                }
                                ui.print_token(token);
                            })
                            .await;

                        stop_animation.store(true, Ordering::Relaxed);

                        match follow_up {
                            Ok((follow_content, follow_tools, follow_usage)) => {
                                // Update token usage
                                total_tokens = follow_usage.total_tokens;
                                ui.update_context(total_tokens);
                                if !response_started && !follow_content.is_empty() {
                                    ui.clear_line();
                                    ui.print_assistant_prefix();
                                    ui.print_token(&follow_content);
                                }

                                if !follow_content.is_empty() {
                                    ui.print_newline();
                                    messages.push(Message {
                                        role: "assistant".to_string(),
                                        content: MessageContent::Text(follow_content),
                                    });
                                }

                                if follow_tools.is_empty() {
                                    pending_results = Vec::new();
                                } else {
                                    if !response_started {
                                        ui.clear_line();
                                    }
                                    pending_results = execute_tools_animated(&ui, &follow_tools);
                                }
                            }
                            Err(e) => {
                                ui.clear_line();
                                ui.print_error(&format!("API error: {}", e));
                                break;
                            }
                        }
                    }

                    if iterations >= max_iterations {
                        ui.print_info("Max iterations reached.");
                    }
                }
            }
            Err(e) => {
                ui.clear_line();
                ui.print_error(&format!("API error: {}", e));
                messages.pop();
            }
        }

        ui.print_newline();
        ui.print_context_status();
    }

    println!("\n\x1b[36m    {} 🐱\x1b[0m\n", ui.strings.goodbye());
    Ok(())
}

fn execute_tools_animated(ui: &UI, tool_calls: &[ToolCall]) -> Vec<ToolResult> {
    let mut results = Vec::new();

    for tool_call in tool_calls.iter() {
        let input_str = serde_json::to_string_pretty(&tool_call.input).unwrap_or_default();
        ui.print_tool_call(&tool_call.name, &input_str);

        // Brief animation while executing
        for frame in 0..3 {
            ui.print_working(frame, &format!("Executing {}", tool_call.name));
            std::thread::sleep(Duration::from_millis(100));
        }
        ui.clear_line();

        let result = ToolExecutor::execute(tool_call);
        ui.print_tool_result(&result.tool_name, &result.output, result.success);

        results.push(result);
    }

    results
}

enum CommandResult {
    Continue,
    Exit,
    Processed,
}

fn handle_command(
    input: &str,
    ui: &mut UI,
    config: &mut AppConfig,
    client: &mut AzureClient,
    messages: &mut Vec<Message>,
    input_reader: &mut InputReader,
    total_tokens: &mut usize,
) -> CommandResult {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let command = parts.first().map(|s| s.to_lowercase()).unwrap_or_default();
    let args: Vec<&str> = parts.iter().skip(1).cloned().collect();

    match command.as_str() {
        "/exit" | "/quit" | "/q" => CommandResult::Exit,

        "/clear" | "/c" => {
            messages.clear();
            *total_tokens = 0;
            ui.update_context(0);
            if let Some(model) = config.get_active_model() {
                let current_dir = std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| ".".to_string());
                ui.clear_screen();
                ui.set_model_info(&model.name, &model.model_type.to_string(), &current_dir);
                ui.print_banner(&model.name, &model.model_type.to_string(), &current_dir);
                ui.print_welcome_line();
            }
            ui.print_success(ui.strings.cleared());
            CommandResult::Processed
        }

        "/help" | "/h" | "/?" => {
            ui.print_help();
            CommandResult::Processed
        }

        "/config" => {
            if let Some(model) = config.get_active_model() {
                let api_key_preview = if model.api_key.len() > 8 {
                    &model.api_key[..8]
                } else {
                    &model.api_key
                };
                ui.print_config(
                    &model.endpoint,
                    &model.deployment,
                    &model.model_type.to_string(),
                    model.max_tokens,
                    model.temperature,
                    api_key_preview,
                );
            }
            CommandResult::Processed
        }

        "/model" => {
            if args.is_empty() {
                // Interactive model selection
                let models: Vec<(String, String, bool)> = config.models
                    .iter()
                    .map(|(name, model)| {
                        (name.clone(), model.model_type.to_string(), name == &config.active_model)
                    })
                    .collect();

                if let Some(selected_idx) = ui.select_model_interactive(&models) {
                    if selected_idx == models.len() {
                        // "Add model" option selected
                        if let Err(e) = add_model_interactive(config) {
                            ui.print_error(&format!("Failed: {}", e));
                        } else {
                            let model_names: Vec<String> = config.models.keys().cloned().collect();
                            input_reader.update_models(model_names);
                        }
                    } else if selected_idx < models.len() {
                        let (selected_name, _, is_active) = &models[selected_idx];
                        if !is_active {
                            // Switch to selected model
                            config.set_active_model(selected_name);
                            if let Some(model) = config.get_active_model() {
                                client.update_config(model.clone());
                                ui.set_context_max(client.get_max_context());
                                ui.set_model_info(&model.name, &model.model_type.to_string(), &ui.current_path.clone());
                                let _ = save_config(config);
                                ui.print_model_switch(&model.name, &model.model_type.to_string());
                                let model_names: Vec<String> = config.models.keys().cloned().collect();
                                input_reader.update_models(model_names);
                            }
                        }
                        // If already active, do nothing
                    }
                }
            } else {
                let model_name = args.join(" ");

                if config.set_active_model(&model_name) {
                    if let Some(model) = config.get_active_model() {
                        client.update_config(model.clone());
                        ui.set_context_max(client.get_max_context());
                        ui.set_model_info(&model.name, &model.model_type.to_string(), &ui.current_path.clone());
                        let _ = save_config(config);
                        ui.print_model_switch(&model.name, &model.model_type.to_string());

                        let model_names: Vec<String> = config.models.keys().cloned().collect();
                        input_reader.update_models(model_names);
                    }
                } else {
                    let matches: Vec<&String> = config.models.keys()
                        .filter(|k| k.to_lowercase().contains(&model_name.to_lowercase()))
                        .collect();

                    if matches.len() == 1 {
                        let matched_name = matches[0].clone();
                        config.set_active_model(&matched_name);
                        if let Some(model) = config.get_active_model() {
                            client.update_config(model.clone());
                            ui.set_context_max(client.get_max_context());
                            ui.set_model_info(&model.name, &model.model_type.to_string(), &ui.current_path.clone());
                            let _ = save_config(config);
                            ui.print_model_switch(&model.name, &model.model_type.to_string());
                        }
                    } else if matches.is_empty() {
                        ui.print_error(&format!("Model '{}' {}", model_name, ui.strings.not_found()));
                    } else {
                        ui.print_info(&format!("Multiple matches: {}",
                            matches.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
                    }
                }
            }
            CommandResult::Processed
        }

        "/add-model" => {
            if let Err(e) = add_model_interactive(config) {
                ui.print_error(&format!("Failed: {}", e));
            } else {
                let model_names: Vec<String> = config.models.keys().cloned().collect();
                input_reader.update_models(model_names);
            }
            CommandResult::Processed
        }

        "/history" => {
            println!("\n\x1b[36m    Conversation ({} messages)\x1b[0m\n", messages.len());
            for (i, msg) in messages.iter().enumerate() {
                let role_color = if msg.role == "user" { "\x1b[32m" } else { "\x1b[36m" };
                let content = msg.content.as_text();
                let preview = if content.len() > 80 {
                    format!("{}...", &content[..77])
                } else {
                    content
                };
                println!("    {}{:>2}. [{}]\x1b[0m {}", role_color, i + 1, msg.role, preview);
            }
            println!();
            CommandResult::Processed
        }

        "/lang" => {
            if args.is_empty() {
                ui.print_language_menu(config.language);
            } else {
                let lang_str = args[0].to_lowercase();
                let new_lang = match lang_str.as_str() {
                    "en" | "english" | "ing" | "inglês" | "ingles" => Some(Language::En),
                    "pt" | "portuguese" | "português" | "portugues" | "br" => Some(Language::Pt),
                    _ => None,
                };

                if let Some(lang) = new_lang {
                    config.language = lang;
                    ui.set_language(lang);
                    let _ = save_config(config);
                    ui.print_lang_switch(&lang.to_string());
                } else {
                    ui.print_error(&format!("Unknown language: {} (use 'en' or 'pt')", args[0]));
                }
            }
            CommandResult::Processed
        }

        _ => {
            ui.print_error(&format!("{}: {}", ui.strings.unknown_cmd(), command));
            CommandResult::Continue
        }
    }
}

/// Estimate token count for messages (rough: 1 token ≈ 4 chars)
fn estimate_tokens(messages: &[Message]) -> usize {
    messages.iter()
        .map(|m| m.content.as_text().len() / 4)
        .sum()
}

/// Compact messages by summarizing older conversation
async fn compact_messages(messages: &[Message], _client: &AzureClient, _ui: &UI) -> Vec<Message> {
    if messages.len() <= 4 {
        return messages.to_vec();
    }

    // Keep the last 4 messages, summarize the rest
    let to_summarize = &messages[..messages.len() - 4];
    let to_keep = &messages[messages.len() - 4..];

    // Create a summary of older messages
    let summary_text: String = to_summarize.iter()
        .map(|m| {
            let role = if m.role == "user" { "User" } else { "Assistant" };
            let content = m.content.as_text();
            let truncated = if content.len() > 200 {
                format!("{}...", &content[..200])
            } else {
                content
            };
            format!("[{}]: {}", role, truncated)
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Create compacted message history
    let mut compacted = vec![Message {
        role: "user".to_string(),
        content: MessageContent::Text(format!(
            "[Conversation Summary - {} earlier messages]\n{}\n[End of Summary]",
            to_summarize.len(),
            summary_text
        )),
    }];

    // Add the recent messages
    compacted.extend(to_keep.iter().cloned());

    compacted
}
