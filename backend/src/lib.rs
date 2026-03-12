#![allow(dead_code, unused)]

pub mod diagram;
pub mod embed;
pub mod params;
pub mod score;

pub use syl_scr_common::models::{DiscordMessage, RecordStatus, VestibuleUserRecord};

use thiserror::Error;

use crate::embed::gemini::GeminiError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    AppError(Box<dyn std::error::Error + Send + Sync>),

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

    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),
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
