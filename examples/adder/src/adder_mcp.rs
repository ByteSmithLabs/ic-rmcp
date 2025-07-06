use ic_rmcp::{model::*, schema_for_type, Error, Handler};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{from_value, Value};

#[derive(JsonSchema, Deserialize)]
struct AddRequest {
    a: f64,
    b: f64,
}

pub struct Adder;

impl Handler for Adder {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "Adder".to_string(),
                version: "1.0.0".to_string(),
            },
            instructions: Some("This server is a tool help adding two number.".to_string()),
            ..Default::default()
        }
    }

    async fn list_tools(&self, _: Option<PaginatedRequestParam>) -> Result<ListToolsResult, Error> {
        ic_cdk::println!("List tools called");
        Ok(ListToolsResult {
            next_cursor: None,
            tools: vec![Tool::new(
                "add",
                "Add two numbers",
                schema_for_type::<AddRequest>(),
            )],
        })
    }

    async fn call_tool(&self, requests: CallToolRequestParam) -> Result<CallToolResult, Error> {
        ic_cdk::println!(
            "Call tool: {}, arguments: {:?}",
            requests.name,
            requests.arguments
        );
        match requests.name.as_ref() {
            "add" => match requests.arguments {
                None => Err(Error::invalid_params("invalid arguments to tool add", None)),
                Some(data) => match from_value::<AddRequest>(Value::Object(data)) {
                    Err(_) => Err(Error::invalid_params("invalid arguments to tool add", None)),
                    Ok(args) => Ok(CallToolResult::success(
                        Content::text(format!("{:.2}", args.a + args.b)).into_contents(),
                    )),
                },
            },
            _ => Err(Error::invalid_params("not found tool", None)),
        }
    }
}
