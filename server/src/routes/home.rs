use crate::{error::AppError, TEMPLATES};
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use sqlx::PgPool;

pub(crate) async fn index() -> Result<impl IntoResponse, AppError> {
    let tmpl = TEMPLATES.get_template("index.html")?;
    let content = tmpl.render(())?;
    Ok(Html(content))
}

pub(crate) async fn health(State(pool): State<PgPool>) -> Result<impl IntoResponse, AppError> {
    let tmpl = TEMPLATES.get_template("index.html")?;
    let _content = tmpl.render(())?;

    sqlx::query_scalar!("select 'OK'").fetch_one(&pool).await?;

    Ok("OK".to_string())
}
