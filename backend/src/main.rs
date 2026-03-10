use std::env;

use syl_scr::AppError;
use syl_scr::DiscordMessage;
use syl_scr::User;
use syl_scr::embed::MessageEmbedder;
use syl_scr::storage;
use syl_scr::storage::AppStorage;
use syl_scr::{input::DiscordMessageStore, score::MessageScorer};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let subscriber = tracing_subscriber::fmt()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true),
        )
        // Log time busy and idle inside function span
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    run()
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
    Ok(())
}
async fn run() -> Result<(), syl_scr::AppError> {
    dotenvy::dotenv()?;

    let mut message_source = syl_scr::input::json_file::JSONFileStore::new("subset.json");
    message_source.init().await?;

    let client = reqwest::Client::new();

    let message_scorer = syl_scr::score::gemini::GeminiMessageScorer::new(
        env::var("GEMINI_API_KEY").map_err(|e| AppError::AppError(Box::new(e)))?,
    );

    let message_embedder = syl_scr::embed::gemini::GeminiMessageEmbedder::new(
        env::var("GEMINI_API_KEY").map_err(|e| AppError::AppError(Box::new(e)))?,
    )?;

    let db_path = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./sylvan.db".to_string());

    let mut pool_store: storage::sqlxlite::SqlxStorage =
        storage::sqlxlite::SqlxStorage::new(db_path);

    let pool = pool_store.init().await?;

    for msg in message_source.all() {
        use syl_scr_common::db_schema::ScoreFlat;

        let score = process_message(&message_scorer, &message_embedder, &client, msg).await?;
        let flattened: ScoreFlat = score.into();
        pool_store.insert_score(&pool, &flattened).await?;
    }

    // Only process one message for testing to save tokens
    #[cfg(debug_assertions)]
    let msg = &message_source.all()[0];
    #[cfg(debug_assertions)]
    let score = process_message(&message_scorer, &message_embedder, &client, msg).await?;

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
) -> Result<User, AppError> {
    let mut score: User = scorer
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

    score.introduction_embedding = Some(embedding);

    Ok(score)
}
