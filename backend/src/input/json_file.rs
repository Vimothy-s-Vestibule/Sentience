use crate::{AppError, DiscordMessage, input::DiscordMessageStore};
use serde::{Deserialize, Serialize};

use tracing::instrument;

#[derive(Debug)]
pub struct JSONFileStore {
    path: std::path::PathBuf,
    messages: Vec<DiscordMessage>,
}

impl JSONFileStore {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            path: path.into(),
            messages: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct RootJSONItem {
    pub json: DiscordMessage,
}

impl DiscordMessageStore for JSONFileStore {
    #[instrument(skip_all, fields(path = tracing::field::Empty, n = tracing::field::Empty))]
    async fn init(&mut self) -> Result<(), crate::AppError> {
        tracing::Span::current().record("path", self.path.to_str());

        let raw_content = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|e| AppError::AppError(Box::new(e)))?;

        let parsed_content: Vec<RootJSONItem> =
            serde_json::from_str(&raw_content).map_err(|e| AppError::AppError(Box::new(e)))?;

        self.messages = parsed_content.iter().map(|i| i.json.clone()).collect();

        tracing::Span::current().record("n", self.messages.len());

        if self.messages.is_empty() {
            return Err(AppError::NoMessagesError);
        }

        Ok(())
    }

    fn all(&self) -> &std::vec::Vec<DiscordMessage> {
        &self.messages
    }
}
