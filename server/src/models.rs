use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    #[serde(skip)]
    pub account_id: i64,
    pub email: String,
    pub picture: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountKey {
    pub id: i64,
    #[serde(skip)]
    pub account_id: i64,
    pub secret: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    #[serde(skip)]
    pub account_id: i64,
    pub url: Option<String>,
    pub description: Option<String>,
    pub posts_prefix: Option<String>,
    pub supported_langs: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SiteKey {
    #[serde(skip)]
    pub account_id: i64,
    pub secret: String,
    pub expires_at: Option<DateTime<Utc>>,
}
