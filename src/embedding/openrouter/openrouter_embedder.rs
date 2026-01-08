use crate::embedding::{embedder_trait::Embedder, EmbedderError};
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

/// Available OpenRouter embedding models (common examples)
/// You can use any embedding model available on OpenRouter by passing the model ID string
pub enum OpenrouterEmbeddingModel {
    /// OpenAI text-embedding-3-small (1536 dimensions)
    TextEmbedding3Small,
    /// OpenAI text-embedding-3-large (3072 dimensions)
    TextEmbedding3Large,
    /// OpenAI text-embedding-ada-002 (1536 dimensions)
    TextEmbeddingAda002,
    /// Qwen3 Embedding 0.6B
    Qwen3Embedding06b,
    /// Qwen3 Embedding 4B
    Qwen3Embedding4b,
}

impl ToString for OpenrouterEmbeddingModel {
    fn to_string(&self) -> String {
        match self {
            OpenrouterEmbeddingModel::TextEmbedding3Small => {
                "openai/text-embedding-3-small".to_string()
            }
            OpenrouterEmbeddingModel::TextEmbedding3Large => {
                "openai/text-embedding-3-large".to_string()
            }
            OpenrouterEmbeddingModel::TextEmbeddingAda002 => {
                "openai/text-embedding-ada-002".to_string()
            }
            OpenrouterEmbeddingModel::Qwen3Embedding06b => {
                "qwen/qwen3-embedding-0.6b".to_string()
            }
            OpenrouterEmbeddingModel::Qwen3Embedding4b => "qwen/qwen3-embedding-4b".to_string(),
        }
    }
}

#[derive(Serialize, Debug)]
struct EmbeddingRequest {
    model: String,
    input: EmbeddingInput,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
enum EmbeddingInput {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Deserialize, Debug)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize, Debug)]
struct EmbeddingData {
    embedding: Vec<f64>,
    #[allow(dead_code)]
    index: usize,
}

/// OpenRouter embedder for generating text embeddings through the OpenRouter API
#[derive(Debug, Clone)]
pub struct OpenrouterEmbedder {
    api_key: String,
    base_url: String,
    model: String,
    /// HTTP-Referer header for identifying your app on openrouter.ai
    http_referer: Option<String>,
    /// X-Title header for setting your app's title on openrouter.ai
    x_title: Option<String>,
}

const DEFAULT_MODEL: &str = "openai/text-embedding-3-small";

impl OpenrouterEmbedder {
    pub fn new<S: Into<String>>(api_key: S, model: S) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model: model.into(),
            http_referer: None,
            x_title: None,
        }
    }

    /// Set the embedding model to use
    pub fn with_model<S: Into<String>>(mut self, model: S) -> Self {
        self.model = model.into();
        self
    }

    /// Set the API key
    pub fn with_api_key<S: Into<String>>(mut self, api_key: S) -> Self {
        self.api_key = api_key.into();
        self
    }

    /// Set the base URL (default: https://openrouter.ai/api/v1)
    pub fn with_base_url<S: Into<String>>(mut self, base_url: S) -> Self {
        self.base_url = base_url.into();
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

    async fn send_request(&self, input: EmbeddingInput) -> Result<EmbeddingResponse, EmbedderError> {
        let client = Client::new();
        
        let request_body = EmbeddingRequest {
            model: self.model.clone(),
            input,
        };

        let mut request = client
            .post(&format!("{}/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        // Add optional OpenRouter-specific headers
        if let Some(ref referer) = self.http_referer {
            request = request.header("HTTP-Referer", referer);
        }
        if let Some(ref title) = self.x_title {
            request = request.header("X-Title", title);
        }

        let response = request.json(&request_body).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(EmbedderError::HttpError {
                status_code: status,
                error_message,
            });
        }

        let embedding_response: EmbeddingResponse = response.json().await?;
        Ok(embedding_response)
    }
}

impl Default for OpenrouterEmbedder {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENROUTER_API_KEY").unwrap_or_default(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model: String::from(DEFAULT_MODEL),
            http_referer: None,
            x_title: None,
        }
    }
}

#[async_trait]
impl Embedder for OpenrouterEmbedder {
    async fn embed_documents(&self, documents: &[String]) -> Result<Vec<Vec<f64>>, EmbedderError> {
        log::debug!("Embedding documents: {:?}", documents);

        let response = self
            .send_request(EmbeddingInput::Multiple(documents.to_vec()))
            .await?;

        // Sort by index to ensure correct order
        let mut data = response.data;
        data.sort_by_key(|d| d.index);

        let embeddings = data.into_iter().map(|d| d.embedding).collect();

        Ok(embeddings)
    }

    async fn embed_query(&self, text: &str) -> Result<Vec<f64>, EmbedderError> {
        log::debug!("Embedding query: {:?}", text);

        let response = self
            .send_request(EmbeddingInput::Single(text.to_string()))
            .await?;

        let embedding = response
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .unwrap_or_default();

        Ok(embedding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_openrouter_embed_query() {
        let embedder = OpenrouterEmbedder::default()
            .with_model(OpenrouterEmbeddingModel::TextEmbedding3Small.to_string());

        let response = embedder.embed_query("Why is the sky blue?").await.unwrap();

        // text-embedding-3-small produces 1536-dimensional embeddings
        assert_eq!(response.len(), 1536);
    }

    #[tokio::test]
    #[ignore]
    async fn test_openrouter_embed_documents() {
        let embedder = OpenrouterEmbedder::default()
            .with_model(OpenrouterEmbeddingModel::TextEmbedding3Small.to_string());

        let documents = vec![
            "The cat sat on the mat".to_string(),
            "Dogs are loyal companions".to_string(),
        ];

        let response = embedder.embed_documents(&documents).await.unwrap();

        assert_eq!(response.len(), 2);
        assert_eq!(response[0].len(), 1536);
        assert_eq!(response[1].len(), 1536);
    }
}
