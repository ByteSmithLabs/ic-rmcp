use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq)]
pub struct OAuthConfig {
    pub metadata_url: String,
    pub resource: String,
    pub issuer_configs: IssuerConfig,
    pub scopes_supported: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Claims {
    pub sub: String,
    iss: String,
    aud: String,
    exp: u64,
}

#[derive(Debug, Default, PartialEq)]
pub struct IssuerConfig {
    pub issuer: String,
    pub jwks_url: String,
    pub authorization_server: Vec<String>,
    pub audience: String,
}

pub fn validate_token(
    token: &str,
    issuer_configs: &IssuerConfig,
    jwt_set: JwkSet,
) -> Result<Claims, String> {
    if token.is_empty() {
        return Err("No token provided".to_string());
    }

    let header = decode_header(token).map_err(|err| format!("decode header: {err}"))?;
    let kid = header
        .kid
        .ok_or("No key ID (kid) in token header".to_string())?;

    let jwk = jwt_set
        .find(&kid)
        .ok_or("No matching key found in JWKS for the given kid".to_string())?;

    let decoding_key =
        DecodingKey::from_jwk(jwk).map_err(|err| format!("get decoding key: {err}"))?;

    let mut validation = Validation::new(header.alg);
    validation.set_issuer(&[&issuer_configs.issuer]);
    validation.set_audience(&[&issuer_configs.audience]);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|err| format!("invalid token: {err}"))?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_value, json};

    #[test]
    fn test_validate_token() {
        assert_eq!(
            validate_token("", &IssuerConfig::default(), JwkSet { keys: vec![] }),
            Err("No token provided".to_string())
        );

        assert!(
            validate_token("ey..", &IssuerConfig::default(), JwkSet { keys: vec![] })
                .unwrap_err()
                .contains("decode header"),
        );

        assert!(validate_token(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..",
            &IssuerConfig::default(),
            JwkSet { keys: vec![] }
        )
        .unwrap_err()
        .contains("No key ID (kid) in token header"),);

        assert!(validate_token(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImtleS0xMjM0In0..",
            &IssuerConfig::default(),
            JwkSet { keys: vec![] }
        )
        .unwrap_err()
        .contains("No matching key found in JWKS for the given kid"),);

        assert!(validate_token(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImtleS0xMjM0In0..",
            &IssuerConfig::default(),
            from_value::<JwkSet>(json!({
              "keys": [
                {
                  "kid": "key-1234",
                  "alg": "ES256",
                  "kty": "EC",
                  "crv": "P-256",
                  "x": "foo",
                  "y": "bar"
                }
              ]
            }))
            .unwrap(),
        )
        .unwrap_err()
        .contains("get decoding key"),);

        assert!(validate_token(
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJFUzI1NiIsImtpZCI6IjdkMGFlYzgyYTRiMmRjN2M4ZjA2NmYzY2Y0ZDY1MTdlIn0.e30.qi44LygNKrsh9x0wpz16aau46quyRNTugZV2MeRtagRzOgAZ9VI4lJkbzNeo7HCFmUcLgHGp_vUxNSYmlk44TA7idqhVXg4oJN2m3GVyfkcV690Ju8j9P5a6lzFWSrNq_RLwAznZY9eHbMdRfMvdmY9c5OfnPwbrJ_NJkiRqbrkA",
            &IssuerConfig::default(),
            from_value::<JwkSet>(json!({
                          "keys": [
                            {
              "crv": "P-256",
              "key_ops": [
                "verify"
              ],
              "kty": "EC",
              "x": "Z0VuXQcCOTh06Ge3u2Ts77FYLRwgHqIQkPU_Mb9pthU",
              "y": "q055d-H_g7_tbxdEVt1hIbOHHG2_1R8X-JZ7kLC48aM",
              "alg": "ES256",
              "use": "sig",
              "kid": "7d0aec82a4b2dc7c8f066f3cf4d6517e"
            }
                          ]
                        }))
            .unwrap(),
        )
        .unwrap_err().contains("invalid token"),);
    }
}
