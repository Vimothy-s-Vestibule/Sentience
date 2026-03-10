use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

pub const OPENAI_MODEL: &str = "gpt-5-nano";
pub const OPENAI_SYSTEM_MSG: &str = "You analyze short self-introduction messages for social compatibility research. Goal: Extract probabilistic psychological and communication signals from the introduction text.";

pub fn build_user_prompt(username: &str, introduction_text: &str, user_id: &str) -> String {
    format!(
        "<username>\n{username}\n</username>\n<user_id>\n{user_id}\n</user_id>\n\n<introduction_text>\n{intro}\n</introduction_text>",
        username = username,
        intro = introduction_text
    )
}

pub fn build_gemini_json_schema() -> serde_json::Value {
    json!({
      "type": "OBJECT",
      "properties": {
        "username": { "type": "STRING" },
        "user_id": { "type": "STRING" },
        "personality": {
          "description": "HEXACO personality traits normalized 0–1 where 0.5 ≈ population average percentile.",
          "type": "OBJECT",
          "properties": {
            "honesty-humility": { "type": "NUMBER" },
            "emotionality": { "type": "NUMBER" },
            "extraversion": { "type": "NUMBER" },
            "agreeableness": { "type": "NUMBER" },
            "conscientiousness": { "type": "NUMBER" },
            "openness-to-experience": { "type": "NUMBER" }
          },
          "required": [
            "honesty-humility",
            "emotionality",
            "extraversion",
            "agreeableness",
            "conscientiousness",
            "openness-to-experience"
          ]
        },
        "communication": {
          "description": "Interpersonal circumplex dimensions.",
          "type": "OBJECT",
          "properties": {
            "agency": {
              "type": "NUMBER",
              "description": "Dominance, assertiveness, conversational control."
            },
            "communion": {
              "type": "NUMBER",
              "description": "Warmth, empathy, affiliative orientation."
            }
          },
          "required": ["agency", "communion"]
        },
        "values": {
          "description": "Schwartz basic human values, normalized importance weights.",
          "type": "OBJECT",
          "properties": {
            "self-direction": { "type": "NUMBER" },
            "stimulation": { "type": "NUMBER" },
            "hedonism": { "type": "NUMBER" },
            "achievement": { "type": "NUMBER" },
            "power": { "type": "NUMBER" },
            "security": { "type": "NUMBER" },
            "conformity": { "type": "NUMBER" },
            "tradition": { "type": "NUMBER" },
            "benevolence": { "type": "NUMBER" },
            "universalism": { "type": "NUMBER" }
          },
          "required": [
            "self-direction",
            "stimulation",
            "hedonism",
            "achievement",
            "power",
            "security",
            "conformity",
            "tradition",
            "benevolence",
            "universalism"
          ]
        },
        "interests": {
          "type": "OBJECT",
          "properties": {
            "domains": { "type": "ARRAY", "items": { "type": "STRING", "description": "Lowercase concise term, 1–3 words max. No punctuation or sentences." } },
            "activities": { "type": "ARRAY", "items": { "type": "STRING", "description": "Lowercase concise term, 1–3 words max. No punctuation or sentences." } }
          },
          "required": ["domains", "activities"]
        }
      },
      "required": ["username", "personality", "communication", "values", "interests", "user_id"]
    })
}

pub fn build_json_schema() -> serde_json::Value {
    json!({
      "type": "object",
      "$defs": {
        "conciseLowercaseTerm": {
          "type": "string",
          "minLength": 2,
          "maxLength": 32,
          "pattern": "^[a-z]+(?: [a-z]+){0,2}$",
          "description": "Lowercase concise term, 1–3 words max. No punctuation or sentences."
        }
      },
      "properties": {
        "username": { "type": "string" },
        "user_id": { "type": "string" },
        "personality": {
          "description": "HEXACO personality traits normalized 0–1 where 0.5 ≈ population average percentile.",
          "type": "object",
          "properties": {
            "honesty-humility": { "type": "number", "minimum": 0, "maximum": 1 },
            "emotionality": { "type": "number", "minimum": 0, "maximum": 1 },
            "extraversion": { "type": "number", "minimum": 0, "maximum": 1 },
            "agreeableness": { "type": "number", "minimum": 0, "maximum": 1 },
            "conscientiousness": { "type": "number", "minimum": 0, "maximum": 1 },
            "openness-to-experience": { "type": "number", "minimum": 0, "maximum": 1 }
          },
          "required": [
            "honesty-humility",
            "emotionality",
            "extraversion",
            "agreeableness",
            "conscientiousness",
            "openness-to-experience"
          ],
          "additionalProperties": false
        },
        "communication": {
          "description": "Interpersonal circumplex dimensions.",
          "type": "object",
          "properties": {
            "agency": {
              "type": "number",
              "minimum": 0,
              "maximum": 1,
              "description": "Dominance, assertiveness, conversational control."
            },
            "communion": {
              "type": "number",
              "minimum": 0,
              "maximum": 1,
              "description": "Warmth, empathy, affiliative orientation."
            }
          },
          "required": ["agency", "communion"],
          "additionalProperties": false
        },
        "values": {
          "description": "Schwartz basic human values, normalized importance weights.",
          "type": "object",
          "properties": {
            "self-direction": { "type": "number", "minimum": 0, "maximum": 1 },
            "stimulation": { "type": "number", "minimum": 0, "maximum": 1 },
            "hedonism": { "type": "number", "minimum": 0, "maximum": 1 },
            "achievement": { "type": "number", "minimum": 0, "maximum": 1 },
            "power": { "type": "number", "minimum": 0, "maximum": 1 },
            "security": { "type": "number", "minimum": 0, "maximum": 1 },
            "conformity": { "type": "number", "minimum": 0, "maximum": 1 },
            "tradition": { "type": "number", "minimum": 0, "maximum": 1 },
            "benevolence": { "type": "number", "minimum": 0, "maximum": 1 },
            "universalism": { "type": "number", "minimum": 0, "maximum": 1 }
          },
          "required": [
            "self-direction",
            "stimulation",
            "hedonism",
            "achievement",
            "power",
            "security",
            "conformity",
            "tradition",
            "benevolence",
            "universalism"
          ],
          "additionalProperties": false
        },
        "interests": {
          "type": "object",
          "properties": {
            "domains": { "type": "array", "items": { "$ref": "#/$defs/conciseLowercaseTerm" } },
            "activities": { "type": "array", "items": { "$ref": "#/$defs/conciseLowercaseTerm" } }
          },
          "required": ["domains", "activities"],
          "additionalProperties": false
        }
      },
      "required": ["username", "personality", "communication", "values", "interests", "user_id"],
      "additionalProperties": false
    })
}
