use serde::{Deserialize, Serialize};

use crate::schema::{Communication, Interests, Personality, User, Values};

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
#[serde(deny_unknown_fields)]
pub struct ScoreFlat {
    pub username: String,
    pub user_id: String,

    // Personality (HEXACO)
    #[serde(rename = "honesty-humility")]
    pub honesty_humility: f64,
    pub emotionality: f64,
    pub extraversion: f64,
    pub agreeableness: f64,
    pub conscientiousness: f64,
    #[serde(rename = "openness-to-experience")]
    pub openness_to_experience: f64,

    // Communication
    pub agency: f64,
    pub communion: f64,

    // Values (Schwartz)
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

    // Interests
    // You can't store Vec<String> as multiple columns unless you normalize.
    // Recommended: store as JSON TEXT, or normalize to join tables.
    pub interest_domains: Vec<String>,
    pub interest_activities: Vec<String>,

    // Vector
    pub introduction_embedding: Option<Vec<f32>>,
}

impl From<User> for ScoreFlat {
    fn from(score: User) -> Self {
        let User {
            username,
            user_id,
            personality,
            communication,
            values,
            interests,
            introduction_embedding,
        } = score;

        let Personality {
            honesty_humility,
            emotionality,
            extraversion,
            agreeableness,
            conscientiousness,
            openness_to_experience,
        } = personality;

        let Communication { agency, communion } = communication;

        let Values {
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
        } = values;

        let Interests {
            domains: interest_domains,
            activities: interest_activities,
        } = interests;

        Self {
            username,
            user_id,
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
            interest_domains,
            interest_activities,
            introduction_embedding,
        }
    }
}

impl From<ScoreFlat> for User {
    fn from(score: ScoreFlat) -> Self {
        let ScoreFlat {
            username,
            user_id,
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
            interest_domains,
            interest_activities,
            introduction_embedding,
        } = score;

        Self {
            username,
            user_id,
            personality: Personality {
                honesty_humility,
                emotionality,
                extraversion,
                agreeableness,
                conscientiousness,
                openness_to_experience,
            },
            communication: Communication { agency, communion },
            values: Values {
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
            },
            interests: Interests {
                domains: interest_domains,
                activities: interest_activities,
            },
            introduction_embedding,
        }
    }
}
