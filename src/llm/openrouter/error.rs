use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenrouterError {
    #[error("OpenRouter API error: Bad Request - {0}")]
    BadRequestError(String),

    #[error("OpenRouter API error: Unauthorized - {0}")]
    UnauthorizedError(String),

    #[error("OpenRouter API error: Payment Required - {0}")]
    PaymentRequiredError(String),

    #[error("OpenRouter API error: Rate Limit Exceeded - {0}")]
    RateLimitError(String),

    #[error("OpenRouter API error: Bad Gateway - {0}")]
    BadGatewayError(String),

    #[error("OpenRouter API error: Service Unavailable - {0}")]
    ServiceUnavailableError(String),

    #[error("OpenRouter API error: Provider Overloaded - {0}")]
    ProviderOverloadedError(String),
}
