use crate::{AppError, storage::AppStorage};
use bytemuck;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::time::Duration;
use syl_scr_common::db_schema::ScoreFlat;

#[derive(Debug, Default)]
pub struct SqlxStorage {
    path: String,
}

impl SqlxStorage {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl AppStorage for SqlxStorage {
    type DBConnectionObject = SqlitePool;

    async fn init(&mut self) -> Result<Self::DBConnectionObject, AppError> {
        let pool: sqlx::Pool<sqlx::Sqlite> = SqlitePoolOptions::new()
            .max_connections(10)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&self.path)
            .await?;

        // Initialize table if it doesn't exist
        let schema = include_str!("../../../common/db/scores_table.sql");
        let query = format!("BEGIN;\n{}\nCOMMIT;", schema);
        sqlx::query(&query).execute(&pool).await?;

        Ok(pool)
    }

    async fn insert_score(
        &self,
        pool: &Self::DBConnectionObject,
        score: &ScoreFlat,
    ) -> Result<(), AppError> {
        let embedding_bytes = score
            .introduction_embedding
            .as_ref()
            .map(|vec| bytemuck::cast_slice::<f32, u8>(vec).to_vec());

        sqlx::query(
            "INSERT INTO scores (
                user_id,
                username,
                honesty_humility,
                emotionality,
                extraversion,
                agreeableness,
                conscientiousness,
                openness_to_experience,
                agency,
                communion,
                self_direction,
                stimulation,
                hedonism,
                achievement,
                power,
                security,
                conformity,
                tradition,
                benevolence,
                universalism,
                introduction_embedding
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(user_id) DO UPDATE SET
                username = excluded.username,
                honesty_humility = excluded.honesty_humility,
                emotionality = excluded.emotionality,
                extraversion = excluded.extraversion,
                agreeableness = excluded.agreeableness,
                conscientiousness = excluded.conscientiousness,
                openness_to_experience = excluded.openness_to_experience,
                agency = excluded.agency,
                communion = excluded.communion,
                self_direction = excluded.self_direction,
                stimulation = excluded.stimulation,
                hedonism = excluded.hedonism,
                achievement = excluded.achievement,
                power = excluded.power,
                security = excluded.security,
                conformity = excluded.conformity,
                tradition = excluded.tradition,
                benevolence = excluded.benevolence,
                universalism = excluded.universalism,
                introduction_embedding = excluded.introduction_embedding,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
        )
        .bind(&score.user_id)
        .bind(&score.username)
        .bind(score.honesty_humility)
        .bind(score.emotionality)
        .bind(score.extraversion)
        .bind(score.agreeableness)
        .bind(score.conscientiousness)
        .bind(score.openness_to_experience)
        .bind(score.agency)
        .bind(score.communion)
        .bind(score.self_direction)
        .bind(score.stimulation)
        .bind(score.hedonism)
        .bind(score.achievement)
        .bind(score.power)
        .bind(score.security)
        .bind(score.conformity)
        .bind(score.tradition)
        .bind(score.benevolence)
        .bind(score.universalism)
        .bind(embedding_bytes)
        .execute(pool)
        .await?;

        Ok(())
    }
}
