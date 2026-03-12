#![allow(dead_code, unused)]
#![allow(clippy::result_large_err)]
pub use syl_scr_common::models::DiscordMessage;

use thiserror::Error;

pub mod commands;
pub mod listener;
pub mod storage;

pub use storage::AppStorage;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Error: {0}")]
    AppError(Box<dyn std::error::Error + Send + Sync>),

    #[error("Dotenvy error: {0}")]
    DotenvyError(#[from] dotenvy::Error),

    #[error("Tracing error: {0}")]
    TracingError(#[from] tracing::subscriber::SetGlobalDefaultError),

    #[error("There are 0 messages in the store.")]
    NoMessagesError,

    #[error("Serenity error: {0}")]
    SerenityError(#[from] serenity::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),

    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid environment variable: {0}")]
    InvalidEnvVar(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}
