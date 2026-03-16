use crate::AppError;

pub mod gemini;

pub trait MessageEmbedder {
    fn embed_text(
        &self,
        text: &str,
        client: &reqwest::Client,
        _user_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<f32>, AppError>> + Send;
}
