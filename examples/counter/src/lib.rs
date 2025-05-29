use ic_cdk_macros::{query, update};
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use ic_rmcp::{Handler, Server};
use rmcp::{handler::server::tool::schema_for_type, model::*, Error};
use std::cell::RefCell;

thread_local! {
    static COUNTER : RefCell<i32> = const {RefCell::new(0)} ;
}

#[query]
fn http_request(_: HttpRequest) -> HttpResponse {
    HttpResponse::builder()
        .with_status_code(StatusCode::OK)
        .with_upgrade(true)
        .build()
}

struct Counter;

impl Handler for Counter {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "Counter".to_string(),
                version: "1.0.0".to_string(),
            },
            instructions: Some("This server provides a counter tool that can increment and decrement values. The counter starts at 0 and can be modified using the 'increment' and 'decrement' tools. Use 'get_value' to check the current count.".to_string()),
            ..Default::default()
        }
    }

    async fn list_tools(&self, _: Option<PaginatedRequestParam>) -> Result<ListToolsResult, Error> {
        Ok(ListToolsResult {
            next_cursor: None,
            tools: vec![
                Tool::new(
                    "increment",
                    "Increment the counter by 1",
                    schema_for_type::<EmptyObject>(),
                ),
                Tool::new(
                    "decrement",
                    "Decrement the counter by 1",
                    schema_for_type::<EmptyObject>(),
                ),
                Tool::new(
                    "get_value",
                    "Get the current value of the counter",
                    schema_for_type::<EmptyObject>(),
                ),
            ],
        })
    }

    async fn call_tool(&self, requests: CallToolRequestParam) -> Result<CallToolResult, Error> {
        match requests.name.as_ref() {
            "increment" => {
                COUNTER.with(|counter| {
                    let mut value = counter.borrow_mut();
                    *value += 1;
                });
                Ok(CallToolResult::success(
                    Content::text("Counter incremented").into_contents(),
                ))
            }
            "decrement" => {
                COUNTER.with(|counter| {
                    let mut value = counter.borrow_mut();
                    *value -= 1;
                });
                Ok(CallToolResult::success(
                    Content::text("Counter decremented").into_contents(),
                ))
            }
            "get_value" => {
                let value = COUNTER.with(|counter| *counter.borrow());
                Ok(CallToolResult::success(
                    Content::text(value.to_string()).into_contents(),
                ))
            }
            _ => Err(Error::invalid_params("not found tool", None)),
        }
    }
}

#[update]
async fn http_request_update(req: HttpRequest<'_>) -> HttpResponse<'_> {
    Counter {}
        .handle_with_auth(req, |headers| -> bool {
            ic_cdk::println!("Headers: {:?}", headers);
            headers
                .iter()
                .any(|(k, v)| k == "x-api-key" && v == "123456")
        })
        .await
}

ic_cdk::export_candid!();
