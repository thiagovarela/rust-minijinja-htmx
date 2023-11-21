use oauth2::{basic::BasicClient, TokenResponse};
// Alternatively, this can be oauth2::curl::http_client or a custom.
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenUrl,
};
use std::env;

use crate::models::OAuthUser;

use once_cell::sync::Lazy;

static GOOGLE_CLIENT: Lazy<BasicClient> = Lazy::new(|| {
    let google_client_id = ClientId::new(
        env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );
    let google_client_secret = ClientSecret::new(
        env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
    );
    let google_redirect_uri = RedirectUrl::new(
        env::var("GOOGLE_REDIRECT_URI")
            .expect("Missing the GOOGLE_REDIRECT_URI environment variable."),
    )
    .expect("Invalid redirect uri");
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
        .expect("Invalid token endpoint URL");

    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(google_redirect_uri);

    client
});

pub async fn get_google_auth_url() -> (String, CsrfToken, PkceCodeVerifier) {
    // Google supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state) = GOOGLE_CLIENT
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.email".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.profile".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    (authorize_url.to_string(), csrf_state, pkce_code_verifier)
}

#[derive(Debug, serde::Deserialize)]
struct GoogleUser {
    pub id: String,
    pub email: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
    pub locale: String,
}

pub async fn retrieve_user(
    code: String,
    pkce_code_verifier: String,
) -> Result<OAuthUser, anyhow::Error> {
    dbg!(&code, &pkce_code_verifier);
    let token_response = GOOGLE_CLIENT
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_code_verifier))
        .request_async(async_http_client)
        .await;

    let token_response = token_response.unwrap();

    let access_token = token_response.access_token().secret();
    let url = format!(
        "https://www.googleapis.com/oauth2/v1/userinfo?alt=json&access_token={}",
        access_token
    );

    let user = reqwest::get(url).await?.json::<GoogleUser>().await?;

    Ok(OAuthUser {
        email: user.email,
        provider: "google".into(),
        provider_id: user.id,
        first_name: user.given_name,
        last_name: user.family_name,
        picture: user.picture,
        locale: user.locale,
    })
}
