use diesel::sql_query;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use futures::stream::StreamExt;
use serenity::http::Http;
use std::sync::Arc;

#[tracing::instrument]
pub fn spawn_notification_listener(db_url: String, http: Arc<Http>) {
    tokio::spawn(async move {
        let mut listen_conn = match AsyncPgConnection::establish(&db_url).await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to connect to Postgres for LISTEN: {}", e);
                return;
            }
        };

        if let Err(e) = sql_query("LISTEN score_complete")
            .execute(&mut listen_conn)
            .await
        {
            tracing::error!("Failed to execute LISTEN: {}", e);
            return;
        }

        tracing::info!("Listening for 'score_complete' notifications...");

        let notifications = listen_conn.notifications_stream();
        tokio::pin!(notifications);

        while let Some(notification_result) = notifications.next().await {
            match notification_result {
                Ok(notification) => {
                    let user_id_str = notification.payload.as_str();
                    tracing::info!(
                        "Frontend received 'score_complete' for user: {}",
                        user_id_str
                    );

                    if let Ok(uid) = user_id_str.parse::<u64>() {
                        tracing::info!(
                            "Got message from backend that scoring is complete for id {}",
                            uid
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Postgres notification error: {}", e);
                }
            }
        }
    });
}
