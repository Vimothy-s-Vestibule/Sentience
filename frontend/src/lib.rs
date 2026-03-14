#![allow(dead_code, unused)]
#![allow(clippy::result_large_err)]
pub use syl_scr_common::models::DiscordMessage;

use thiserror::Error;

pub mod commands;

use diesel::prelude::*;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection, RunQueryDsl};
use std::collections::HashSet;
use syl_scr_common::diesel_schema::{messages, vestibule_users};
use syl_scr_common::models::{RecordStatus, VestibuleUserRecord};

pub async fn insert_introduction_message(
    conn: &mut AsyncPgConnection,
    message: &DiscordMessage,
) -> Result<bool, AppError> {
    diesel::insert_into(messages::table)
        .values(message)
        .on_conflict(messages::message_id)
        .do_nothing()
        .execute(conn)
        .await
        .map_err(AppError::DatabaseError)?;

    let empty_user = VestibuleUserRecord {
        discord_user_id: message.user_id.clone(),
        discord_username: message.username.clone(),
        intro_message_id: Some(message.message_id.clone()),
        status: RecordStatus::Pending,
        ..Default::default()
    };

    diesel::insert_into(vestibule_users::table)
        .values(&empty_user)
        .on_conflict(vestibule_users::discord_user_id)
        .do_update()
        .set(vestibule_users::intro_message_id.eq(Some(message.message_id.clone())))
        .execute(conn)
        .await
        .map_err(AppError::DatabaseError)?;

    Ok(true)
}

pub async fn get_existing_user_ids(
    pool: &Pool<AsyncPgConnection>,
) -> Result<HashSet<String>, AppError> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| AppError::AppError(Box::new(e)))?;

    let user_ids: Vec<String> = vestibule_users::table
        .inner_join(messages::table)
        .select(vestibule_users::discord_user_id)
        .load(&mut conn)
        .await
        .map_err(AppError::DatabaseError)?;

    Ok(user_ids.into_iter().collect())
}

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
