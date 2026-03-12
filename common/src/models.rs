use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::AsExpression;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Selectable, Insertable, Default)]
#[diesel(table_name = crate::diesel_schema::messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DiscordMessage {
    pub message_id: String,
    pub username: String,
    pub user_id: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow, Default,
)]
#[diesel(sql_type = Text)]
#[serde(rename_all = "lowercase")]
pub enum RecordStatus {
    #[default]
    Pending,
    Scored,
}

impl ToSql<Text, Pg> for RecordStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            RecordStatus::Pending => out.write_all(b"pending")?,
            RecordStatus::Scored => out.write_all(b"scored")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Pg> for RecordStatus {
    fn from_sql(bytes: diesel::pg::PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"pending" => Ok(RecordStatus::Pending),
            b"scored" => Ok(RecordStatus::Scored),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Queryable,
    Selectable,
    Insertable,
    AsChangeset,
    QueryableByName,
    Default,
)]
#[diesel(table_name = crate::diesel_schema::vestibule_users)]
pub struct PersonalityTraits {
    #[serde(alias = "honesty-humility")]
    pub honesty_humility: f64,
    pub emotionality: f64,
    pub extraversion: f64,
    pub agreeableness: f64,
    pub conscientiousness: f64,
    #[serde(alias = "openness-to-experience")]
    pub openness_to_experience: f64,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Queryable,
    Selectable,
    Insertable,
    AsChangeset,
    QueryableByName,
    Default,
)]
#[diesel(table_name = crate::diesel_schema::vestibule_users)]
pub struct CommunicationTraits {
    pub agency: f64,
    pub communion: f64,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Queryable,
    Selectable,
    Insertable,
    AsChangeset,
    QueryableByName,
    Default,
)]
#[diesel(table_name = crate::diesel_schema::vestibule_users)]
pub struct Values {
    #[serde(alias = "self-direction")]
    pub self_direction: f64,
    pub stimulation: f64,
    pub hedonism: f64,
    pub achievement: f64,
    pub power: f64,
    pub security: f64,
    pub conformity: f64,
    pub tradition: f64,
    pub benevolence: f64,
    pub universalism: f64,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Queryable,
    Selectable,
    Insertable,
    AsChangeset,
    QueryableByName,
    Default,
)]
#[diesel(table_name = crate::diesel_schema::vestibule_users)]
pub struct Interests {
    #[serde(alias = "domains")]
    pub domains: Vec<String>,
    #[serde(alias = "activities")]
    pub activities: Vec<String>,
}

#[derive(
    Debug, Clone, Queryable, QueryableByName, Selectable, Insertable, AsChangeset, Default,
)]
#[diesel(table_name = crate::diesel_schema::vestibule_users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VestibuleUserRecord {
    pub discord_user_id: String,
    pub discord_username: String,
    pub yt_username: Option<String>,
    pub yt_display_name: Option<String>,
    pub intro_message_id: Option<String>,

    #[diesel(embed)]
    pub personality: PersonalityTraits,
    #[diesel(embed)]
    pub communication: CommunicationTraits,
    #[diesel(embed)]
    pub values: Values,
    #[diesel(embed)]
    pub interests: Interests,

    pub intro_embedding: Option<pgvector::Vector>,
    pub intro_diagram: Option<Vec<u8>>,

    pub status: RecordStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AiScoreResponse {
    pub username: String,
    pub user_id: String,
    pub personality: PersonalityTraits,
    pub communication: CommunicationTraits,
    pub values: Values,
    pub interests: Interests,
    pub introduction_embedding: Option<Vec<f32>>,
}
