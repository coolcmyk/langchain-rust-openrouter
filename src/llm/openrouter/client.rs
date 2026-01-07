use crate::{
    language_models::{llm::LLM, options::CallOptions, GenerateResult, LLMError, TokenUsage},
    llm::OpenrouterError,
    schemas::{Message, StreamData},
};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde_json::Value;
use std::{pin::Pin, str};

use super::models::{ApiResponse, OpenrouterMessage, Payload, ProviderPreferences, ResponseFormat};

/// Available OpenRouter models (common examples)
/// You can use any model available on OpenRouter by passing the model ID string
pub enum OpenrouterModel {
    /// OpenAI GPT-4o
    Gpt4o,
    /// OpenAI GPT-4o Mini
    Gpt4oMini,
    /// Anthropic Claude 3.5 Sonnet
    Claude35Sonnet,
    /// Anthropic Claude 3 Opus
    Claude3Opus,
    /// Google Gemini 2.0 Flash
    Gemini2Flash,
    /// Meta Llama 3.1 405B
    Llama31405b,
    /// DeepSeek V3
    DeepseekV3,
    /// Mistral Large
    MistralLarge,
}

impl ToString for OpenrouterModel {
    fn to_string(&self) -> String {
        match self {
            OpenrouterModel::Gpt4o => "openai/gpt-4o".to_string(),
            OpenrouterModel::Gpt4oMini => "openai/gpt-4o-mini".to_string(),
            OpenrouterModel::Claude35Sonnet => "anthropic/claude-3.5-sonnet".to_string(),
            OpenrouterModel::Claude3Opus => "anthropic/claude-3-opus".to_string(),
            OpenrouterModel::Gemini2Flash => "google/gemini-2.0-flash-001".to_string(),
            OpenrouterModel::Llama31405b => "meta-llama/llama-3.1-405b-instruct".to_string(),
            OpenrouterModel::DeepseekV3 => "deepseek/deepseek-chat-v3".to_string(),
            OpenrouterModel::MistralLarge => "mistralai/mistral-large".to_string(),
        }
    }
}

/// OpenRouter client for accessing multiple AI models through a unified API
#[derive(Clone)]
pub struct Openrouter {
    model: String,
    options: CallOptions,
    api_key: String,
    base_url: String,
    json_mode: bool,
    /// HTTP-Referer header for identifying your app on openrouter.ai
    http_referer: Option<String>,
    /// X-Title header for setting your app's title on openrouter.ai
    x_title: Option<String>,
    /// Fallback models to try if the primary model fails
    fallback_models: Option<Vec<String>>,
    /// Provider routing preferences
    provider_preferences: Option<ProviderPreferences>,
    /// Top-k sampling parameter (not available for OpenAI models)
    top_k: Option<u32>,
    /// Repetition penalty
    repetition_penalty: Option<f32>,
    /// Min-p sampling parameter
    min_p: Option<f32>,
    /// Top-a sampling parameter
    top_a: Option<f32>,
    /// Prompt transforms (e.g., "middle-out")
    transforms: Option<Vec<String>>,
    /// Random seed for reproducible outputs
    seed: Option<u64>,
}

impl Default for Openrouter {
    fn default() -> Self {
        Self::new()
    }
}

