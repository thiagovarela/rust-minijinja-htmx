use auth::AuthUser;
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, Request},
};
use minijinja::context;

use crate::{error::AppError, models::User, AppState};

pub async fn add_user_to_request_extensions<B>(
    State(state): State<AppState>,
    auth_user: AuthUser,
    mut request: Request<B>,
) -> Result<Request<B>, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT u.id, u.account_id, u.email, u.picture, u.updated_at
        FROM users u
        INNER JOIN accounts a on (a.id = u.account_id)
        INNER JOIN account_keys ak on (ak.account_id = a.id)
        where ak.id = $1 and u.id = $2
        "#,
        auth_user.kid,
        auth_user.user_id,
    )
    .fetch_one(&state.db_pool)
    .await?;

    request.extensions_mut().insert(user);

    Ok(request)
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(user) = parts.extensions.get::<User>() {
            Ok(user.clone())
        } else {
            Err(AppError::Unauthorized(("User is not authenticated").into()))
        }
    }
}

pub struct JinjaContext(pub minijinja::Value);

impl From<JinjaContext> for minijinja::Value {
    fn from(value: JinjaContext) -> Self {
        value.0
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for JinjaContext
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Ok(user) = User::from_request_parts(parts, &state).await {
            Ok(JinjaContext(context! { user => user }))
        } else {
            Err(AppError::Unauthorized(("User is not authenticated").into()))
        }
    }
}
