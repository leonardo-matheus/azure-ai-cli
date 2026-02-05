use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use crate::i18n::Language;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub active_model: String,
    pub models: HashMap<String, ModelConfig>,
    #[serde(default)]
    pub github_username: String,
    #[serde(default)]
    pub language: Language,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub api_key: String,
    pub endpoint: String,
    pub deployment: String,
    pub model_type: ModelType,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

// Legacy config for backwards compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyConfig {
    pub api_key: String,
    pub endpoint: String,
    pub deployment: String,
    pub model_type: ModelType,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_max_tokens() -> u32 { 4096 }
fn default_temperature() -> f32 { 0.7 }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Claude,
    Gpt,
    DeepSeek,
    Other,
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Claude => write!(f, "Claude"),
            ModelType::Gpt => write!(f, "GPT"),
            ModelType::DeepSeek => write!(f, "DeepSeek"),
            ModelType::Other => write!(f, "Other"),
        }
    }
}

pub fn get_config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".aicli").join("config.toml")
}

impl AppConfig {
    pub fn get_active_model(&self) -> Option<&ModelConfig> {
        self.models.get(&self.active_model)
    }

    pub fn set_active_model(&mut self, name: &str) -> bool {
        if self.models.contains_key(name) {
            self.active_model = name.to_string();
            true
        } else {
            false
        }
    }

    pub fn add_model(&mut self, model: ModelConfig) {
        let name = model.name.clone();
        self.models.insert(name.clone(), model);
        if self.active_model.is_empty() {
            self.active_model = name;
        }
    }

    pub fn list_models(&self) -> Vec<(&String, &ModelConfig)> {
        self.models.iter().collect()
    }
}

pub fn load_config() -> Result<AppConfig> {
    // Try environment variables first
    if let (Ok(api_key), Ok(endpoint), Ok(deployment)) = (
        std::env::var("AZURE_API_KEY"),
        std::env::var("AZURE_ENDPOINT"),
        std::env::var("AZURE_DEPLOYMENT"),
    ) {
        let model_type = detect_model_type(&deployment);
        let model = ModelConfig {
            name: deployment.clone(),
            api_key,
            endpoint,
            deployment: deployment.clone(),
            model_type,
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
        };

        let mut models = HashMap::new();
        models.insert(deployment.clone(), model);

        return Ok(AppConfig {
            active_model: deployment,
            models,
            github_username: "leonardo-matheus".to_string(),
            language: Language::default(),
        });
    }

    // Load from config file
    let config_path = get_config_path();
    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config from {:?}", config_path))?;

    // Try new format first
    if let Ok(config) = toml::from_str::<AppConfig>(&content) {
        return Ok(config);
    }

    // Fall back to legacy format
    let legacy: LegacyConfig = toml::from_str(&content)
        .with_context(|| "Failed to parse config file")?;

    let model = ModelConfig {
        name: legacy.deployment.clone(),
        api_key: legacy.api_key,
        endpoint: legacy.endpoint,
        deployment: legacy.deployment.clone(),
        model_type: legacy.model_type,
        max_tokens: legacy.max_tokens,
        temperature: legacy.temperature,
    };

    let mut models = HashMap::new();
    models.insert(legacy.deployment.clone(), model);

    Ok(AppConfig {
        active_model: legacy.deployment,
        models,
        github_username: "leonardo-matheus".to_string(),
        language: Language::default(),
    })
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(config)?;
    fs::write(&config_path, content)?;

    Ok(())
}

pub fn detect_model_type(deployment: &str) -> ModelType {
    let lower = deployment.to_lowercase();
    if lower.contains("claude") || lower.contains("anthropic") {
        ModelType::Claude
    } else if lower.contains("gpt") || lower.contains("o1") || lower.contains("o3") {
        ModelType::Gpt
    } else if lower.contains("deepseek") || lower.contains("r1") {
        ModelType::DeepSeek
    } else {
        ModelType::Other
    }
}

