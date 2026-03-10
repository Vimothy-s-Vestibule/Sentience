#![allow(dead_code, unused)]

pub mod embed;
pub mod input;
pub mod params;
pub mod score;
pub mod storage;

pub use syl_scr_common::schema::{DiscordMessage, User};

use thiserror::Error;

use crate::embed::gemini::GeminiError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Error: {0}")]
    AppError(Box<dyn std::error::Error>),

    #[error("Tracing error: {0}")]
    TracingError(#[from] tracing::subscriber::SetGlobalDefaultError),

    #[error("{0}")]
    OpenAIRespMalformed(OpenAIRespErrors),

    #[error("{0}")]
    GeminiRespMalformed(GeminiRespErrors),

    #[error("{0}")]
    GeminiError(#[from] GeminiError),

    #[error("There are 0 messages in the store.")]
    NoMessagesError,

    #[error("Dotenvy error: {0}")]
    DotenvyError(#[from] dotenvy::Error),

    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    // #[error("libSQL error: {0}")]
    // LibSQLError(#[from] libsql::Error),
}

#[derive(Error, Debug)]
pub enum OpenAIRespErrors {
    #[error("OpenAI response malformed, could not parse JSON. Error: {1}. Response: {0}")]
    SerdeJSONError(String, serde_json::Error),
    #[error("OpenAI response malformed, could not find output text field. Response: {0}")]
    ParseError(String),
}

#[derive(Error, Debug)]
pub enum GeminiRespErrors {
    #[error("Gemini response malformed, could not parse JSON. Error: {1}. Response: {0}")]
    SerdeJSONError(String, serde_json::Error),
    #[error("Gemini response malformed, could not find output text field. Response: {0}")]
    ParseError(String),
}
