use ic_cdk::api::time;
use ic_cdk::management_canister::{http_request, HttpMethod, HttpRequestArgs};
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;

thread_local! {
    pub static JWT_SETS: RefCell<HashMap<String, JwkSet>> = RefCell::default();
}

pub struct OAuthConfig<'a> {
    pub issuer_configs: &'a [IssuerConfig<'a>],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    iss: String,
    aud: String,
    exp: u64,
}

#[derive(Debug)]
pub struct IssuerConfig<'a> {
    pub issuer: &'a str,
    pub jwks_url: &'a str,
    pub expected_audience: &'a str,
    pub authorization_server: &'a str,
}

async fn fetch_jwks(jwks_url: &str) -> Result<JwkSet, Box<dyn Error>> {
    if let Some(set) = JWT_SETS.with_borrow(|jwt_sets| jwt_sets.get(jwks_url).map(|set| set.clone()))
    {
        return Ok(set);
    }

    let body = http_request(&HttpRequestArgs {
        url: jwks_url.to_string(),
        max_response_bytes: Some(5_000),
        method: HttpMethod::GET,
        headers: vec![],
        body: None,
        transform: None,
    })
    .await?
    .body;

    let set = from_slice::<JwkSet>(&body)?;
    JWT_SETS.with_borrow_mut(|jwt_sets| jwt_sets.insert(jwks_url.to_string(), set.clone()));
    Ok(set)
}

fn extract_issuer(token: &str) -> Result<String, Box<dyn Error>> {
    let validation = &mut Validation::default();
    validation.insecure_disable_signature_validation();

    let token_data: TokenData<Claims> = decode(token, &DecodingKey::from_secret(&[]), validation)?;
    Ok(token_data.claims.iss)
}

pub async fn validate_token(
    token: &str,
    issuer_configs: &[IssuerConfig<'_>],
) -> Result<Claims, Box<dyn Error>> {
    if token.is_empty() {
        return Err("No token provided".into());
    }

    let issuer = extract_issuer(token)?;

    let config = issuer_configs
        .iter()
        .find(|cfg| cfg.issuer == issuer)
        .ok_or(format!("Unknown issuer: {}", issuer))?;

    let header = decode_header(token)?;
    let kid = header.kid.ok_or("No key ID (kid) in token header")?;

    let jwks = fetch_jwks(&config.jwks_url).await?;
    let jwk = jwks
        .find(&kid)
        .ok_or("No matching key found in JWKS for the given kid")?;

    let decoding_key = DecodingKey::from_jwk(jwk)?;

    let mut validation = Validation::new(header.alg);
    validation.set_issuer(&[&config.issuer]);
    validation.set_audience(&[&config.expected_audience]);
    validation.validate_exp = false;

    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;

    if token_data.claims.exp < time() / 1_000_000_000 {
        return Err("Token has expired".into());
    }

    Ok(token_data.claims)
}
