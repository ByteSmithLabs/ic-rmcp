use ic_cdk::api::time;
use ic_cdk::management_canister::{http_request, HttpMethod, HttpRequestArgs};
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    pub static JWT_SETS: RefCell<HashMap<String, JwkSet>> = RefCell::default();
}

pub struct OAuthConfig<'a> {
    pub metadata_url: &'a str,
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
pub enum Error {
    FetchJwk(String),
    Invalid(String),
    Expired,
    HttpOutcall(String),
}

#[derive(Debug)]
pub struct IssuerConfig<'a> {
    pub issuer: &'a str,
    pub jwks_url: &'a str,
    pub expected_audience: &'a str,
    pub authorization_server: &'a str,
}

async fn fetch_jwks(jwks_url: &str) -> Result<JwkSet, Error> {
    if let Some(set) =
        JWT_SETS.with_borrow(|jwt_sets| jwt_sets.get(jwks_url).map(|set| set.clone()))
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
    .await
    .map_err(|err| Error::HttpOutcall(err.to_string()))?
    .body;

    let set = from_slice::<JwkSet>(&body).map_err(|err| Error::FetchJwk(err.to_string()))?;

    JWT_SETS.with_borrow_mut(|jwt_sets| jwt_sets.insert(jwks_url.to_string(), set.clone()));
    Ok(set)
}

fn extract_issuer(token: &str) -> Result<String, Error> {
    let validation = &mut Validation::default();
    validation.insecure_disable_signature_validation();

    decode::<Claims>(token, &DecodingKey::from_secret(&[]), validation)
        .map_err(|err| Error::Invalid(err.to_string()))
        .map(|token_data| token_data.claims.iss)
}

pub async fn validate_token(
    token: &str,
    issuer_configs: &[IssuerConfig<'_>],
) -> Result<Claims, Error> {
    if token.is_empty() {
        return Err(Error::Invalid("No token provided".to_string()));
    }

    let issuer = extract_issuer(token)?;

    let config = issuer_configs
        .iter()
        .find(|cfg| cfg.issuer == issuer)
        .ok_or(Error::Invalid(format!("Unknown issuer: {}", issuer)))?;

    let header = decode_header(token).map_err(|err| Error::Invalid(err.to_string()))?;
    let kid = header.kid.ok_or(Error::Invalid("No key ID (kid) in token header".to_string()))?;

    let jwks = fetch_jwks(&config.jwks_url).await?;
    let jwk = jwks
        .find(&kid)
        .ok_or(Error::Invalid("No matching key found in JWKS for the given kid".to_string()))?;

    let decoding_key = DecodingKey::from_jwk(jwk).map_err(|err| Error::Invalid(err.to_string()))?;

    let mut validation = Validation::new(header.alg);
    validation.set_issuer(&[&config.issuer]);
    validation.set_audience(&[&config.expected_audience]);
    validation.validate_exp = false;

    let token_data = decode::<Claims>(token, &decoding_key, &validation).map_err(|err| Error::Invalid(err.to_string()))?;

    if token_data.claims.exp < time() / 1_000_000_000 {
        return Err(Error::Expired);
    }

    Ok(token_data.claims)
}
