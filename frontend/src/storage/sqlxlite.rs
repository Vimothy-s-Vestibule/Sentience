use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::collections::HashSet;
use std::time::Duration;

use syl_scr_common::schema::DiscordMessage;

use super::AppStorage;

pub struct SqlxStorage {
    pool: SqlitePool,
}

impl SqlxStorage {
    pub async fn new(path: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .connect(path)
            .await?;

        let _ = sqlx::query(include_str!(
            "../../../common/db/introduction_messages_table.sql"
        ))
        .execute(&pool)
        .await;

        Ok(Self { pool })
    }
}

impl AppStorage for SqlxStorage {
    async fn insert_introduction_message(
        &self,
        message: &DiscordMessage,
    ) -> Result<bool, sqlx::Error> {
        let exists: Option<String> =
            sqlx::query_scalar("SELECT user_id FROM introduction_messages WHERE user_id = ?")
                .bind(&message.user_id)
                .fetch_optional(&self.pool)
                .await?;

        if exists.is_some() {
            return Ok(false);
        }

        sqlx::query(
            "INSERT INTO introduction_messages (user_id, username, message_id, content) VALUES (?, ?, ?, ?)",
        )
        .bind(&message.user_id)
        .bind(&message.username)
        .bind(message.message_id)
        .bind(&message.content)
        .execute(&self.pool)
        .await?;

        Ok(true)
    }

    async fn get_all_introduction_messages(&self) -> Result<Vec<DiscordMessage>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DiscordMessage>(
            "SELECT username, user_id, content, message_id FROM introduction_messages",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn get_existing_user_ids(&self) -> Result<HashSet<String>, sqlx::Error> {
        let rows = sqlx::query_scalar::<_, String>("SELECT user_id FROM introduction_messages")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().collect())
    }
}