pub async fn setup_config_interactive() -> Result<AppConfig> {
    println!("\x1b[36m╔═══════════════════════════════════════════════════════════════╗\x1b[0m");
    println!("\x1b[36m║              AICLI Configuration Setup                        ║\x1b[0m");
    println!("\x1b[36m╚═══════════════════════════════════════════════════════════════╝\x1b[0m\n");

    let mut config = load_config().unwrap_or_else(|_| AppConfig {
        active_model: String::new(),
        models: HashMap::new(),
        github_username: "leonardo-matheus".to_string(),
        language: Language::default(),
    });

    loop {
        println!("\x1b[33mAdd a new model configuration:\x1b[0m\n");

        print!("\x1b[33mModel name (e.g., gpt-4, claude-opus):\x1b[0m ");
        io::stdout().flush()?;
        let mut name = String::new();
        io::stdin().read_line(&mut name)?;
        let name = name.trim().to_string();

        print!("\x1b[33mAzure AI Endpoint URL:\x1b[0m ");
        io::stdout().flush()?;
        let mut endpoint = String::new();
        io::stdin().read_line(&mut endpoint)?;
        let endpoint = endpoint.trim().to_string();

        print!("\x1b[33mAPI Key:\x1b[0m ");
        io::stdout().flush()?;
        let mut api_key = String::new();
        io::stdin().read_line(&mut api_key)?;
        let api_key = api_key.trim().to_string();

        print!("\x1b[33mDeployment/Model ID:\x1b[0m ");
        io::stdout().flush()?;
        let mut deployment = String::new();
        io::stdin().read_line(&mut deployment)?;
        let deployment = deployment.trim().to_string();

        println!("\n\x1b[33mSelect model type:\x1b[0m");
        println!("  1. Claude (Anthropic)");
        println!("  2. GPT (OpenAI)");
        println!("  3. DeepSeek");
        println!("  4. Other");
        print!("\x1b[33mChoice [1-4]:\x1b[0m ");
        io::stdout().flush()?;
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;

        let model_type = match choice.trim() {
            "1" => ModelType::Claude,
            "2" => ModelType::Gpt,
            "3" => ModelType::DeepSeek,
            _ => detect_model_type(&deployment),
        };

        print!("\x1b[33mMax tokens [4096]:\x1b[0m ");
        io::stdout().flush()?;
        let mut max_tokens_str = String::new();
        io::stdin().read_line(&mut max_tokens_str)?;
        let max_tokens: u32 = max_tokens_str.trim().parse().unwrap_or(4096);

        print!("\x1b[33mTemperature [0.7]:\x1b[0m ");
        io::stdout().flush()?;
        let mut temp_str = String::new();
        io::stdin().read_line(&mut temp_str)?;
        let temperature: f32 = temp_str.trim().parse().unwrap_or(0.7);

        let model = ModelConfig {
            name: name.clone(),
            api_key,
            endpoint,
            deployment,
            model_type,
            max_tokens,
            temperature,
        };

        config.add_model(model);
        println!("\n\x1b[32m✓ Model '{}' added!\x1b[0m", name);

        print!("\n\x1b[33mAdd another model? [y/N]:\x1b[0m ");
        io::stdout().flush()?;
        let mut another = String::new();
        io::stdin().read_line(&mut another)?;
        if !another.trim().to_lowercase().starts_with('y') {
            break;
        }
        println!();
    }

    save_config(&config)?;
    println!("\n\x1b[32m✓ Configuration saved to {:?}\x1b[0m", get_config_path());

    Ok(config)
}

pub fn add_model_interactive(config: &mut AppConfig) -> Result<()> {
    println!("\n\x1b[36m━━━ Add New Model ━━━\x1b[0m\n");

    print!("\x1b[33mModel name:\x1b[0m ");
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let name = name.trim().to_string();

    print!("\x1b[33mEndpoint URL:\x1b[0m ");
    io::stdout().flush()?;
    let mut endpoint = String::new();
    io::stdin().read_line(&mut endpoint)?;
    let endpoint = endpoint.trim().to_string();

    print!("\x1b[33mAPI Key:\x1b[0m ");
    io::stdout().flush()?;
    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim().to_string();

    print!("\x1b[33mDeployment ID:\x1b[0m ");
    io::stdout().flush()?;
    let mut deployment = String::new();
    io::stdin().read_line(&mut deployment)?;
    let deployment = deployment.trim().to_string();

    let model_type = detect_model_type(&deployment);

    let model = ModelConfig {
        name: name.clone(),
        api_key,
        endpoint,
        deployment,
        model_type,
        max_tokens: default_max_tokens(),
        temperature: default_temperature(),
    };

    config.add_model(model);
    save_config(config)?;
    println!("\x1b[32m✓ Model '{}' added!\x1b[0m\n", name);

    Ok(())
}
