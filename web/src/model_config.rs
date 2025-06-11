use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use strum_macros::{Display, EnumString};

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
    #[strum(serialize = "deepseek")]
    DeepSeek,
    #[strum(serialize = "meta")]
    Meta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Capabilities {
    pub image_gen: bool,
    pub web_search: bool,
    pub image_analysis: bool,
    pub reasoning: bool,
    pub pdf_analysis: bool,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            image_gen: false,
            web_search: false,
            image_analysis: false,
            reasoning: true,
            pdf_analysis: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelConfig {
    pub provider: Provider,
    pub company: Company,
    pub model_name: String,
    #[serde(flatten)]
    pub capabilities: Capabilities,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: Provider::OpenAI,
            company: Company::OpenAI,
            model_name: "gpt-4o".to_string(),
            capabilities: Capabilities::default(),
        }
    }
}

impl ModelConfig {
    // Load models from a JSON file
    pub fn load_models() -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let json_path = Path::new("models.json");
        let json_content = fs::read_to_string(json_path)?;
        let models: Vec<ModelConfig> = serde_json::from_str(&json_content)?;
        Ok(models)
    }

    // Get the default model (first one in the list)
    pub fn default_model() -> Self {
        Self::load_models()
            .and_then(|mut models| models.pop().ok_or_else(|| "No models found".into()))
            .unwrap_or_default()
    }

    // Helper method to check if this is a DALL-E model
    pub fn is_dalle(&self) -> bool {
        self.capabilities.image_gen
    }
}
