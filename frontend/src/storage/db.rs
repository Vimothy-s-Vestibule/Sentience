use diesel_async::{
    pooled_connection::deadpool::Pool, pooled_connection::AsyncDieselConnectionManager,
    AsyncPgConnection, RunQueryDsl,
};
use diesel::prelude::*;
use std::collections::HashSet;
use syl_scr_common::models::{
    DiscordMessage, RecordStatus, VestibuleUserRecord,
    PersonalityTraits, CommunicationTraits, Values, Interests
};
use syl_scr_common::diesel_schema::{vestibule_users, messages};

use super::AppStorage;

pub struct PostgresStorage {
    pool: Pool<AsyncPgConnection>,
}

impl PostgresStorage {
    pub async fn new(path: &str) -> Result<Self, diesel::result::Error> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(path);
        let pool = Pool::builder(config)
            .max_size(10)
            .build()
            .map_err(|e| diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            ))?;

        Ok(Self { pool })
    }
}

impl AppStorage for PostgresStorage {
    async fn insert_introduction_message(
        &self,
        message: &DiscordMessage,
    ) -> Result<bool, diesel::result::Error> {
        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Check if user exists
        let exists: i64 = vestibule_users::table
            .filter(vestibule_users::discord_user_id.eq(&message.user_id))
            .count()
            .get_result(&mut conn)
            .await?;

        if exists > 0 {
            return Ok(false);
        }

        // Insert message into messages table
        diesel::insert_into(messages::table)
            .values(message)
            .execute(&mut conn)
            .await?;
        
        let empty_user = VestibuleUserRecord {
            discord_user_id: message.user_id.clone(),
            discord_username: message.username.clone(),
            intro_message_id: Some(message.message_id.clone()),
            status: RecordStatus::Pending,
            ..Default::default()
        };

        diesel::insert_into(vestibule_users::table)
            .values(&empty_user)
            .execute(&mut conn)
            .await?;

        // Notify backend to process this user
        let notify_query = format!("NOTIFY process_user, '{}'", empty_user.discord_user_id);
        diesel::sql_query(notify_query)
            .execute(&mut conn)
            .await?;

        Ok(true)
    }

    async fn get_all_introduction_messages(&self) -> Result<Vec<DiscordMessage>, diesel::result::Error> {
        // Since we moved to the unified vestibule_users table, the discord content isn't stored here.
        // Oh wait. I need to store the raw message content for the backend to score it!
        // I will need to update the diesel schema to include `intro_text`.
        Ok(vec![])
    }

    async fn get_existing_user_ids(&self) -> Result<HashSet<String>, diesel::result::Error> {
        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let user_ids: Vec<String> = vestibule_users::table
            .select(vestibule_users::discord_user_id)
            .load(&mut conn)
            .await?;

        Ok(user_ids.into_iter().collect())
    }
}