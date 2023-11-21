use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::{Cookie, Key, PrivateCookieJar, SameSite};

use crate::jwt::validate_jwt_token;
use crate::models::AuthUser;

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Key: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = PrivateCookieJar::<Key>::from_request_parts(parts, &state)
            .await
            .unwrap();
        if let Some(token) = jar.get("auth.token") {
            let auth_user = validate_jwt_token(token.value().to_string())
                .map_err(|_er| StatusCode::UNAUTHORIZED)?;
            Ok(auth_user)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CsrfToken {
    pub state: String,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for CsrfToken
where
    Key: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar: PrivateCookieJar<Key> = PrivateCookieJar::from_request_parts(parts, state)
            .await
            .unwrap();
        if let Some(token) = jar.get("auth.csrf-token") {
            Ok(CsrfToken {
                state: token.value().to_string(),
            })
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PkceToken {
    pub code: String,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for PkceToken
where
    Key: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar: PrivateCookieJar<Key> = PrivateCookieJar::from_request_parts(parts, state)
            .await
            .unwrap();
        if let Some(token) = jar.get("auth.pkce-token") {
            Ok(PkceToken {
                code: token.value().to_string(),
            })
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AuthKeyId {
    pub id: i64,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthKeyId
where
    Key: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar: PrivateCookieJar<Key> = PrivateCookieJar::from_request_parts(parts, state)
            .await
            .unwrap();
        if let Some(token) = jar.get("auth.key-id") {
            Ok(AuthKeyId {
                id: token
                    .value()
                    .to_string()
                    .parse::<i64>()
                    .map_err(|_| StatusCode::BAD_REQUEST)?,
            })
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

pub async fn redirect_unauthenticated_user<B>(request: Request<B>, next: Next<B>) -> Response {
    let response = next.run(request).await;
    match response.status() {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Redirect::to("/auth").into_response(),
        _ => response,
    }
}

pub fn build_cookie_response(mut jar: PrivateCookieJar, token: String) -> Response {
    let cookie = Cookie::build("auth.token", token)
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .finish();
    let mut csrf = Cookie::build("auth.csrf-token", "")
        .path("/")
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();
    csrf.make_removal();
    let mut pkce = Cookie::build("auth.pkce-token", "")
        .path("/")
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();
    pkce.make_removal();

    jar = jar.add(csrf);
    jar = jar.add(pkce);
    jar = jar.add(cookie);

    (jar, Redirect::to("/app")).into_response()
}
