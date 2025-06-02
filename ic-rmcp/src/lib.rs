//! This SDK is intended for supporting MCP server development on [Internet Computer](https://internetcomputer.org) canisters. For normal platforms, check out [official SDK](https://github.com/modelcontextprotocol/rust-sdk).

//! ```rust
//! use ic_cdk_macros::{query, update};
//! use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
//! use ic_rmcp::{Handler, Server};
//! use rmcp::{handler::server::tool::schema_for_type, model::*, Error};
//! use ic_cdk::api::time;
//!
//! #[query]
//! fn http_request(_: HttpRequest) -> HttpResponse {
//!     HttpResponse::builder()
//!         .with_status_code(StatusCode::OK)
//!         .with_upgrade(true)
//!         .build()
//! }
//!
//! struct Clock;
//!
//! impl Handler for Clock {
//!     fn get_info(&self) -> ServerInfo {
//!         ServerInfo {
//!             capabilities: ServerCapabilities::builder().enable_tools().build(),
//!             server_info: Implementation {
//!                 name: "Clock".to_string(),
//!                 version: "1.0.0".to_string(),
//!             },
//!             ..Default::default()
//!         }
//!     }
//!
//!     async fn list_tools(&self, _: Option<PaginatedRequestParam>) -> Result<ListToolsResult, Error> {
//!         Ok(ListToolsResult {
//!             next_cursor: None,
//!             tools: vec![
//!                 Tool::new(
//!                     "get_time",
//!                     "Get the current timestamp in nanoseconds.",
//!                     schema_for_type::<EmptyObject>(),
//!                 ),
//!             ],
//!         })
//!     }
//!
//!     async fn call_tool(&self, requests: CallToolRequestParam) -> Result<CallToolResult, Error> {
//!         match requests.name.as_ref() {
//!             "get_time" => {
//!                 Ok(CallToolResult::success(
//!                     Content::text(format!("{}", time())).into_contents(),
//!                 ))
//!             },
//!             _ => Err(Error::invalid_params("not found tool", None)),
//!         }
//!     }
//! }
//!
//! #[update]
//! async fn http_request_update(req: HttpRequest<'_>) -> HttpResponse<'_> {
//!     Clock {}.handle(req).await
//! }
//!
//! ic_cdk::export_candid!();
//! ```
mod handler;
pub use handler::Handler;

mod server;
pub use server::Server;
