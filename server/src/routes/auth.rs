use crate::models::{AccountKey, User};
use crate::{AppError, AppState};
use auth::{
    axum::{AuthKeyId, CsrfToken, PkceToken},
    google::{self, get_google_auth_url},
    jwt::create_jwt_token,
};
use axum::{
    body::Body,
    extract::{FromRequestParts, Path, Query, State},
    http::{Request, StatusCode},
    response::{Html, IntoResponse, Redirect},
    Form,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    PrivateCookieJar,
};
use sqlx::PgPool;

use crate::TEMPLATES;

pub(crate) async fn choose(jar: PrivateCookieJar) -> Result<impl IntoResponse, AppError> {
    if let Some(_) = jar.get("auth.key-id") {
        Ok(Redirect::to("/auth/sign-in").into_response())
    } else {
        let tmpl = TEMPLATES.get_template("auth/choose.html")?;
        let content = tmpl.render(())?;
        Ok(Html(content).into_response())
    }
}

pub fn build_cookie(name: &'static str, value: String) -> Cookie {
    Cookie::build(name, value)
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .finish()
}

#[derive(serde::Deserialize)]
pub struct ChooseAction {
    account_name: String,
}

pub(crate) async fn choose_action(
    State(pool): State<PgPool>,
    mut jar: PrivateCookieJar,
    Form(choose): Form<ChooseAction>,
) -> Result<impl IntoResponse, AppError> {
    let key = sqlx::query_as!(
        AccountKey,
        r#"
        SELECT ak.id, ak.account_id, ak.secret, ak.expires_at
        FROM account_keys ak
        INNER JOIN accounts a on (ak.account_id = a.id)
        WHERE a.subdomain = $1 and (ak.expires_at is null OR ak.expires_at > now())
        ORDER BY ak.expires_at DESC
        "#,
        choose.account_name,
    )
    .fetch_one(&pool)
    .await?;

    jar = jar.add(build_cookie("auth.key-id", key.id.to_string()));
    Ok((jar, Redirect::to("/auth/sign-in")).into_response())
}

pub(crate) async fn sign_in(jar: PrivateCookieJar) -> Result<impl IntoResponse, AppError> {
    if jar.get("auth.key-id").is_none() {
        return Ok(Redirect::to("/auth").into_response());
    }

    let tmpl = TEMPLATES.get_template("auth/sign-in.html")?;
    let content = tmpl.render(())?;
    Ok(Html(content).into_response())
}

pub(crate) async fn sign_out(jar: PrivateCookieJar) -> Result<impl IntoResponse, AppError> {
    let jar = jar.remove(Cookie::named("auth.token"));
    Ok((jar, Redirect::to("/auth")).into_response())
}

pub(crate) async fn sign_in_with_google(
    mut jar: PrivateCookieJar,
) -> Result<impl IntoResponse, AppError> {
    let (url, token, code) = get_google_auth_url().await;
    let token = token.secret().clone();
    let code = code.secret().clone();
    jar = jar.add(build_cookie("auth.csrf-token", token));
    jar = jar.add(build_cookie("auth.pkce-token", code));

    Ok((jar, Redirect::to(url.as_str())))
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct GoogleQuery {
    pub code: String,
    pub state: String,
}

pub async fn auth_callback_handler(
    State(app_state): State<AppState>,
    Path(provider): Path<String>,
    csrf_token: CsrfToken,
    pkce_token: PkceToken,
    auth_key_id: AuthKeyId,
    mut jar: PrivateCookieJar,
    req: Request<Body>,
) -> Result<impl IntoResponse, AppError> {
    let mut parts = req.into_parts();
    let oauth_user = {
        match provider.as_ref() {
            "google" => {
                let google_query: Query<GoogleQuery> =
                    Query::from_request_parts(&mut parts.0, &app_state)
                        .await
                        .unwrap();
                if google_query.state == csrf_token.state {
                    google::retrieve_user(google_query.code.clone(), pkce_token.code).await
                } else {
                    Err(anyhow::anyhow!("invalid csrf state"))
                }
            }
            _ => Err(anyhow::anyhow!("invalid provider")),
        }
    }
    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()).into_response())
    .unwrap();

    let pool = app_state.db_pool;
    let email = oauth_user.email;
    let name = format!("{} {}", oauth_user.first_name, oauth_user.last_name);

    let key = sqlx::query_as!(
        AccountKey,
        r#"
        SELECT ak.id, ak.account_id, ak.secret, ak.expires_at
        FROM account_keys ak
        WHERE ak.id = $1 AND (ak.expires_at is null OR ak.expires_at > now())        
        "#,
        auth_key_id.id,
    )
    .fetch_one(&pool)
    .await?;

    let user = sqlx::query_as!(
        User,
        r#"
        SELECT u.id, u.account_id, u.email, u.picture, u.updated_at
        FROM users u                
        where u.account_id = $1 and u.email = $2
        "#,
        key.account_id,
        email,
    )
    .fetch_one(&pool)
    .await?;

    let _ = sqlx::query_as!(
        User,
        r#"
        UPDATE users SET name = $2, picture = $3, preferred_language = $4
        WHERE id = $1
        "#,
        user.id,
        name,
        oauth_user.picture,
        oauth_user.locale
    )
    .execute(&pool)
    .await?;

    let token = create_jwt_token(user.id.to_string(), key.id.to_string()).unwrap();
    jar = jar.add(build_cookie("auth.token", token));
    jar = jar.remove(Cookie::named("auth.pkce-token"));
    jar = jar.remove(Cookie::named("auth.csrf-token"));

    Ok((jar, Redirect::to("/app")))
}
