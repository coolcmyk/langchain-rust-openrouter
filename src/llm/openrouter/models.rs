use crate::schemas::{Message, MessageType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct OpenrouterMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl OpenrouterMessage {
    pub fn new<S: Into<String>>(role: S, content: S) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
            name: None,
        }
    }

    pub fn from_message(message: &Message) -> Self {
        match message.message_type {
            MessageType::SystemMessage => Self::new("system", &message.content),
            MessageType::AIMessage => Self::new("assistant", &message.content),
            MessageType::HumanMessage => Self::new("user", &message.content),
            MessageType::ToolMessage => Self::new("tool", &message.content),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
}

/// Provider preferences for routing requests
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProviderPreferences {
    /// Ordered list of provider names to prefer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,
    /// Whether to allow fallbacks to other providers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,
    /// Data collection preference: "allow" or "deny"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_collection: Option<String>,
    /// Require specific parameters to be supported
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_parameters: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Payload {
    pub model: String,
    pub messages: Vec<OpenrouterMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_a: Option<f32>,
    // OpenRouter-specific parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderPreferences>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ChoiceMessage {
    pub role: String,
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Choice {
    pub message: ChoiceMessage,
    pub finish_reason: Option<String>,
    #[serde(default)]
    pub native_finish_reason: Option<String>,
    pub index: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ApiResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub usage: Option<Usage>,
    #[serde(default)]
    pub system_fingerprint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Delta {
    #[serde(default)]
    pub content: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct StreamChoice {
    pub delta: Delta,
    pub finish_reason: Option<String>,
    #[serde(default)]
    pub native_finish_reason: Option<String>,
    pub index: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct StreamResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<StreamChoice>,
    #[serde(default)]
    pub system_fingerprint: Option<String>,
    #[serde(default)]
    pub usage: Option<Usage>,
}
