use crate::AppError;
use serde::{Deserialize, Serialize};

pub mod json_file;
pub mod sqlite;

pub trait DiscordMessageStore {
    fn init(&mut self) -> impl std::future::Future<Output = Result<(), crate::AppError>> + Send;
    fn all(&self) -> &Vec<crate::DiscordMessage>;
}
