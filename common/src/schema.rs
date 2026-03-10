use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
#[serde(deny_unknown_fields)]
pub struct DiscordMessage {
    pub username: String,
    pub user_id: String,
    pub content: String,
    #[serde(default)]
    pub message_id: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct User {
    pub username: String,
    pub user_id: String,
    pub personality: Personality,
    pub communication: Communication,
    pub values: Values,
    pub interests: Interests,
    pub introduction_embedding: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Personality {
    #[serde(rename = "honesty-humility")]
    pub honesty_humility: f64,
    pub emotionality: f64,
    pub extraversion: f64,
    pub agreeableness: f64,
    pub conscientiousness: f64,
    #[serde(rename = "openness-to-experience")]
    pub openness_to_experience: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Communication {
    pub agency: f64,
    pub communion: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Values {
    #[serde(rename = "self-direction")]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Interests {
    pub domains: Vec<String>,
    pub activities: Vec<String>,
}
