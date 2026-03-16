use crate::VestibuleUserRecord;

pub mod gemini;

pub trait MessageScorer {
    fn score_message(
        &self,
        client: &reqwest::Client,
        model: &str,
        user_id: &str,
        content: &str,
    ) -> impl std::future::Future<Output = Result<VestibuleUserRecord, crate::AppError>> + Send;
}
