use axum::response::{Html, IntoResponse};
use minijinja::context;

use crate::{error::AppError, extractors::JinjaContext, models::User, TEMPLATES};

pub(crate) async fn index(ctx: JinjaContext) -> Result<impl IntoResponse, AppError> {
    let tmpl = TEMPLATES.get_template("app/index.html")?;
    let content = tmpl.render(context! { ..ctx })?;
    Ok(Html(content))
}

pub(crate) async fn profile_snippet(user: User) -> Result<impl IntoResponse, AppError> {
    let tmpl = TEMPLATES.get_template("app/profile-snippet.html")?;
    let content = tmpl.render(context! { user => user })?;
    Ok(Html(content))
}
