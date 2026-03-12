use std::env;

use diesel::expression_methods::ExpressionMethods;
use diesel::query_dsl::QueryDsl;
use diesel::sql_query;
use diesel::OptionalExtension;
use diesel::SelectableHelper;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::RunQueryDsl;
use futures::stream::StreamExt;
use syl_scr::embed::MessageEmbedder;
use syl_scr::score::MessageScorer;
use syl_scr::AppError;
use syl_scr::DiscordMessage;
use syl_scr::RecordStatus;
use syl_scr::VestibuleUserRecord;
use syl_scr_common::diesel_schema::vestibule_users;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let subscriber = tracing_subscriber::fmt()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true),
        )
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    run()
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
    Ok(())
}

async fn run() -> Result<(), syl_scr::AppError> {
    dotenvy::dotenv().ok();

    let client_http = reqwest::Client::new();

    let message_scorer = syl_scr::score::gemini::GeminiMessageScorer::new(
        env::var("GEMINI_API_KEY").map_err(|e| AppError::AppError(Box::new(e)))?,
    );

    let message_embedder = syl_scr::embed::gemini::GeminiMessageEmbedder::new(
        env::var("GEMINI_API_KEY").map_err(|e| AppError::AppError(Box::new(e)))?,
    )?;

    let database_url = env::var("DATABASE_URL").map_err(|e| AppError::AppError(Box::new(e)))?;
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    let pool = Pool::builder(config)
        .build()
        .map_err(|e| AppError::AppError(Box::new(e)))?;

    let mut listen_conn = pool
        .get()
        .await
        .map_err(|e| AppError::AppError(Box::new(e)))?;

    // Register for notifications
    diesel::sql_query("LISTEN process_user")
        .execute(&mut listen_conn)
        .await
        .map_err(|e| AppError::AppError(Box::new(e)))?;

    // Create the notifications stream - must be pinned
    let notifications = listen_conn.notifications_stream();

    tracing::info!("Listening for 'process_user' notifications...");

    // Main loop: process notifications as they arrive
    tokio::pin!(notifications);

    while let Some(notification_result) = notifications.next().await {
        let notification = notification_result.map_err(|e| AppError::AppError(Box::new(e)))?;

        let discord_user_id = notification.payload.as_str();
        tracing::info!("Received request to process user: {}", discord_user_id);

        let mut conn = pool
            .get()
            .await
            .map_err(|e| AppError::AppError(Box::new(e)))?;

        let result: Option<(VestibuleUserRecord, DiscordMessage)> = vestibule_users::table
            .inner_join(syl_scr_common::diesel_schema::messages::table)
            .filter(vestibule_users::discord_user_id.eq(&discord_user_id))
            .select((
                VestibuleUserRecord::as_select(),
                DiscordMessage::as_select(),
            ))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::AppError(Box::new(e)))?;

        if let Some((user_record, discord_msg)) = result {
            if user_record.status == RecordStatus::Scored {
                tracing::info!("User {} already scored, skipping.", discord_user_id);
                continue;
            }

            match process_message(
                &message_scorer,
                &message_embedder,
                &client_http,
                &discord_msg,
            )
            .await
            {
                Ok(mut score) => {
                    score.status = RecordStatus::Scored;
                    score.intro_message_id = user_record.intro_message_id;

                    if let Err(e) = diesel::update(vestibule_users::table)
                        .filter(vestibule_users::discord_user_id.eq(&discord_user_id))
                        .set(&score)
                        .execute(&mut conn)
                        .await
                    {
                        tracing::error!("Failed to save score for user {}: {}", discord_user_id, e);
                    } else {
                        tracing::info!("Successfully scored user {}", discord_user_id);

                        let notify_query = format!("NOTIFY score_complete, '{}'", discord_user_id);
                        if let Err(e) = sql_query(notify_query).execute(&mut conn).await {
                            tracing::error!(
                                "Failed to notify frontend for user {}: {}",
                                discord_user_id,
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to score user {}: {}", discord_user_id, e);
                }
            }
        } else {
            tracing::warn!(
                "Received notification for {}, but record not found",
                discord_user_id
            );
        }
    }

    Ok(())
}

#[tracing::instrument(
    skip_all,
    fields(user_id = %msg.user_id, username = %msg.username)
)]
async fn process_message(
    scorer: &impl MessageScorer,
    embedder: &impl MessageEmbedder,
    client: &reqwest::Client,
    msg: &DiscordMessage,
) -> Result<VestibuleUserRecord, AppError> {
    let mut score: VestibuleUserRecord = scorer
        .score_message(
            client,
            "gemini-2.5-flash",
            &msg.username,
            &msg.user_id,
            &msg.content,
        )
        .await?;

    let embedding = embedder
        .embed_text(&msg.content, client, &msg.username)
        .await?;

    score.intro_embedding = Some(pgvector::Vector::from(embedding));

    let diagram_bytes =
        syl_scr::diagram::generate_personality_chart(&score).map_err(AppError::AppError)?;

    score.intro_diagram = Some(diagram_bytes);
    score.intro_message_id = Some(msg.message_id.clone());

    Ok(score)
}
