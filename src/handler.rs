use crate::{error::Error, model::*, service::Service};
use std::cmp::Ordering;

impl<H: ServerHandler> Service for H {
    async fn handle_request(&self, request: ClientRequest) -> Result<ServerResult, Error> {
        match request {
            ClientRequest::InitializeRequest(request) => self
                .initialize(request.params)
                .await
                .map(ServerResult::InitializeResult),
            ClientRequest::PingRequest(_request) => self.ping().await.map(ServerResult::empty),
            ClientRequest::CallToolRequest(request) => self
                .call_tool(request.params)
                .await
                .map(ServerResult::CallToolResult),
            ClientRequest::ListToolsRequest(request) => self
                .list_tools(request.params)
                .await
                .map(ServerResult::ListToolsResult),
            _ => Err(Error::new(ErrorCode::METHOD_NOT_FOUND,"Method not found", None)),
        }
    }

    async fn handle_notification(&self, notification: ClientNotification) -> Result<(), Error> {
        match notification {
            ClientNotification::InitializedNotification(_notification) => {
                self.on_initialized().await
            }
            _ => (),
        };
        Ok(())
    }
}

#[allow(unused_variables)]
pub trait ServerHandler {
    fn ping(&self) -> impl Future<Output = Result<(), Error>> {
        std::future::ready(Ok(()))
    }
    fn initialize(
        &self,
        request: InitializeRequestParam,
    ) -> impl Future<Output = Result<InitializeResult, Error>> {
        let mut info = self.get_info();
        let request_version = request.protocol_version.clone();

        let negotiated_protocol_version =
            match request_version.partial_cmp(&info.protocol_version) {
                Some(Ordering::Less) => request.protocol_version.clone(),
                Some(Ordering::Equal) => {
                   request.protocol_version.clone()
                }
                Some(Ordering::Greater) => {
                    info.protocol_version
                },
                None => {
                    return std::future::ready(Err(Error::internal_error("UnsupportedProtocolVersion", None)));
                }
            };

        info.protocol_version = negotiated_protocol_version;
        std::future::ready(Ok(info))
    }
    fn call_tool(
        &self,
        request: CallToolRequestParam,
    ) -> impl Future<Output = Result<CallToolResult, Error>> {
        std::future::ready(Err(Error::method_not_found::<CallToolRequestMethod>()))
    }
    fn list_tools(
        &self,
        request: Option<PaginatedRequestParam>,
    ) -> impl Future<Output = Result<ListToolsResult, Error>> {
        std::future::ready(Ok(ListToolsResult::default()))
    }
    fn on_initialized(&self) -> impl Future<Output = ()> {
        std::future::ready(())
    }
    fn get_info(&self) -> ServerInfo {
        ServerInfo::default()
    }
}
