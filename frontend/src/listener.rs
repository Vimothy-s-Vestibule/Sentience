use diesel::sql_query;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use futures::stream::StreamExt;
use serenity::http::Http;
use std::sync::Arc;

use crate::AppError;

#[tracing::instrument(skip_all, fields(user_id = tracing::field::Empty))]
pub async fn spawn_notification_listener(db_url: String, http: Arc<Http>) -> Result<(), AppError> {
    let mut listen_conn = match AsyncPgConnection::establish(&db_url).await {
        Ok(c) => c,
        Err(e) => {
            return Err(AppError::AppError(
                format!("Failed to connect to Postgres for LISTEN: {}", e).into(),
            ));
        }
    };

    if let Err(e) = sql_query("LISTEN score_complete")
        .execute(&mut listen_conn)
        .await
    {
        return Err(AppError::AppError(
            format!("Failed to execute LISTEN: {}", e).into(),
        ));
    }

    tracing::info!("Listening for 'score_complete' notifications...");

    let notifications = listen_conn.notifications_stream();
    tokio::pin!(notifications);

    while let Some(notification_result) = notifications.next().await {
        match notification_result {
            Ok(notification) => {
                let user_id_str = notification.payload.as_str();
                if let Ok(uid) = user_id_str.parse::<u64>() {
                    tracing::Span::current().record("user_id", uid);
                    tracing::info!("notification recieved: intro scoring completed");
                }
            }
            Err(e) => {
                return Err(AppError::AppError(
                    format!("Postgres notification error: {}", e).into(),
                ));
            }
        }
    }
    Ok(())
}
