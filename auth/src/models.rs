#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum AuthUserKind {
    Authenticated(AuthUser),
    Anonymous,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AuthUser {
    pub user_id: i64,
    pub kid: i64,
}

#[derive(Debug, Clone)]
pub struct OAuthUser {
    pub email: String,
    pub provider: String,
    pub provider_id: String,
    pub first_name: String,
    pub last_name: String,
    pub picture: String,
    pub locale: String,
}
