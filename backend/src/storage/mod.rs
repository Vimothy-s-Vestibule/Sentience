pub mod sqlxlite;

use crate::AppError;
use syl_scr_common::db_schema::ScoreFlat;

pub trait AppStorage {
    type DBConnectionObject;

    fn init(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Self::DBConnectionObject, AppError>> + Send;

    fn insert_score(
        &self,
        pool: &Self::DBConnectionObject,
        score: &ScoreFlat,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}
