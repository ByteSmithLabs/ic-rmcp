use candid::CandidType;
use chrono::DateTime;
use ic_cdk::{api::time, init, query, update};
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use ic_rmcp::{
    model::*, schema_for_type, Context, Error, Handler, IssuerConfig, OAuthConfig, Server,
};
use serde::Deserialize;
use std::cell::RefCell;

thread_local! {
    static ARGS : RefCell<InitArgs> =  RefCell::default();
}

#[init]
fn init(config: InitArgs) {
    ARGS.with_borrow_mut(|args| *args = config);
}

#[derive(Deserialize, CandidType, Default)]
struct InitArgs {
    metadata_url: String,
    resource: String,
    issuer: String,
    jwks_url: String,
    authorization_server: Vec<String>,
    audience: String,
}

#[query]
fn http_request(_: HttpRequest) -> HttpResponse {
    HttpResponse::builder()
        .with_status_code(StatusCode::OK)
        .with_upgrade(true)
        .build()
}

struct Clock;

impl Handler for Clock {
    fn get_info(&self, _: Context) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "Clock".to_string(),
                version: "1.0.0".to_string(),
            },
            instructions: Some(
                "This server provides a tell_time tool that tell the current time in GMT+0"
                    .to_string(),
            ),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _: Context,
        _: Option<PaginatedRequestParam>,
    ) -> Result<ListToolsResult, Error> {
        Ok(ListToolsResult {
            next_cursor: None,
            tools: vec![Tool::new(
                "tell_time",
                "Tell the current time in GMT+0",
                schema_for_type::<EmptyObject>(),
            )],
        })
    }

    async fn call_tool(
        &self,
        context: Context,
        requests: CallToolRequestParam,
    ) -> Result<CallToolResult, Error> {
        match requests.name.as_ref() {
            "tell_time" => Ok(CallToolResult::success(
                Content::text(format!(
                    "You're logged in as {}, and the current time is: {}",
                    context.subject.unwrap(),
                    DateTime::from_timestamp_nanos(time() as i64)
                        .to_rfc3339()
                ))
                .into_contents(),
            )),
            _ => Err(Error::invalid_params("not found tool", None)),
        }
    }
}

#[update]
async fn http_request_update(req: HttpRequest<'_>) -> HttpResponse<'_> {
    Clock {}
        .handle_with_oauth(
            &req,
            ARGS.with_borrow(|args| OAuthConfig {
                metadata_url: args.metadata_url.clone(),
                resource: args.resource.clone(),
                issuer_configs: IssuerConfig {
                    issuer: args.issuer.clone(),
                    jwks_url: args.jwks_url.clone(),
                    authorization_server: args.authorization_server.clone(),
                    audience: args.audience.clone(),
                },
                scopes_supported: vec![],
            }),
        )
        .await
}

ic_cdk::export_candid!();
