pub struct OAuthConfig<'a> {
    pub metadata: Metadata<'a>,
}

pub struct Metadata<'a> {
    pub authorization_servers:&'a [String],
}


use jsonwebtoken::{decode, decode_header, jwk, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::SystemTime;

// Claims structure for Google OAuth 2.0 JWT
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,    // Issuer (should be Google)
    aud: String,    // Audience (should match your client ID)
    exp: i64,       // Expiration time (Unix timestamp)
    iat: i64,       // Issued at
    sub: String,    // Subject (user ID)
    // Add other claims as needed
}

// Function to fetch Google's JWKS (JSON Web Key Set)
async fn fetch_google_jwks() -> Result<jwk::JWKSet, Box<dyn Error>> {
    let client = Client::new();
    let response = client
        .get("https://www.googleapis.com/oauth2/v3/certs")
        .send()?
        .json::<jwk::JWKSet>()?;
    Ok(response)
}

// Function to validate the Google OAuth 2.0 access token
async fn validate_google_token(
    token: &str,
    expected_audience: &str,
) -> Result<Claims, Box<dyn Error>> {
    // Step 1: Check if token is provided
    if token.is_empty() {
        return Err("No token provided".into());
    }

    // Step 2: Decode the token header to get the key ID (kid)
    let header = decode_header(token)?;
    let kid = header.kid.ok_or("No key ID (kid) in token header")?;

    // Step 3: Fetch Google's public keys (JWKS)
    let jwks = fetch_google_jwks().await?;
    let jwk = jwks
        .find(&kid)
        .ok_or("No matching key found in JWKS for the given kid")?;

    // Step 4: Create a decoding key from the JWK
    let decoding_key = DecodingKey::from_jwk(jwk)?;

    // Step 5: Set up validation rules
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&["https://accounts.google.com", "accounts.google.com"]);
    validation.set_audience(&[expected_audience]);
    validation.validate_exp = true; // Ensure expiration is checked

    // Step 6: Decode and validate the token
    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;

    // Step 7: Additional manual check for expiration (optional, as validation.validate_exp handles it)
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs() as i64;
    if token_data.claims.exp < current_time {
        return Err("Token has expired".into());
    }

    Ok(token_data.claims)
}


async fn main() -> Result<(), Box<dyn Error>> {
    // Example token (replace with actual token from request)
    let token = "eyJhbGciOiJSUzI1NiIsImtpZCI6Ij..."; // Replace with real token
    let expected_audience = "your-client-id.apps.googleusercontent.com"; // Replace with your Google Client ID

    match validate_google_token(token, expected_audience).await {
        Ok(claims) => {
            println!("Token is valid! Claims: {:?}", claims);
            // Proceed with granting access to the resource
        }
        Err(e) => {
            println!("Token validation failed: {}", e);
            // Deny access
        }
    }
    Ok(())
}