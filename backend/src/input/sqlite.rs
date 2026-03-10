use crate::{AppError, DiscordMessage, input::DiscordMessageStore};
use sqlx::sqlite::SqlitePoolOptions;
use std::time::Duration;
use tracing::instrument;

#[derive(Debug)]
pub struct SqliteMessageStore {
    path: String,
    messages: Vec<DiscordMessage>,
}

impl SqliteMessageStore {
    pub fn new(path: String) -> Self {
        Self {
            path,
            messages: Vec::new(),
        }
    }
}

impl DiscordMessageStore for SqliteMessageStore {
    #[instrument(skip_all, fields(path = %self.path, n = tracing::field::Empty))]
    async fn init(&mut self) -> Result<(), AppError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&self.path)
            .await?;

        // Fetch all messages
        self.messages = sqlx::query_as::<_, DiscordMessage>(
            "SELECT username, user_id, content, message_id FROM introduction_messages",
        )
        .fetch_all(&pool)
        .await?;

        tracing::Span::current().record("n", self.messages.len());

        if self.messages.is_empty() {
            return Err(AppError::NoMessagesError);
        }

        Ok(())
    }

    fn all(&self) -> &Vec<DiscordMessage> {
        &self.messages
    }
}
