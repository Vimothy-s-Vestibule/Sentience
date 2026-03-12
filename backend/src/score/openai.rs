use crate::{
    AppError, OpenAIRespErrors,
    params::{build_json_schema, build_user_prompt},
    score::MessageScorer,
};
use syl_scr_common::models::{PersonalityTraits, CommunicationTraits, Values, Interests};
use serde_json::json;
use tracing::{info, instrument};

use crate::score::models::AiScoreResponse;

pub struct OpenAIMessageScorer {
    api_key: String,
}

impl OpenAIMessageScorer {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl MessageScorer for OpenAIMessageScorer {
    #[instrument(skip_all, fields(username = username, content = &content[0..10], resp_status = tracing::field::Empty))]
    async fn score_message(
        &self,
        client: &reqwest::Client,
        model: &str,
        username: &str,
        user_id: &str,
        content: &str,
    ) -> Result<crate::VestibuleUserRecord, AppError> {
        #[cfg(not(debug_assertions))]
        let schema = build_json_schema();
        #[cfg(not(debug_assertions))]
        let user_prompt = build_user_prompt(username, content, user_id);

        #[cfg(not(debug_assertions))]
        let body = json!({
          "model": model,
          "input": [
            { "role": "system", "content": [ { "type": "input_text", "text": crate::params::OPENAI_SYSTEM_MSG } ] },
            { "role": "user",   "content": [ { "type": "input_text", "text": user_prompt } ] }
          ],
          "text": {
            "format": {
              "type": "json_schema",
              "name": "intro_scoring",
              "strict": true,
              "schema": schema
            }
          }
        });

        #[cfg(not(debug_assertions))]
        let resp = client
            .post("https://api.openai.com/v1/responses")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", &self.api_key),
            )
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await
            .unwrap();

        #[cfg(not(debug_assertions))]
        tracing::Span::current().record("resp_status", resp.status().to_string());

        #[cfg(not(debug_assertions))]
        let resp_text = resp
            .text()
            .await
            .map_err(|e| AppError::AppError(Box::new(e)))?;

        // Read use predefined response for dev/testing
        #[cfg(debug_assertions)]
        let resp_text = tokio::fs::read_to_string("sample_resp_openai.json")
            .await
            .map_err(|e| AppError::AppError(Box::new(e)))?;

        parse_score_from_openai_response(&resp_text)
    }
}

fn parse_score_from_openai_response(raw: &str) -> Result<crate::VestibuleUserRecord, AppError> {
    let v: serde_json::Value = serde_json::from_str(raw).map_err(|e| {
        AppError::OpenAIRespMalformed(crate::OpenAIRespErrors::SerdeJSONError(raw.to_owned(), e))
    })?;

    // Find first output_text.text
    // janky but works for now
    let text = v["output"]
        .as_array()
        .and_then(|outs| {
            outs.iter().find_map(|o| {
                let content = o.get("content")?.as_array()?;
                content.iter().find_map(|c| {
                    if c.get("type")?.as_str()? == "output_text" {
                        c.get("text")?.as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
            })
        })
        .ok_or(AppError::OpenAIRespMalformed(
            crate::OpenAIRespErrors::ParseError(raw.to_owned()),
        ))?;

    // There's an extra field `introduction_embedding` in the Score struct, but it's an option, so serde_json allows it to not be present here.
    let parsed: AiScoreResponse = serde_json::from_str(&text).map_err(|e| {
        AppError::OpenAIRespMalformed(crate::OpenAIRespErrors::SerdeJSONError(raw.to_owned(), e))
    })?;

    Ok(crate::VestibuleUserRecord {
        discord_user_id: parsed.user_id,
        discord_username: parsed.username,
        personality: PersonalityTraits {
            honesty_humility: Some(parsed.personality.honesty_humility),
            emotionality: Some(parsed.personality.emotionality),
            extraversion: Some(parsed.personality.extraversion),
            agreeableness: Some(parsed.personality.agreeableness),
            conscientiousness: Some(parsed.personality.conscientiousness),
            openness_to_experience: Some(parsed.personality.openness_to_experience),
        },
        communication: CommunicationTraits {
            agency: Some(parsed.communication.agency),
            communion: Some(parsed.communication.communion),
        },
        values: Values {
            self_direction: Some(parsed.values.self_direction),
            stimulation: Some(parsed.values.stimulation),
            hedonism: Some(parsed.values.hedonism),
            achievement: Some(parsed.values.achievement),
            power: Some(parsed.values.power),
            security: Some(parsed.values.security),
            conformity: Some(parsed.values.conformity),
            tradition: Some(parsed.values.tradition),
            benevolence: Some(parsed.values.benevolence),
            universalism: Some(parsed.values.universalism),
        },
        interests: Interests {
            interest_domains: parsed.interests.domains,
            interest_activities: parsed.interests.activities,
        },

        status: crate::RecordStatus::Scored,
        ..Default::default()
    })
}
