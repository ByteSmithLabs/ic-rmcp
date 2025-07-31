use ic_cdk::management_canister::{http_request, HttpMethod, HttpRequestArgs};
use jsonwebtoken::jwk::JwkSet;
use serde_json::from_slice;
use std::cell::RefCell;

thread_local! {
   pub static JWT_SET: RefCell<Option<JwkSet>> = RefCell::default();
}

pub async fn fetch_jwks(jwks_url: &str) -> Result<JwkSet, String> {
    if let Some(set) = JWT_SET.with_borrow(|jwt_set| jwt_set.clone()) {
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
    .map_err(|err| err.to_string())?
    .body;

    let set = from_slice::<JwkSet>(&body).map_err(|err| err.to_string())?;

    JWT_SET.with_borrow_mut(|jwt_set| *jwt_set = Some(set.clone()));
    Ok(set)
}
