use jsonwebtoken::{decode, decode_header, jwk, Algorithm, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::error::Error;
use ic_cdk::api::time;

pub struct OAuthConfig<'a> {
    pub issuer_configs: &'a [IssuerConfig<'a>],
}

#[derive(Debug, Serialize, Deserialize)]
struct GenericClaims {
    iss: String,
    aud: String,
    exp: u64,
}

#[derive(Debug)]
struct IssuerConfig<'a> {
    pub issuer: &'a str,
    pub jwks_url: &'a str,
    pub expected_audience: &'a str,
    pub authorization_server: &'a str,
}

async fn fetch_jwks(jwks_url: &str) -> Result<jwk::JWKSet, Box<dyn Error>> {
    let client = Client::new();
    let response = client.get(jwks_url).send()?.json::<jwk::JWKSet>()?;
    Ok(response)
}

fn extract_issuer(token: &str) -> Result<String, Box<dyn Error>> {
    let validation = &mut Validation::default();
    validation.insecure_disable_signature_validation();

    let token_data: TokenData<GenericClaims> =
        decode(token, &DecodingKey::from_secret(&[]), validation)?;
    Ok(token_data.claims.iss)
}

async fn validate_token(
    token: &str,
    issuer_configs: &[IssuerConfig<'_>],
) -> Result<GenericClaims, Box<dyn Error>> {
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

    let token_data = decode::<GenericClaims>(token, &decoding_key, &validation)?;

    if token_data.claims.exp < time()/ 1_000_000_000 {
        return Err("Token has expired".into());
    }

    Ok(token_data.claims)
}
