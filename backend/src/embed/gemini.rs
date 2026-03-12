use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, instrument};

use crate::{embed::MessageEmbedder, AppError};

#[derive(Debug, Clone)]
pub struct GeminiMessageEmbedder {
    api_key: String,
}

impl GeminiMessageEmbedder {
    pub fn new(api_key: impl Into<String>) -> Result<Self, AppError> {
        let api_key = api_key.into();
        if api_key.trim().is_empty() {
            return Err(AppError::GeminiError(GeminiError::MissingApiKey));
        }

        Ok(Self { api_key })
    }
}

impl MessageEmbedder for GeminiMessageEmbedder {
    #[instrument(skip_all, fields(username = _username, content = %crate::truncate_chars(text, 10), resp_status = tracing::field::Empty))]
    async fn embed_text(
        &self,
        text: &str,
        client: &reqwest::Client,
        _username: &str,
    ) -> Result<Vec<f32>, AppError> {
        let cleaned = text.replace('\n', "");
        let cleaned = cleaned.trim();

        if cleaned.is_empty() {
            return Err(AppError::GeminiError(GeminiError::EmptyText));
        }

        let body = serde_json::json!( {
            "model": "models/gemini-embedding-001",
            "content": Content {
                parts: vec![Part { text: cleaned }],
            },
        });

        let resp = client
            .post("https://generativelanguage.googleapis.com/v1beta/models/gemini-embedding-001:embedContent")
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::GeminiError(GeminiError::Reqwest(e)))?;

        tracing::Span::current().record("resp_status", resp.status().to_string());

        let parsed: EmbedContentResponse = resp
            .json()
            .await
            .map_err(|e| AppError::GeminiError(GeminiError::Reqwest(e)))?;

        Ok(parsed.embedding.values)
    }
}

#[derive(Debug, Serialize)]
struct Content<'a> {
    parts: Vec<Part<'a>>,
}

#[derive(Debug, Serialize)]
struct Part<'a> {
    text: &'a str,
}

#[derive(Debug, Deserialize)]
struct EmbedContentResponse {
    embedding: ContentEmbedding,
}

#[derive(Debug, Deserialize)]
struct ContentEmbedding {
    values: Vec<f32>,
}

#[derive(Debug, Error)]
pub enum GeminiError {
    #[error("missing api key (or provided api key was empty)")]
    MissingApiKey,

    #[error("input text is empty after trimming/cleaning")]
    EmptyText,

    #[error("http client error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("unexpected HTTP status {status} with body: {body}")]
    HttpStatus { status: StatusCode, body: String },
}
