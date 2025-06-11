use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

/// Represents different AI model providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum Provider {
    #[strum(serialize = "openai")]
    OpenAI,
    #[strum(serialize = "anthropic")]
    Anthropic,
    #[strum(serialize = "google")]
    Google,
    #[strum(serialize = "xai")]
    XAI,
    #[strum(serialize = "groq")]
    Groq,
    #[strum(serialize = "deepseek")]
    DeepSeek,
    #[strum(serialize = "openrouter")]
    OpenRouter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum Company {
    #[strum(serialize = "openai")]
    OpenAI,
    #[strum(serialize = "anthropic")]
    Anthropic,
    #[strum(serialize = "google")]
    Google,
    #[strum(serialize = "xai")]
    XAI,
    #[strum(serialize = "groq")]
    Groq,
    #[strum(serialize = "deepseek")]
    DeepSeek,
    #[strum(serialize = "openrouter")]
    OpenRouter,
}

/// Represents the capabilities of a model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Capabilities {
    pub text: bool,
    pub image_generation: bool,
    pub image_understanding: bool,
    pub web_search: bool,
    pub file_upload: bool,
    pub function_calling: bool,
}

/// Configuration for an AI model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelConfig {
    pub name: String,
    pub provider: Provider,
    pub company: Company,
    pub max_tokens: usize,
    pub capabilities: Capabilities,
    pub description: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            name: "gpt-4o".to_string(),
            provider: Provider::OpenAI,
            company: Company::OpenAI,
            max_tokens: 128000,
            capabilities: Capabilities::default(),
            description: "OpenAI's most advanced model".to_string(),
        }
    }
}
