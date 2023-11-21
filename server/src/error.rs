use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum AppError {
    InternalServerError(anyhow::Error),
    QueryError(sqlx::Error),
    Forbidden(String),
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InternalServerError(_inner) => {
                dbg!(&_inner);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("something went wrong: {}", _inner),
                )
            }
            AppError::QueryError(_) => (StatusCode::BAD_REQUEST, "Bad Request".to_string()),
            AppError::Forbidden(message) => (StatusCode::FORBIDDEN, message),
            AppError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message),
        };

        (status, error_message).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError::InternalServerError(error.into())
    }
}

impl From<minijinja::Error> for AppError {
    fn from(error: minijinja::Error) -> Self {
        AppError::InternalServerError(error.into())
    }
}
