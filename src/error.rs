//! Top-level error types

use error_stack::Report;

#[derive(Debug, thiserror::Error)]
#[error("An application error has occured")]
pub struct AppError;

pub type AppResult<T> = Result<T, Report<AppError>>;

/// A suggestion displayed to the user
pub struct Suggestion(pub &'static str);