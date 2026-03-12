use std::env;

use syl_scr_bot::{AppError, commands};

#[tokio::main]
#[allow(clippy::result_large_err)]
async fn main() -> Result<(), AppError> {
    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("syl_scr_bot", Level::INFO)
        .with_target("serenity", LevelFilter::OFF);

    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(
                    tracing_subscriber::fmt::format()
                        .with_file(true)
                        .with_line_number(true),
                )
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE),
        )
        .with(filter);

    tracing::subscriber::set_global_default(subscriber).map_err(AppError::TracingError)?;

    dotenvy::dotenv().map_err(AppError::DotenvyError)?;

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").map_err(|e| AppError::AppError(Box::new(e)))?;
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Spawn LISTEN task for 'score_complete' using Diesel's native notifications API
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://sylvan:sylvanpassword@localhost/sylvan_db".to_string());
    let http = client.http.clone();
    
    tokio::spawn(async move {
        use diesel_async::AsyncConnection;
        use diesel_async::AsyncPgConnection;
        use futures::stream::StreamExt;
        use diesel::sql_query;
        use diesel_async::RunQueryDsl;
        
        let mut listen_conn = match AsyncPgConnection::establish(&db_url).await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to connect to Postgres for LISTEN: {}", e);
                return;
            }
        };

        // Register for notifications
        if let Err(e) = sql_query("LISTEN score_complete").execute(&mut listen_conn).await {
            tracing::error!("Failed to execute LISTEN: {}", e);
            return;
        }

        tracing::info!("Listening for 'score_complete' notifications...");

        // Create the notifications stream - must be pinned
        let notifications = listen_conn.notifications_stream();
        tokio::pin!(notifications);

        while let Some(notification_result) = notifications.next().await {
            match notification_result {
                Ok(notification) => {
                    let user_id_str = notification.payload.as_str();
                    tracing::info!("Frontend received 'score_complete' for user: {}", user_id_str);
                    
                    if let Ok(uid) = user_id_str.parse::<u64>() {
                        let user_id = serenity::model::id::UserId::new(uid);
                        // We can notify the user that their score is ready via DM
                        if let Ok(dm_channel) = user_id.create_dm_channel(&http).await {
                            let _ = dm_channel.say(&http, "Your personality profile is ready! The backend has successfully processed your introduction.").await;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Postgres notification error: {}", e);
                    break;
                }
            }
        }
    });

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        tracing::error!("Client error: {}", why);
    }

    Ok(())
}

use serenity::async_trait;
use serenity::builder::{
    CreateInteractionResponse, CreateInteractionResponseMessage, EditInteractionResponse,
};
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use tracing::Level;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            tracing::debug!("Received command: {}", command.data.name);

            let options = command.data.options();
            let guild_id = command.guild_id.unwrap();
            let command_user_id = command.user.id;
            let result: Result<String, AppError> = match command.data.name.as_str() {
                "scrape_intros" => {
                    commands::scrape_intros::run(&ctx, &options, guild_id, command_user_id).await
                }

                _ => Ok("not implemented".to_string()),
            };

            let (content, deferred) = match result {
                Ok(c) => (c, true),
                Err(e) => {
                    tracing::error!("Command '{}' failed: {}", command.data.name, e);
                    (
                        "An error occurred. Please try again later.".to_string(),
                        true,
                    )
                }
            };

            if deferred {
                if let Err(why) = command.defer(&ctx.http).await {
                    tracing::warn!("Cannot defer slash command: {}", why);
                    return;
                }
                if let Err(why) = command
                    .edit_response(&ctx.http, EditInteractionResponse::new().content(content))
                    .await
                {
                    tracing::warn!("Cannot edit deferred response: {}", why);
                }
            } else {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    tracing::warn!("Cannot respond to slash command: {}", why);
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("{} is connected!", ready.user.name);

        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let _commands = guild_id
            .set_commands(
                &ctx.http,
                vec![
                    commands::scrape_intros::register(),
                    // commands::id::register(),
                    // commands::welcome::register(),
                    // commands::numberinput::register(),
                    // commands::attachmentinput::register(),
                    // commands::modal::register(),
                ],
            )
            .await;

        // let global_command =
        //     Command::create_global_command(&ctx.http, commands::wonderful_command::register())
        //         .await;

        // println!("I created the following global slash command: {global_command:#?}");
    }
}
