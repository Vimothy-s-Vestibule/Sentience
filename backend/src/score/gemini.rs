use crate::{
    embed::gemini::GeminiError,
    params::{build_gemini_json_schema, build_user_prompt},
    score::MessageScorer,
    AppError, GeminiRespErrors,
};
use serde_json::json;
use syl_scr_common::models::AiScoreResponse;
use tracing::{info, instrument};

pub struct GeminiMessageScorer {
    api_key: String,
}

impl GeminiMessageScorer {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl MessageScorer for GeminiMessageScorer {
    #[instrument(skip_all, fields(user_id = user_id, content = %crate::truncate_chars(content, 10), resp_status = tracing::field::Empty))]
    async fn score_message(
        &self,
        client: &reqwest::Client,
        model: &str,
        user_id: &str,
        content: &str,
    ) -> Result<crate::VestibuleUserRecord, AppError> {
        let cleaned = content.replace('\n', "");
        let content = cleaned.trim();

        if content.is_empty() {
            return Err(AppError::GeminiError(GeminiError::EmptyText));
        }

        let schema = build_gemini_json_schema();
        let user_prompt = build_user_prompt(content, user_id);

        let body = json!({
            "systemInstruction": {
                "parts": [
                    { "text": crate::params::OPENAI_SYSTEM_MSG }
                ]
            },
            "contents": [
                {
                    "role": "user",
                    "parts": [
                        { "text": user_prompt }
                    ]
                }
            ],
            "generationConfig": {
                "responseMimeType": "application/json",
                "responseSchema": schema
            }
        });

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            model
        );

        let resp = client
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::AppError(Box::new(e)))?;

        tracing::Span::current().record("resp_status", resp.status().to_string());

        let resp_text = resp
            .text()
            .await
            .map_err(|e| AppError::AppError(Box::new(e)))?;

        parse_score_from_gemini_response(&resp_text)
    }
}

fn parse_score_from_gemini_response(raw: &str) -> Result<crate::VestibuleUserRecord, AppError> {
    let v: serde_json::Value = serde_json::from_str(raw).map_err(|e| {
        AppError::GeminiRespMalformed(crate::GeminiRespErrors::SerdeJSONError(raw.to_owned(), e))
    })?;

    // Find first candidates[0].content.parts[0].text
    let text = v["candidates"]
        .as_array()
        .and_then(|cands| cands.first())
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
        .and_then(|p| p.first())
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .ok_or(AppError::GeminiRespMalformed(
            crate::GeminiRespErrors::ParseError(raw.to_owned()),
        ))?;

    let parsed: AiScoreResponse = serde_json::from_str(&text).map_err(|e| {
        AppError::GeminiRespMalformed(crate::GeminiRespErrors::SerdeJSONError(raw.to_owned(), e))
    })?;

    Ok(crate::VestibuleUserRecord {
        discord_user_id: parsed.user_id,
        discord_username: parsed.username,
        personality: parsed.personality,
        communication: parsed.communication,
        values: parsed.values,
        interests: parsed.interests,
        status: crate::RecordStatus::Scored,
        ..Default::default()
    })
}
