use crate::models::Site;
use crate::{
    error::AppError,
    extractors::JinjaContext,
    models::User,
    TEMPLATES,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Form,
};
use chrono::{DateTime, Utc};
use minijinja::context;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostWithRelations {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub slug: String,
    pub lang: String,
    pub short_description: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Author,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
pub struct Author {
    pub id: i64,
    pub name: String,
}

pub(crate) async fn index(
    State(pool): State<PgPool>,
    user: User,
    ctx: JinjaContext,
) -> Result<impl IntoResponse, AppError> {
    let posts = sqlx::query_as!(
        PostWithRelations,
        r#"
        SELECT p.id, p.title, p.content, p.lang, p.slug, p.short_description, p.updated_at, p.created_at, p.published_at,
                (u.id, u.name) as "author!: Author"
        FROM posts p
        INNER JOIN users u on (u.id = p.author_id)
        WHERE p.account_id = $1
        ORDER BY p.updated_at DESC 
        "#,
        user.account_id
    )
    .fetch_all(&pool)
    .await?;

    let ctx = context! {
        posts,
        ..ctx
    };

    let tmpl = TEMPLATES.get_template("app/posts/index.html")?;
    let content = tmpl.render(ctx)?;
    Ok(Html(content))
}

pub(crate) async fn draft(
    State(pool): State<PgPool>,
    user: User,
) -> Result<impl IntoResponse, AppError> {
    let draft_id = sqlx::query_scalar!(
        r#"
            INSERT INTO posts(title, content, slug, lang, account_id, author_id)
            VALUES ('Draft', '', 'draft', 'pt' , $1, $2)
            RETURNING id             
        "#,
        user.account_id,
        user.id
    )
    .fetch_one(&pool)
    .await?;

    Ok(Redirect::to(format!("/app/posts/{}", draft_id).as_str()))
}

async fn get_post(
    pool: &PgPool,
    post_id: i64,
    account_id: i64,
) -> Result<PostWithRelations, AppError> {
    Ok(sqlx::query_as!(
        PostWithRelations,
        r#"
        SELECT p.id, p.title, p.content, p.slug, p.lang, p.short_description, p.updated_at, p.created_at, p.published_at,
                (u.id, u.name) as "author!: Author"
        FROM posts p
        INNER JOIN users u on (u.id = p.author_id)
        WHERE p.id = $1 and p.account_id = $2
        ORDER BY p.updated_at DESC 
        "#,
        post_id,
        account_id
    )
    .fetch_one(pool)
    .await?)
}

async fn get_site(pool: &PgPool, account_id: i64) -> Result<Site, AppError> {
    Ok(sqlx::query_as!(
        Site,
        "SELECT account_id, url, description, posts_prefix, supported_langs
        FROM sites WHERE account_id = $1",
        account_id
    )
    .fetch_one(pool)
    .await?)
}

pub(crate) async fn post(
    State(pool): State<PgPool>,
    user: User,
    ctx: JinjaContext,
    Path(post_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let post = get_post(&pool, post_id, user.account_id).await?;

    let tmpl = TEMPLATES.get_template("app/posts/post.html")?;
    let ctx = context! {
        post => post,
        ..ctx
    };
    let content = tmpl.render(ctx)?;
    Ok(Html(content))
}

#[derive(serde::Deserialize)]
pub struct PostSave {
    pub content: String,
    pub title: String,
}

pub(crate) async fn post_patch(
    State(pool): State<PgPool>,
    user: User,
    Path(post_id): Path<i64>,
    Form(form): Form<PostSave>,
) -> Result<impl IntoResponse, AppError> {
    let slug = slug::slugify(&form.title);
    sqlx::query!(
        "UPDATE posts SET 
            title = COALESCE($3, title),
            content = COALESCE($4, content),
            slug = COALESCE($5, content)
        WHERE id = $1 AND account_id = $2",
        post_id,
        user.account_id,
        form.title,
        form.content,
        slug
    )
    .execute(&pool)
    .await?;

    Ok((StatusCode::OK, "Post saved successfully"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostSettings {
    pub id: i64,
    pub language: String,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Author,
}

pub(crate) async fn settings_general(
    State(pool): State<PgPool>,
    user: User,
    Path(post_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let post = get_post(&pool, post_id, user.account_id).await?;
    let site: Site = get_site(&pool, user.account_id).await?;

    let post_url_prefix = format!(
        "{}/{}",
        site.url.as_ref().unwrap_or(&"www.yoursite.com".to_string()),
        site.posts_prefix.as_ref().unwrap_or(&"posts".to_string())
    );

    let ctx = context! {
        post,
        site,
        post_url_prefix => post_url_prefix
    };

    let tmpl = TEMPLATES.get_template("app/posts/settings/general.html")?;
    let content = tmpl.render(ctx)?;
    Ok(Html(content))
}

pub(crate) async fn settings_seo(
    State(pool): State<PgPool>,
    user: User,
    Path(post_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let post = get_post(&pool, post_id, user.account_id).await?;
    let site: Site = get_site(&pool, user.account_id).await?;

    let tmpl = TEMPLATES.get_template("app/posts/settings/seo.html")?;
    let content = tmpl.render(context! {
        post,
        site => site,
    })?;
    Ok(Html(content))
}

pub(crate) async fn settings_open_graph(
    State(pool): State<PgPool>,
    user: User,
    Path(post_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let post = get_post(&pool, post_id, user.account_id).await?;
    let site: Site = get_site(&pool, user.account_id).await?;

    let tmpl = TEMPLATES.get_template("app/posts/settings/open-graph.html")?;
    let content = tmpl.render(context! {
        post,
        site => site,
    })?;
    Ok(Html(content))
}