impl Openrouter {
    pub fn new() -> Self {
        Self {
            model: OpenrouterModel::Gpt4oMini.to_string(),
            options: CallOptions::default(),
            api_key: std::env::var("OPENROUTER_API_KEY").unwrap_or_default(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            json_mode: false,
            http_referer: None,
            x_title: None,
            fallback_models: None,
            provider_preferences: None,
            top_k: None,
            repetition_penalty: None,
            min_p: None,
            top_a: None,
            transforms: None,
            seed: None,
        }
    }

    /// Set the model to use (e.g., "openai/gpt-4o", "anthropic/claude-3.5-sonnet")
    pub fn with_model<S: Into<String>>(mut self, model: S) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_options(mut self, options: CallOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_api_key<S: Into<String>>(mut self, api_key: S) -> Self {
        self.api_key = api_key.into();
        self
    }

    pub fn with_base_url<S: Into<String>>(mut self, base_url: S) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Enable JSON mode for structured output (supported by some models)
    pub fn with_json_mode(mut self, json_mode: bool) -> Self {
        self.json_mode = json_mode;
        self
    }

    /// Set HTTP-Referer header for app identification on openrouter.ai
    pub fn with_http_referer<S: Into<String>>(mut self, http_referer: S) -> Self {
        self.http_referer = Some(http_referer.into());
        self
    }

    /// Set X-Title header for app title on openrouter.ai
    pub fn with_x_title<S: Into<String>>(mut self, x_title: S) -> Self {
        self.x_title = Some(x_title.into());
        self
    }

    /// Set fallback models to try if the primary model fails
    pub fn with_fallback_models(mut self, models: Vec<String>) -> Self {
        self.fallback_models = Some(models);
        self
    }

    /// Set provider routing preferences
    pub fn with_provider_preferences(mut self, preferences: ProviderPreferences) -> Self {
        self.provider_preferences = Some(preferences);
        self
    }

    /// Set top-k sampling parameter (not available for OpenAI models)
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Set repetition penalty (range: 0 to 2)
    pub fn with_repetition_penalty(mut self, repetition_penalty: f32) -> Self {
        self.repetition_penalty = Some(repetition_penalty);
        self
    }

    /// Set min-p sampling parameter (range: 0 to 1)
    pub fn with_min_p(mut self, min_p: f32) -> Self {
        self.min_p = Some(min_p);
        self
    }

    /// Set top-a sampling parameter (range: 0 to 1)
    pub fn with_top_a(mut self, top_a: f32) -> Self {
        self.top_a = Some(top_a);
        self
    }

    /// Set prompt transforms (e.g., ["middle-out"])
    pub fn with_transforms(mut self, transforms: Vec<String>) -> Self {
        self.transforms = Some(transforms);
        self
    }

    /// Set random seed for reproducible outputs
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    async fn generate(&self, messages: &[Message]) -> Result<GenerateResult, LLMError> {
        let client = Client::new();
        let is_stream = self.options.streaming_func.is_some();

        let payload = self.build_payload(messages, is_stream);
        
        let mut request = client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        // Add optional OpenRouter-specific headers
        if let Some(ref referer) = self.http_referer {
            request = request.header("HTTP-Referer", referer);
        }
        if let Some(ref title) = self.x_title {
            request = request.header("X-Title", title);
        }

        let res = request.json(&payload).send().await?;

        let status = res.status().as_u16();

        let res = match status {
            400 => Err(LLMError::OpenrouterError(OpenrouterError::BadRequestError(
                "Invalid request format".to_string(),
            ))),
            401 => Err(LLMError::OpenrouterError(
                OpenrouterError::UnauthorizedError("Invalid API Key".to_string()),
            )),
            402 => Err(LLMError::OpenrouterError(
                OpenrouterError::PaymentRequiredError("Insufficient credits".to_string()),
            )),
            429 => Err(LLMError::OpenrouterError(OpenrouterError::RateLimitError(
                "Rate limit exceeded".to_string(),
            ))),
            502 => Err(LLMError::OpenrouterError(OpenrouterError::BadGatewayError(
                "Provider error".to_string(),
            ))),
            503 => Err(LLMError::OpenrouterError(
                OpenrouterError::ServiceUnavailableError("No available providers".to_string()),
            )),
            529 => Err(LLMError::OpenrouterError(
                OpenrouterError::ProviderOverloadedError("Provider overloaded".to_string()),
            )),
            _ => Ok(res.json::<ApiResponse>().await?),
        }?;

        let choice = res.choices.first();

        let generation = choice
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        let tokens = res.usage.map(|usage| TokenUsage {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        });

        Ok(GenerateResult { tokens, generation })
    }

    fn build_payload(&self, messages: &[Message], stream: bool) -> Payload {
        let mut response_format = None;
        if self.json_mode {
            response_format = Some(ResponseFormat {
                format_type: "json_object".to_string(),
            });
        }

        let mut payload = Payload {
            model: self.model.clone(),
            messages: messages
                .iter()
                .map(OpenrouterMessage::from_message)
                .collect::<Vec<_>>(),
            max_tokens: self.options.max_tokens,
            stream: None,
            temperature: self.options.temperature,
            top_p: self.options.top_p,
            top_k: self.top_k,
            frequency_penalty: None,
            presence_penalty: None,
            repetition_penalty: self.repetition_penalty,
            stop: self.options.stop_words.clone(),
            response_format,
            seed: self.seed,
            min_p: self.min_p,
            top_a: self.top_a,
            transforms: self.transforms.clone(),
            models: self.fallback_models.clone(),
            route: if self.fallback_models.is_some() {
                Some("fallback".to_string())
            } else {
                None
            },
            provider: self.provider_preferences.clone(),
        };

        if stream {
            payload.stream = Some(true);
        }

        // Apply frequency_penalty if it's in the options range
        if let Some(fp) = self.options.frequency_penalty {
            if fp >= -2.0 && fp <= 2.0 {
                payload.frequency_penalty = Some(fp);
            }
        }

        // Apply presence_penalty if it's in the options range
        if let Some(pp) = self.options.presence_penalty {
            if pp >= -2.0 && pp <= 2.0 {
                payload.presence_penalty = Some(pp);
            }
        }

        payload
    }

    fn parse_sse_chunk(chunk: &[u8]) -> Result<Vec<Value>, LLMError> {
        let text = str::from_utf8(chunk).map_err(|e| LLMError::ParsingError(e.to_string()))?;
        let mut values = Vec::new();

        for line in text.lines() {
            // Skip SSE comments (OpenRouter sends these to keep connection alive)
            if line.starts_with(": ") || line.starts_with(":") {
                continue;
            }
            if line.starts_with("data: ") {
                let data = &line[6..];
                if data == "[DONE]" {
                    continue;
                }
                let value: Value = serde_json::from_str(data).map_err(|e| {
                    LLMError::ParsingError(format!("Failed to parse SSE data: {}", e))
                })?;
                values.push(value);
            }
        }

        Ok(values)
    }
}

#[async_trait]
impl LLM for Openrouter {
    async fn generate(&self, messages: &[Message]) -> Result<GenerateResult, LLMError> {
        match &self.options.streaming_func {
            Some(func) => {
                let mut complete_response = String::new();
                let mut stream = self.stream(messages).await?;
                while let Some(data) = stream.next().await {
                    match data {
                        Ok(value) => {
                            let mut func = func.lock().await;
                            complete_response.push_str(&value.content);
                            let _ = func(value.content).await;
                        }
                        Err(e) => return Err(e),
                    }
                }
                let mut generate_result = GenerateResult::default();
                generate_result.generation = complete_response;
                Ok(generate_result)
            }
            None => self.generate(messages).await,
        }
    }

    async fn stream(
        &self,
        messages: &[Message],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, LLMError>> + Send>>, LLMError> {
        let client = Client::new();
        let payload = self.build_payload(messages, true);
        
        let mut request_builder = client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        // Add optional OpenRouter-specific headers
        if let Some(ref referer) = self.http_referer {
            request_builder = request_builder.header("HTTP-Referer", referer);
        }
        if let Some(ref title) = self.x_title {
            request_builder = request_builder.header("X-Title", title);
        }

        let request = request_builder.json(&payload).build()?;

        let stream = client.execute(request).await?;
        let stream = stream.bytes_stream();

        let processed_stream = stream
            .then(move |result| async move {
                match result {
                    Ok(bytes) => {
                        let chunks = Self::parse_sse_chunk(&bytes)?;

                        for chunk in chunks {
                            // Check for mid-stream error
                            if let Some(error) = chunk.get("error") {
                                let error_msg = error
                                    .get("message")
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("Unknown error");
                                return Err(LLMError::OtherError(format!(
                                    "Stream error: {}",
                                    error_msg
                                )));
                            }

                            if let Some(choices) = chunk.get("choices").and_then(|c| c.as_array()) {
                                if let Some(choice) = choices.first() {
                                    if let Some(delta) = choice.get("delta") {
                                        if let Some(content) =
                                            delta.get("content").and_then(|c| c.as_str())
                                        {
                                            if !content.is_empty() {
                                                let usage =
                                                    if let Some(usage) = chunk.get("usage") {
                                                        Some(TokenUsage {
                                                            prompt_tokens: usage
                                                                .get("prompt_tokens")
                                                                .and_then(|t| t.as_u64())
                                                                .unwrap_or(0)
                                                                as u32,
                                                            completion_tokens: usage
                                                                .get("completion_tokens")
                                                                .and_then(|t| t.as_u64())
                                                                .unwrap_or(0)
                                                                as u32,
                                                            total_tokens: usage
                                                                .get("total_tokens")
                                                                .and_then(|t| t.as_u64())
                                                                .unwrap_or(0)
                                                                as u32,
                                                        })
                                                    } else {
                                                        None
                                                    };

                                                return Ok(StreamData::new(
                                                    chunk.clone(),
                                                    usage,
                                                    content,
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // If we didn't return within the loop, return an empty stream data
                        Ok(StreamData::new(Value::Null, None, ""))
                    }
                    Err(e) => Err(LLMError::OtherError(e.to_string())),
                }
            })
            .filter_map(|result| async move {
                match result {
                    Ok(data) if !data.content.is_empty() => Some(Ok(data)),
                    Ok(_) => None,
                    Err(e) => Some(Err(e)),
                }
            });

        Ok(Box::pin(processed_stream))
    }

    fn add_options(&mut self, options: CallOptions) {
        self.options = options;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::{Message, MessageType};

    #[tokio::test]
    #[ignore]
    async fn test_openrouter_generate() {
        let messages = vec![Message {
            content: "Hello".to_string(),
            message_type: MessageType::HumanMessage,
            id: Some("test_id".to_string()),
            images: None,
            tool_calls: None,
        }];

        let client = Openrouter::new();
        let res = client.generate(&messages).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_openrouter_stream() {
        let messages = vec![Message {
            content: "Hello".to_string(),
            message_type: MessageType::HumanMessage,
            id: Some("test_id".to_string()),
            images: None,
            tool_calls: None,
        }];

        let client = Openrouter::new();
        let res = client.stream(&messages).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_openrouter_with_fallback_models() {
        let messages = vec![Message {
            content: "What is 2+2?".to_string(),
            message_type: MessageType::HumanMessage,
            id: Some("test_id".to_string()),
            images: None,
            tool_calls: None,
        }];

        let client = Openrouter::new()
            .with_model(OpenrouterModel::Gpt4o.to_string())
            .with_fallback_models(vec![
                OpenrouterModel::Claude35Sonnet.to_string(),
                OpenrouterModel::Gemini2Flash.to_string(),
            ]);

        let res = client.generate(&messages).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_openrouter_with_provider_preferences() {
        let messages = vec![Message {
            content: "Hello".to_string(),
            message_type: MessageType::HumanMessage,
            id: Some("test_id".to_string()),
            images: None,
            tool_calls: None,
        }];

        let preferences = ProviderPreferences {
            order: Some(vec!["openai".to_string(), "azure".to_string()]),
            allow_fallbacks: Some(true),
            data_collection: Some("deny".to_string()),
            require_parameters: None,
        };

        let client = Openrouter::new()
            .with_model(OpenrouterModel::Gpt4o.to_string())
            .with_provider_preferences(preferences);

        let res = client.generate(&messages).await;
        assert!(res.is_ok());
    }
}
