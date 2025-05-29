use ic_cdk_macros::{query, update};
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use ic_rmcp::{Handler, Server};
use rmcp::{model::*, Error};
use serde::{Deserialize, Serialize};
use serde_json::{from_value, json, Value};
use std::borrow::Cow;

#[query]
fn http_request(_: HttpRequest) -> HttpResponse {
    HttpResponse::builder()
        .with_status_code(StatusCode::OK)
        .with_upgrade(true)
        .build()
}

struct Adder;

impl Handler for Adder {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "Adder".to_string(),
                version: "1.0.0".to_string(),
            },
            ..Default::default()
        }
    }

    async fn list_tools(&self, _: Option<PaginatedRequestParam>) -> Result<ListToolsResult, Error> {
        Ok(ListToolsResult {
            next_cursor: None,
            tools: vec![Tool::new(
                "add",
                "Add two numbers",
                object(json!({
                    "type": "object",
                    "properties": {
                        "a": { "type": "number", "description": "The first number" },
                        "b": { "type": "number", "description": "The second number" }
                    },
                    "required": ["a", "b"]
                })),
            )],
        })
    }

    async fn call_tool(&self, requests: CallToolRequestParam) -> Result<CallToolResult, Error> {
        let name = match requests.name {
            Cow::Borrowed(s) => s,
            Cow::Owned(ref s) => s,
        };

        match name {
            "add" => {
                #[derive(Serialize, Deserialize)]
                struct AddArgs {
                    a: f64,
                    b: f64,
                }

                match requests.arguments {
                    None => Err(Error::invalid_params("invalid arguments to tool add", None)),
                    Some(data) => match from_value::<AddArgs>(Value::Object(data)) {
                        Err(_) => Err(Error::invalid_params("invalid arguments to tool add", None)),
                        Ok(args) => Ok(CallToolResult::success(
                            Content::text(format!("{:.2}", args.a + args.b)).into_contents(),
                        )),
                    },
                }
            }
            _ => Err(Error::invalid_params("not found tool", None)),
        }
    }
}

#[update]
async fn http_request_update(req: HttpRequest<'_>) -> HttpResponse<'_> {
    Adder {}
        .handle_with_auth(req, |headers| -> bool {
            ic_cdk::println!("Headers: {:?}", headers);
            headers
                .iter()
                .any(|(k, v)| k == "x-api-key" && v == "123456")
        })
        .await
}

ic_cdk::export_candid!();
