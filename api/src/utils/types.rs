use crate::utils::errors::AppError;

pub type AppResult<T> = Result<T, AppError>;