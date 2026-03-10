#![allow(async_fn_in_trait)]

pub mod sqlxlite;

use std::collections::HashSet;

use syl_scr_common::schema::DiscordMessage;

pub trait AppStorage {
    async fn insert_introduction_message(
        &self,
        message: &DiscordMessage,
    ) -> Result<bool, sqlx::Error>;
    async fn get_all_introduction_messages(&self) -> Result<Vec<DiscordMessage>, sqlx::Error>;
    async fn get_existing_user_ids(&self) -> Result<HashSet<String>, sqlx::Error>;
}
