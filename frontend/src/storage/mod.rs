#![allow(async_fn_in_trait)]

pub mod postgres;

use std::collections::HashSet;

use syl_scr_common::models::DiscordMessage;

pub trait AppStorage {
    async fn insert_introduction_message(
        &self,
        message: &DiscordMessage,
    ) -> Result<bool, diesel::result::Error>;
    async fn get_all_introduction_messages(
        &self,
    ) -> Result<Vec<DiscordMessage>, diesel::result::Error>;
    async fn get_existing_user_ids(&self) -> Result<HashSet<String>, diesel::result::Error>;
}
