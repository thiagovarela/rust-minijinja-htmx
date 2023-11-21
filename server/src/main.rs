use axum::extract::FromRef;
use axum::middleware;
use axum::routing::post;
use axum::{routing::get, Router};
use axum_extra::extract::cookie::Key;
use minijinja::{path_loader, Environment};
use routes::auth::auth_callback_handler;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};
use tower_http::{compression::CompressionLayer, services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod error;
mod extractors;
mod models;
pub(crate) use error::AppError;

mod routes;

use once_cell::sync::Lazy;

use crate::extractors::add_user_to_request_extensions;

static TEMPLATES: Lazy<Environment<'static>> = Lazy::new(|| {
    let mut env = Environment::new();
    minijinja_contrib::add_to_environment(&mut env);
    env.set_loader(path_loader("templates"));
    env
});

pub async fn database_pool(db_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("SET application_name = 'platform'; SET search_path = 'public';")
                    .await?;
                Ok(())
            })
        })
        .connect(db_url)
        .await
        .expect("Can't connect to database")
}

#[derive(Clone, FromRef)]
pub struct AppState {
    pub cookie_key: Key,
    pub db_pool: PgPool,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), axum::BoxError> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let serve_dir = ServeDir::new("public");
    let cookie_key =
        std::env::var("COOKIE_KEY").expect("COOKIE_KEY environment variable must be present");

    let db_str =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set");

    let pool = database_pool(&db_str).await;
    sqlx::migrate!("../migrations").run(&pool).await?;

    let app_state = AppState {
        cookie_key: Key::from(cookie_key.as_bytes()),
        db_pool: pool,
    };

    let auth_routes = Router::new()
        .route("/", get(routes::auth::choose))
        .route("/choose-action", post(routes::auth::choose_action))
        .route("/sign-in", get(routes::auth::sign_in))
        .route("/sign-out", get(routes::auth::sign_out))
        .route(
            "/sign-in-with-google",
            post(routes::auth::sign_in_with_google),
        )
        .route("/callback/:provider", get(auth_callback_handler));

    let posts_routes = Router::new()
        .route("/", get(routes::posts::index))
        .route("/draft", post(routes::posts::draft))
        .route(
            "/:id",
            get(routes::posts::post).patch(routes::posts::post_patch),
        )
        .route(
            "/:id/settings/general",
            get(routes::posts::settings_general),
        )
        .route("/:id/settings/seo", get(routes::posts::settings_seo))
        .route(
            "/:id/settings/open-graph",
            get(routes::posts::settings_open_graph),
        );

    let settings_routes = Router::new()
        .route(
            "/",
            get(routes::settings::site).patch(routes::settings::site_patch),
        )
        .route("/api-keys", get(routes::settings::api_keys));

    let app_routes = Router::new()
        .route("/", get(routes::app::index))
        .route("/profile-snippet", get(routes::app::profile_snippet))
        .nest("/posts", posts_routes)
        .nest("/settings", settings_routes)
        .layer(middleware::map_request_with_state(
            app_state.clone(),
            add_user_to_request_extensions,
        ));

    let app = Router::new()
        .route("/", get(routes::home::index))
        .route("/health", get(routes::home::health))
        .nest("/auth", auth_routes)
        .nest("/app", app_routes)
        .layer(TraceLayer::new_for_http())
        .fallback_service(serve_dir)
        .layer(CompressionLayer::new())
        .with_state(app_state);

    println!("Listening on 0.0.0.0:3000");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
