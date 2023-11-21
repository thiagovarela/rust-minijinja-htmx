use chrono::{Duration, Utc};
use jwt_compact::{
    alg::{Hs256, Hs256Key},
    AlgorithmExt, Claims, Header, TimeOptions, Token, UntrustedToken,
};
use serde::{Deserialize, Serialize};

use crate::models::AuthUser;

use once_cell::sync::Lazy;

static JWT_KEY: Lazy<String> =
    Lazy::new(|| std::env::var("JWT_KEY").expect("JWT_KEY environment variable must be set."));

/// Custom claims encoded in the token.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct CustomClaims {
    #[serde(rename = "sub")]
    subject: String,
    kid: String,
}

pub fn create_jwt_token(subject: String, kid: String) -> Result<String, anyhow::Error> {
    let time_options = TimeOptions::default();
    let key = Hs256Key::new(JWT_KEY.as_bytes());
    let header = Header::empty();
    let claims = Claims::new(CustomClaims { subject, kid })
        .set_duration_and_issuance(&time_options, Duration::days(7))
        .set_not_before(Utc::now());
    let token_string = Hs256.token(&header, &claims, &key)?;
    Ok(token_string)
}

pub fn validate_jwt_token(token_string: String) -> Result<AuthUser, anyhow::Error> {
    let time_options = TimeOptions::default();
    let key = Hs256Key::new(JWT_KEY.as_bytes());
    let token = UntrustedToken::new(&token_string)?;
    let token: Token<CustomClaims> = Hs256.validator(&key).validate(&token)?;
    token
        .claims()
        .validate_expiration(&time_options)?
        .validate_maturity(&time_options)?;
    let claims = &token.claims().custom;
    Ok(AuthUser {
        user_id: claims.subject.clone().parse::<i64>()?,
        kid: claims.kid.clone().parse::<i64>()?,
    })
}
