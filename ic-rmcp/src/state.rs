use ic_cdk::management_canister::{
    http_request_with_closure, HttpMethod, HttpRequestArgs, HttpRequestResult,
};
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

    let body = http_request_with_closure(
        &HttpRequestArgs {
            url: jwks_url.to_string(),
            max_response_bytes: Some(5_000),
            method: HttpMethod::GET,
            headers: vec![],
            body: None,
            transform: None,
        },
        |raw| HttpRequestResult {
            status: raw.status.clone(),
            body: raw.body.clone(),
            headers: vec![],
        },
    )
    .await
    .map_err(|err| err.to_string())?
    .body;

    let set = from_slice::<JwkSet>(&body).map_err(|err| err.to_string())?;

    JWT_SET.with_borrow_mut(|jwt_set| *jwt_set = Some(set.clone()));
    Ok(set)
}
