use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Pt,
}

impl Default for Language {
    fn default() -> Self {
        Language::En
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::En => write!(f, "English"),
            Language::Pt => write!(f, "Português"),
        }
    }
}

pub struct Strings {
    pub lang: Language,
}

impl Strings {
    pub fn new(lang: Language) -> Self {
        Self { lang }
    }

    // Banner & Welcome
    pub fn cli_subtitle(&self) -> &'static str {
        match self.lang {
            Language::En => "Your AI Assistant",
            Language::Pt => "Seu Assistente IA",
        }
    }

    pub fn tips_commands(&self) -> &'static str {
        match self.lang {
            Language::En => "commands",
            Language::Pt => "comandos",
        }
    }

    pub fn tips_files(&self) -> &'static str {
        match self.lang {
            Language::En => "files",
            Language::Pt => "arquivos",
        }
    }

    pub fn tips_quit(&self) -> &'static str {
        match self.lang {
            Language::En => "quit",
            Language::Pt => "sair",
        }
    }

    // Commands help
    pub fn cmd_help(&self) -> &'static str {
        match self.lang {
            Language::En => "Show this help",
            Language::Pt => "Mostra esta ajuda",
        }
    }

    pub fn cmd_exit(&self) -> &'static str {
        match self.lang {
            Language::En => "Exit the CLI",
            Language::Pt => "Sair do CLI",
        }
    }

    pub fn cmd_clear(&self) -> &'static str {
        match self.lang {
            Language::En => "Clear history",
            Language::Pt => "Limpar histórico",
        }
    }

    pub fn cmd_model(&self) -> &'static str {
        match self.lang {
            Language::En => "List models",
            Language::Pt => "Listar modelos",
        }
    }

    pub fn cmd_model_switch(&self) -> &'static str {
        match self.lang {
            Language::En => "Switch model",
            Language::Pt => "Trocar modelo",
        }
    }

    pub fn cmd_add_model(&self) -> &'static str {
        match self.lang {
            Language::En => "Add new model",
            Language::Pt => "Adicionar modelo",
        }
    }

    pub fn cmd_config(&self) -> &'static str {
        match self.lang {
            Language::En => "Show config",
            Language::Pt => "Mostrar config",
        }
    }

    pub fn cmd_lang(&self) -> &'static str {
        match self.lang {
            Language::En => "Change language",
            Language::Pt => "Mudar idioma",
        }
    }

    // Section titles
    pub fn title_commands(&self) -> &'static str {
        match self.lang {
            Language::En => "Commands",
            Language::Pt => "Comandos",
        }
    }

    pub fn title_models(&self) -> &'static str {
        match self.lang {
            Language::En => "Available Models",
            Language::Pt => "Modelos Disponíveis",
        }
    }

    pub fn title_config(&self) -> &'static str {
        match self.lang {
            Language::En => "Configuration",
            Language::Pt => "Configuração",
        }
    }

    pub fn title_context(&self) -> &'static str {
        match self.lang {
            Language::En => "Context",
            Language::Pt => "Contexto",
        }
    }

    pub fn title_language(&self) -> &'static str {
        match self.lang {
            Language::En => "Language",
            Language::Pt => "Idioma",
        }
    }

    // Messages
    pub fn thinking(&self) -> &'static str {
        match self.lang {
            Language::En => "Thinking...",
            Language::Pt => "Pensando...",
        }
    }

    pub fn executing(&self) -> &'static str {
        match self.lang {
            Language::En => "Executing",
            Language::Pt => "Executando",
        }
    }

    pub fn switched_to(&self) -> &'static str {
        match self.lang {
            Language::En => "Switched to",
            Language::Pt => "Trocado para",
        }
    }

    pub fn cleared(&self) -> &'static str {
        match self.lang {
            Language::En => "Conversation cleared",
            Language::Pt => "Conversa limpa",
        }
    }

    pub fn goodbye(&self) -> &'static str {
        match self.lang {
            Language::En => "Goodbye!",
            Language::Pt => "Até logo!",
        }
    }

    pub fn not_found(&self) -> &'static str {
        match self.lang {
            Language::En => "not found",
            Language::Pt => "não encontrado",
        }
    }

    pub fn unknown_cmd(&self) -> &'static str {
        match self.lang {
            Language::En => "Unknown command (try /help)",
            Language::Pt => "Comando desconhecido (tente /help)",
        }
    }

    pub fn file_context_hint(&self) -> &'static str {
        match self.lang {
            Language::En => "Use @path/file to include files",
            Language::Pt => "Use @caminho/arquivo para incluir arquivos",
        }
    }

    pub fn example(&self) -> &'static str {
        match self.lang {
            Language::En => "Example",
            Language::Pt => "Exemplo",
        }
    }

    pub fn select_language(&self) -> &'static str {
        match self.lang {
            Language::En => "Select language",
            Language::Pt => "Selecione o idioma",
        }
    }

    pub fn language_changed(&self) -> &'static str {
        match self.lang {
            Language::En => "Language changed to",
            Language::Pt => "Idioma alterado para",
        }
    }

    pub fn current(&self) -> &'static str {
        match self.lang {
            Language::En => "current",
            Language::Pt => "atual",
        }
    }

    pub fn model_switch_hint(&self) -> &'static str {
        match self.lang {
            Language::En => "/model <name> to switch",
            Language::Pt => "/model <nome> para trocar",
        }
    }

    pub fn add_model_hint(&self) -> &'static str {
        match self.lang {
            Language::En => "/add-model to add new",
            Language::Pt => "/add-model para adicionar",
        }
    }

    pub fn ctrl_c_hint(&self) -> &'static str {
        match self.lang {
            Language::En => "Ctrl+C - type /exit to quit",
            Language::Pt => "Ctrl+C - digite /exit para sair",
        }
    }
}
