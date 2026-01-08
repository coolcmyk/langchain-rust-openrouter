# LangChain Rust OpenRouter

A fork of [langchain-rust](https://github.com/Abraxas-365/langchain-rust) with first-class support for [OpenRouter](https://openrouter.ai), providing unified access to multiple AI model providers through a single API.

## Overview

This library extends langchain-rust with native OpenRouter integration, allowing you to:

- Access 200+ LLM models from OpenAI, Anthropic, Google, Meta, Mistral, and more through a single API
- Use OpenRouter's unified embeddings API with multiple embedding models
- Leverage automatic fallback routing between providers
- Configure provider preferences for cost optimization and data privacy

## Installation

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
langchain-rust = { git = "https://github.com/coolcmyk/langchain-rust-openrouter" }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
```

## Environment Variables

Set your OpenRouter API key:

```bash
export OPENROUTER_API_KEY="your-api-key-here"
```

## Usage

### LLM Client

The `Openrouter` client provides access to all models available on OpenRouter.

#### Basic Usage

```rust
use langchain_rust::llm::{Openrouter, OpenrouterModel};
use langchain_rust::language_models::llm::LLM;

#[tokio::main]
async fn main() {
    // Initialize with defaults (reads OPENROUTER_API_KEY from environment)
    let client = Openrouter::default();
    
    // Or configure explicitly
    let client = Openrouter::default()
        .with_api_key("your-api-key")
        .with_model(OpenrouterModel::Gpt4oMini.to_string());
    
    let response = client.invoke("What is Rust?").await.unwrap();
    println!("{}", response);
}
```

#### Using Different Models

```rust
use langchain_rust::llm::{Openrouter, OpenrouterModel};

// Using predefined model enum
let client = Openrouter::default()
    .with_model(OpenrouterModel::Claude35Sonnet.to_string());

// Using custom model string
let client = Openrouter::default()
    .with_model("google/gemini-2.0-flash-001");

// Available predefined models:
// - OpenrouterModel::Gpt4o
// - OpenrouterModel::Gpt4oMini
// - OpenrouterModel::Claude35Sonnet
// - OpenrouterModel::Claude3Opus
// - OpenrouterModel::Claude3Haiku
// - OpenrouterModel::Claude4_5Haiku
// - OpenrouterModel::Gemini2Flash
// - OpenrouterModel::Llama31405b
// - OpenrouterModel::DeepseekV3
// - OpenrouterModel::MistralLarge
```

#### Fallback Models

Configure automatic fallback to alternative models if the primary model fails:

```rust
use langchain_rust::llm::Openrouter;

let client = Openrouter::default()
    .with_model("openai/gpt-4o")
    .with_fallback_models(vec![
        "anthropic/claude-3.5-sonnet".to_string(),
        "google/gemini-2.0-flash-001".to_string(),
    ]);
```

#### Provider Preferences

Control which providers serve your requests:

```rust
use langchain_rust::llm::{Openrouter, ProviderPreferences};

let preferences = ProviderPreferences {
    order: Some(vec!["openai".to_string(), "azure".to_string()]),
    allow_fallbacks: Some(true),
    data_collection: Some("deny".to_string()),
    require_parameters: None,
};

let client = Openrouter::default()
    .with_model("openai/gpt-4o")
    .with_provider_preferences(preferences);
```

#### Streaming Responses

```rust
use langchain_rust::llm::Openrouter;
use langchain_rust::language_models::llm::LLM;
use langchain_rust::schemas::{Message, MessageType};
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let client = Openrouter::default();
    
    let messages = vec![Message {
        content: "Write a short poem about Rust".to_string(),
        message_type: MessageType::HumanMessage,
        id: None,
        images: None,
        tool_calls: None,
    }];
    
    let mut stream = client.stream(&messages).await.unwrap();
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(data) => print!("{}", data.content),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
```

#### Using with Chains

```rust
use langchain_rust::{
    chain::{Chain, LLMChainBuilder},
    llm::Openrouter,
    fmt_message, fmt_template,
    message_formatter,
    prompt::HumanMessagePromptTemplate,
    prompt_args,
    schemas::Message,
    template_fstring,
};

#[tokio::main]
async fn main() {
    let client = Openrouter::default()
        .with_model("anthropic/claude-3.5-sonnet");
    
    let prompt = message_formatter![
        fmt_message!(Message::new_system_message(
            "You are a helpful assistant."
        )),
        fmt_template!(HumanMessagePromptTemplate::new(template_fstring!(
            "{input}", "input"
        )))
    ];
    
    let chain = LLMChainBuilder::new()
        .prompt(prompt)
        .llm(client)
        .build()
        .unwrap();
    
    let result = chain
        .invoke(prompt_args! { "input" => "What is the capital of France?" })
        .await
        .unwrap();
    
    println!("{}", result);
}
```

### Embeddings

The `OpenrouterEmbedder` provides access to embedding models through OpenRouter.

#### Basic Usage

```rust
use langchain_rust::embedding::{Embedder, OpenrouterEmbedder, OpenrouterEmbeddingModel};

#[tokio::main]
async fn main() {
    // Initialize with defaults
    let embedder = OpenrouterEmbedder::default();
    
    // Or configure explicitly
    let embedder = OpenrouterEmbedder::default()
        .with_api_key("your-api-key")
        .with_model(OpenrouterEmbeddingModel::TextEmbedding3Small.to_string());
    
    // Embed a single query
    let embedding = embedder.embed_query("Hello world").await.unwrap();
    println!("Embedding dimensions: {}", embedding.len());
}
```

#### Batch Document Embedding

```rust
use langchain_rust::embedding::{Embedder, OpenrouterEmbedder};

#[tokio::main]
async fn main() {
    let embedder = OpenrouterEmbedder::default()
        .with_model("openai/text-embedding-3-small");
    
    let documents = vec![
        "First document about machine learning".to_string(),
        "Second document about rust programming".to_string(),
        "Third document about embeddings".to_string(),
    ];
    
    let embeddings = embedder.embed_documents(&documents).await.unwrap();
    
    for (i, embedding) in embeddings.iter().enumerate() {
        println!("Document {}: {} dimensions", i, embedding.len());
    }
}
```

#### Available Embedding Models

| Model | Dimensions | Description |
|-------|------------|-------------|
| `openai/text-embedding-3-small` | 1536 | Fast and cost-effective |
| `openai/text-embedding-3-large` | 3072 | Higher quality embeddings |
| `openai/text-embedding-ada-002` | 1536 | Legacy model |
| `qwen/qwen3-embedding-0.6b` | Variable | Lightweight option |
| `qwen/qwen3-embedding-4b` | Variable | Balanced performance |

#### Using with Vector Stores

```rust
use langchain_rust::embedding::{Embedder, OpenrouterEmbedder};
use langchain_rust::vectorstore::qdrant::Qdrant;

#[tokio::main]
async fn main() {
    let embedder = OpenrouterEmbedder::default()
        .with_model("openai/text-embedding-3-small");
    
    // Use with Qdrant vector store
    let vector_store = Qdrant::new(
        "http://localhost:6334",
        "my_collection",
        Box::new(embedder),
    ).await.unwrap();
    
    // Add documents, search, etc.
}
```

## Configuration Options

### LLM Client Options

| Method | Description |
|--------|-------------|
| `with_model()` | Set the model to use |
| `with_api_key()` | Set the API key |
| `with_base_url()` | Set custom base URL |
| `with_json_mode()` | Enable JSON output mode |
| `with_http_referer()` | Set app identification header |
| `with_x_title()` | Set app title header |
| `with_fallback_models()` | Set fallback model list |
| `with_provider_preferences()` | Configure provider routing |
| `with_top_k()` | Set top-k sampling |
| `with_repetition_penalty()` | Set repetition penalty |
| `with_min_p()` | Set min-p sampling |
| `with_top_a()` | Set top-a sampling |
| `with_seed()` | Set random seed |

### Embedder Options

| Method | Description |
|--------|-------------|
| `with_model()` | Set the embedding model |
| `with_api_key()` | Set the API key |
| `with_base_url()` | Set custom base URL |
| `with_http_referer()` | Set app identification header |
| `with_x_title()` | Set app title header |

## Error Handling

The library provides specific error types for OpenRouter API responses:

```rust
use langchain_rust::llm::OpenrouterError;

// Error variants:
// - BadRequestError (400)
// - UnauthorizedError (401)
// - PaymentRequiredError (402)
// - RateLimitError (429)
// - BadGatewayError (502)
// - ServiceUnavailableError (503)
// - ProviderOverloadedError (529)
```

## Additional Features

This fork includes all features from the original langchain-rust:

- Vector Stores: Qdrant, Postgres, SQLite, SurrealDB, OpenSearch
- Document Loaders: PDF, HTML, CSV, Pandoc, Git commits, Source code
- Chains: LLM Chain, Conversational Chain, Q&A Chain, SQL Chain
- Agents: Chat Agent with Tools, OpenAI Tools Agent
- Semantic Routing: Static and Dynamic routing

## Credits

This project is a fork of [langchain-rust](https://github.com/Abraxas-365/langchain-rust) by Abraxas-365.

## License

MIT License - see the original repository for details.
