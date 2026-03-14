use std::env;

use diesel::expression_methods::ExpressionMethods;
use diesel::query_dsl::QueryDsl;
use diesel::SelectableHelper;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::RunQueryDsl;
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
        // See how long a function takes
        // .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
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

    tracing::info!("Starting background polling loop for 'Pending' users...");

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

    loop {
        interval.tick().await;

        let mut conn = match pool.get().await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to get database connection from pool: {}", e);
                continue;
            }
        };

        let pending_users: Vec<(VestibuleUserRecord, DiscordMessage)> = vestibule_users::table
            .inner_join(syl_scr_common::diesel_schema::messages::table)
            .filter(vestibule_users::status.eq(RecordStatus::Pending))
            .select((
                VestibuleUserRecord::as_select(),
                DiscordMessage::as_select(),
            ))
            .load(&mut conn)
            .await
            .unwrap_or_else(|e| {
                tracing::error!("Failed to fetch pending users: {}", e);
                vec![]
            });

        for (user_record, discord_msg) in pending_users {
            let discord_user_id = user_record.discord_user_id.clone();
            tracing::info!("Processing pending user: {}", discord_user_id);

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
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to score user {}: {}", discord_user_id, e);
                }
            }
        }
    }
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
