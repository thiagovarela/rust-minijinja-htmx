use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use axum_extra::extract::Form;
use minijinja::context;
use sqlx::PgPool;

use crate::{
    error::AppError,
    models::{Site, SiteKey, User},
    TEMPLATES,
};

pub async fn site(State(pool): State<PgPool>, user: User) -> Result<impl IntoResponse, AppError> {
    let site = sqlx::query_as!(
        Site,
        "SELECT account_id, url, description, posts_prefix, supported_langs
        FROM sites WHERE account_id = $1",
        user.account_id
    )
    .fetch_one(&pool)
    .await?;

    let tmpl = TEMPLATES.get_template("app/settings/site.html")?;
    let content = tmpl.render(context! {site})?;
    Ok(Html(content))
}

#[derive(Debug, serde::Deserialize)]
pub struct SiteSave {
    pub url: Option<String>,
    pub description: Option<String>,
    pub posts_prefix: Option<String>,
    #[serde(alias = "supported_langs[]")]
    pub supported_langs: Vec<String>,
}

pub async fn site_patch(
    State(pool): State<PgPool>,
    user: User,
    Form(form): Form<SiteSave>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query_as!(
        Site,
        "UPDATE sites SET 
        url = COALESCE($2, url),
        description = COALESCE($3, description),
        posts_prefix = COALESCE($4, posts_prefix), 
        supported_langs = COALESCE($5, supported_langs)   
        WHERE account_id = $1
        ",
        user.account_id,
        form.url,
        form.description,
        form.posts_prefix,
        &form.supported_langs
    )
    .execute(&pool)
    .await?;

    Ok((StatusCode::OK, "Site settings saved successfully"))
}

pub async fn api_keys(
    State(pool): State<PgPool>,
    user: User,
) -> Result<impl IntoResponse, AppError> {
    let keys = sqlx::query_as!(
        SiteKey,
        "SELECT account_id, secret, expires_at FROM site_keys WHERE account_id = $1 order by expires_at DESC",
        user.account_id
    )
    .fetch_all(&pool)
    .await?;
    let tmpl = TEMPLATES.get_template("app/settings/api-keys.html")?;
    let content = tmpl.render(context! { keys })?;
    Ok(Html(content))
}
