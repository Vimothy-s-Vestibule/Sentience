use std::env;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use diesel::expression_methods::ExpressionMethods;
use diesel::query_dsl::QueryDsl;
use diesel::SelectableHelper;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::RunQueryDsl;
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use syl_scr::embed::MessageEmbedder;
use syl_scr::score::MessageScorer;
use syl_scr::{AppError, DiscordMessage, RecordStatus, VestibuleUserRecord};

use syl_scr_common::diesel_schema::vestibule_users;

mod tui;
use tui::{draw_ui, App, AppEvent};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Initialize tui-logger instead of standard tracing-subscriber
    tui_logger::init_logger(log::LevelFilter::Info).unwrap_or(());
    tui_logger::set_default_level(log::LevelFilter::Info);

    // Only initialize tracing-subscriber if it hasn't been already
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    dotenvy::dotenv().ok();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, mut rx) = mpsc::unbounded_channel();

    // Spawn polling worker thread
    let tx_worker = tx.clone();
    tokio::spawn(async move {
        let _ = tx_worker.send(AppEvent::DbConnecting);
        if let Err(e) = run_worker(tx_worker.clone()).await {
            tracing::error!("Worker thread failed: {:?}", e);
            let _ = tx_worker.send(AppEvent::DbError(e.to_string()));
        }
    });

    // Spawn input thread
    let tx_input = tx.clone();
    tokio::spawn(async move {
        let tick_rate = Duration::from_millis(200);
        loop {
            if event::poll(tick_rate).unwrap_or(false) {
                if let Ok(c_event) = event::read() {
                    match c_event {
                        CEvent::Key(key) => {
                            let _ = tx_input.send(AppEvent::Input(key.code));
                        }
                        CEvent::Mouse(mouse) => {
                            let _ = tx_input.send(AppEvent::Mouse(mouse));
                        }
                        _ => {}
                    }
                }
            } else {
                let _ = tx_input.send(AppEvent::Tick);
            }
        }
    });

    let mut app = App::new();
    tokio::fs::write("/etc/ready", "1").await?;
    // Main TUI loop
    loop {
        terminal.draw(|f| draw_ui(f, &mut app))?;

        if let Some(event) = rx.recv().await {
            app.handle_event(event);
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

async fn run_worker(tx: mpsc::UnboundedSender<AppEvent>) -> Result<(), syl_scr::AppError> {
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

    tracing::info!("Starting background database operations...");

    // Initial Load
    {
        let mut conn = pool
            .get()
            .await
            .map_err(|e| AppError::AppError(Box::new(e)))?;
        let all_users: Vec<(VestibuleUserRecord, DiscordMessage)> = vestibule_users::table
            .inner_join(syl_scr_common::diesel_schema::messages::table)
            .select((
                VestibuleUserRecord::as_select(),
                DiscordMessage::as_select(),
            ))
            .load(&mut conn)
            .await
            .unwrap_or_else(|e| {
                tracing::error!("Failed to fetch initial users: {}", e);
                vec![]
            });
        let _ = tx.send(AppEvent::Init(all_users));
    }

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

    loop {
        interval.tick().await;

        let mut conn = match pool.get().await {
            Ok(c) => {
                let _ = tx.send(AppEvent::DbConnected);
                c
            }
            Err(e) => {
                tracing::error!("Failed to get database connection from pool: {}", e);
                let _ = tx.send(AppEvent::DbError(e.to_string()));
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

        if !pending_users.is_empty() {
            let _ = tx.send(AppEvent::NewPending(pending_users.clone()));
        }

        let mut processed_any = false;

        for (user_record, discord_msg) in pending_users {
            let discord_user_id = user_record.discord_user_id.clone();
            tracing::info!("Processing pending user: {}", discord_user_id);
            let _ = tx.send(AppEvent::Processing(discord_user_id.clone()));

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
                    score.intro_message_id = user_record.intro_message_id.clone();

                    if let Err(e) = diesel::update(vestibule_users::table)
                        .filter(vestibule_users::discord_user_id.eq(&discord_user_id))
                        .set(&score)
                        .execute(&mut conn)
                        .await
                    {
                        tracing::error!("Failed to save score for user {}: {}", discord_user_id, e);
                    } else {
                        tracing::info!("Successfully scored user {}", discord_user_id);
                        let _ = tx.send(AppEvent::Scored(score, discord_msg));
                        processed_any = true;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to score user {}: {}", discord_user_id, e);
                }
            }
        }

        if processed_any {
            tracing::info!("All queued users have been scored and embedded successfully.");
        }
    }
}

#[tracing::instrument(
    skip_all,
    fields(user_id = %msg.user_id, username = %msg.user_id)
)]
async fn process_message(
    scorer: &impl MessageScorer,
    embedder: &impl MessageEmbedder,
    client: &reqwest::Client,
    msg: &DiscordMessage,
) -> Result<VestibuleUserRecord, AppError> {
    let mut score: VestibuleUserRecord = scorer
        .score_message(client, "gemini-2.5-flash", &msg.user_id, &msg.content)
        .await?;

    let embedding = embedder
        .embed_text(&msg.content, client, &msg.user_id)
        .await?;

    score.intro_embedding = Some(pgvector::Vector::from(embedding));

    let score_clone = score.clone();
    let diagram_bytes = tokio::task::spawn_blocking(move || {
        syl_scr::diagram::generate_personality_chart(&score_clone)
    })
    .await
    .map_err(|e| AppError::AppError(Box::new(e)))?
    .map_err(AppError::AppError)?;

    score.intro_diagram = Some(diagram_bytes);
    score.intro_message_id = Some(msg.message_id.clone());

    Ok(score)
}
