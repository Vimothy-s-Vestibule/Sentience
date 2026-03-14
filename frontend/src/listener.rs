use diesel::sql_query;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use futures::stream::StreamExt;
use serenity::http::Http;
use std::sync::Arc;

use crate::AppError;

#[tracing::instrument(skip_all, fields(user_id = tracing::field::Empty))]
pub async fn spawn_notification_listener(db_url: String, http: Arc<Http>) -> Result<(), AppError> {
    let mut listen_conn = AsyncPgConnection::establish(&db_url)
        .await
        .map_err(|e| {
            AppError::AppError(format!("Failed to connect to Postgres for LISTEN: {}", e).into())
        })?;

    sql_query("LISTEN score_complete")
        .execute(&mut listen_conn)
        .await
        .map_err(|e| {
            AppError::AppError(format!("Failed to execute LISTEN: {}", e).into())
        })?;

    tracing::info!("Listening for 'score_complete' notifications...");

    let notifications = listen_conn.notifications_stream();
    tokio::pin!(notifications);

    while let Some(notification_result) = notifications.next().await {
        let notification = notification_result.map_err(|e| {
            AppError::AppError(format!("Postgres notification error: {}", e).into())
        })?;

        let user_id_str = notification.payload.as_str();
        let uid = match user_id_str.parse::<u64>() {
            Ok(id) => id,
            Err(_) => continue,
        };

        tracing::Span::current().record("user_id", uid);
        tracing::info!("notification recieved: intro scoring completed");
    }
    Ok(())
}
