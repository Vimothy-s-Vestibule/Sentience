use serenity::all::{
    ChannelId, Context, CreateCommand, GetMessages, Message, ResolvedOption, RoleId, UserId,
};

use crate::AppError;
use syl_scr_common::models::DiscordMessage;

async fn introduction_message_by_user(
    ctx: &Context,
    channel_id: ChannelId,
    user_id: UserId,
) -> Result<Option<Message>, AppError> {
    let mut before = None;

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

        if let Some(message) = messages.iter().find(|m| m.author.id == user_id) {
            return Ok(Some(message.clone()));
        }

        before = messages.last().map(|message| message.id);
        if messages.len() < 100 {
            break;
        }
    }

    Ok(None)
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

#[tracing::instrument(
    skip_all,
    fields(username = %command_user_id.to_string())
)]
pub async fn run(
    ctx: &Context,
    _options: &[ResolvedOption<'_>],
    guild_id: serenity::model::id::GuildId,
    command_user_id: UserId,
    pool: &diesel_async::pooled_connection::deadpool::Pool<diesel_async::AsyncPgConnection>,
    role_id: RoleId,
    channel_id: ChannelId,
) -> Result<String, AppError> {
    if !has_role(ctx, guild_id, command_user_id, role_id).await? {
        return Err(AppError::PermissionDenied(
            "You do not have the required role.".into(),
        ));
    }

    let existing_user_ids = crate::get_existing_user_ids(pool).await?;

    let members = guild_id
        .members(&ctx.http, Some(1000), None)
        .await
        .map_err(AppError::SerenityError)?;

    let mut scraped_count = 0;
    let mut skipped_count = 0;
    let mut failed_users = Vec::new();

    let mut conn = pool
        .get()
        .await
        .map_err(|e| AppError::AppError(Box::new(e)))?;

    for member in members {
        let user = &member.user;
        let user_id = user.id;
        let username = user.name.clone();

        if existing_user_ids.contains(&user_id.get().to_string()) {
            skipped_count += 1;
            continue;
        }

        let message = match introduction_message_by_user(ctx, channel_id, user_id).await {
            Ok(Some(msg)) => msg,
            Ok(None) => {
                failed_users.push(format!("{}: No messages", username));
                continue;
            }
            Err(e) => {
                failed_users.push(format!("{}: API error", username));
                tracing::warn!("Failed to fetch messages for {}: {}", username, e);
                continue;
            }
        };

        let discord_msg = DiscordMessage {
            username: username.clone(),
            user_id: user_id.get().to_string(),
            content: message.content.clone(),
            message_id: message.id.get().to_string(),
            created_at: *message.timestamp,
        };

        if let Err(e) = crate::insert_introduction_message(&mut conn, &discord_msg).await {
            failed_users.push(format!("{}: DB error", username));
            tracing::error!("Failed to store message for {}: {}", username, e);
            continue;
        }

        scraped_count += 1;
    }

    Ok(format!(
        "Scraped {} new messages.\nSkipped (already in db): {}\nFailed: {:#?}",
        scraped_count, skipped_count, failed_users
    ))
}

pub fn register() -> CreateCommand {
    CreateCommand::new("scrape_intros").description(
        "Scrapes introduction messages from all users in the intro channel (requires SCRAPER_ROLE_ID)",
    )
}
