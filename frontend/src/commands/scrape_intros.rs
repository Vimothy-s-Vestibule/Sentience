use std::env;

use serenity::all::{
    ChannelId, Context, CreateCommand, GetMessages, Message, ResolvedOption, RoleId, UserId,
};

use crate::AppError;
use crate::AppStorage;
use crate::storage::sqlxlite::SqlxStorage;
use syl_scr_common::schema::DiscordMessage;

#[allow(clippy::result_large_err)]
fn get_env_database_path() -> Result<String, AppError> {
    env::var("DATABASE_PATH").map_err(|_| AppError::MissingEnvVar("DATABASE_PATH".into()))
}

async fn introduction_message_by_user(
    ctx: &Context,
    channel_id: ChannelId,
    user_id: UserId,
) -> Result<Option<Message>, AppError> {
    let mut before = None;
    let mut intro_found = None;

    loop {
        let mut request = GetMessages::new().limit(100);
        if let Some(message_id) = before {
            request = request.before(message_id);
        }

        let messages = channel_id
            .messages(&ctx.http, request)
            .await
            .map_err(AppError::SerenityError)?;

        if messages.is_empty() {
            break;
        }

        for message in &messages {
            if message.author.id == user_id {
                intro_found = Some(message.clone());
            }
        }

        before = messages.last().map(|message| message.id);
        if messages.len() < 100 {
            break;
        }
    }

    Ok(intro_found)
}

async fn has_role(
    ctx: &Context,
    guild_id: serenity::model::id::GuildId,
    user_id: UserId,
    role_id: RoleId,
) -> Result<bool, AppError> {
    let member = guild_id
        .member(&ctx.http, user_id)
        .await
        .map_err(AppError::SerenityError)?;
    Ok(member.roles.contains(&role_id))
}

fn get_env_role_id() -> Result<RoleId, AppError> {
    let id = env::var("SCRAPER_ROLE_ID")
        .map_err(|_| AppError::MissingEnvVar("SCRAPER_ROLE_ID".into()))?;
    let id = id
        .parse::<u64>()
        .map_err(|_| AppError::InvalidEnvVar("SCRAPER_ROLE_ID must be a valid u64".into()))?;
    Ok(RoleId::new(id))
}

fn get_env_intro_channel_id() -> Result<ChannelId, AppError> {
    let id = env::var("DISCORD_INTRO_CHANNEL_ID")
        .map_err(|_| AppError::MissingEnvVar("DISCORD_INTRO_CHANNEL_ID".into()))?;
    let id = id.parse::<u64>().map_err(|_| {
        AppError::InvalidEnvVar("DISCORD_INTRO_CHANNEL_ID must be a valid u64".into())
    })?;
    Ok(ChannelId::new(id))
}

pub async fn run(
    ctx: &Context,
    _options: &[ResolvedOption<'_>],
    guild_id: serenity::model::id::GuildId,
    command_user_id: UserId,
) -> Result<String, AppError> {
    let role_id = get_env_role_id()?;
    let channel_id = get_env_intro_channel_id()?;

    if !has_role(ctx, guild_id, command_user_id, role_id).await? {
        return Err(AppError::PermissionDenied(
            "You do not have the required role.".into(),
        ));
    }

    let db_path = get_env_database_path()?;
    let storage = SqlxStorage::new(&db_path)
        .await
        .map_err(AppError::DatabaseError)?;

    let existing_user_ids = storage
        .get_existing_user_ids()
        .await
        .map_err(AppError::DatabaseError)?;

    let members = guild_id
        .members(&ctx.http, Some(1000), None)
        .await
        .map_err(AppError::SerenityError)?;

    let mut scraped_count = 0;
    let mut skipped_count = 0;
    let mut failed_users = Vec::new();

    for member in members {
        let user = &member.user;
        let user_id = user.id;
        let user_id_str = user_id.get().to_string();
        let username = user.name.clone();

        if existing_user_ids.contains(&user_id_str) {
            skipped_count += 1;
            continue;
        }

        match introduction_message_by_user(ctx, channel_id, user_id).await {
            Ok(Some(message)) => {
                let discord_msg = DiscordMessage {
                    username: username.clone(),
                    user_id: user_id.get().to_string(),
                    content: message.content.clone(),
                    message_id: message.id.get() as i64,
                };

                if let Err(e) = storage.insert_introduction_message(&discord_msg).await {
                    failed_users.push(format!("{}: DB error", username));
                    tracing::error!("Failed to store message for {}: {}", username, e);
                } else {
                    scraped_count += 1;
                }
            }
            Ok(None) => {
                failed_users.push(format!("{}: No messages", username));
            }
            Err(e) => {
                failed_users.push(format!("{}: API error", username));
                tracing::warn!("Failed to fetch messages for {}: {}", username, e);
            }
        }
    }

    Ok(format!(
        "Scraped {} new messages.\nSkipped (already have message): {}\nFailed: {}",
        scraped_count,
        skipped_count,
        failed_users.len()
    ))
}

pub fn register() -> CreateCommand {
    CreateCommand::new("scrape_intros").description(
        "Scrapes introduction messages from all users in the intro channel (requires SCRAPER_ROLE_ID)",
    )
}
