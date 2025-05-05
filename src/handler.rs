use crate::{error::Error, model::*, service::Service};

impl<H: ServerHandler> Service for H {
    async fn handle_request(&self, request: ClientRequest) -> Result<ServerResult, Error> {
        match request {
            ClientRequest::InitializeRequest(request) => self
                .initialize(request.params)
                .await
                .map(ServerResult::InitializeResult),
            ClientRequest::PingRequest(_request) => self.ping().await.map(ServerResult::empty),
            ClientRequest::CompleteRequest(request) => self
                .complete(request.params)
                .await
                .map(ServerResult::CompleteResult),
            ClientRequest::SetLevelRequest(_) => {
                Err(Error::method_not_found::<SetLevelRequestMethod>())
                    .map(ServerResult::CompleteResult)
            }
            ClientRequest::GetPromptRequest(request) => self
                .get_prompt(request.params)
                .await
                .map(ServerResult::GetPromptResult),
            ClientRequest::ListPromptsRequest(request) => self
                .list_prompts(request.params)
                .await
                .map(ServerResult::ListPromptsResult),
            ClientRequest::ListResourcesRequest(request) => self
                .list_resources(request.params)
                .await
                .map(ServerResult::ListResourcesResult),
            ClientRequest::ListResourceTemplatesRequest(request) => self
                .list_resource_templates(request.params)
                .await
                .map(ServerResult::ListResourceTemplatesResult),
            ClientRequest::ReadResourceRequest(request) => self
                .read_resource(request.params)
                .await
                .map(ServerResult::ReadResourceResult),
            ClientRequest::SubscribeRequest(_) => {
                Err(Error::method_not_found::<SubscribeRequestMethod>()).map(ServerResult::empty)
            }
            ClientRequest::UnsubscribeRequest(_) => {
                Err(Error::method_not_found::<UnsubscribeRequestMethod>()).map(ServerResult::empty)
            }
            ClientRequest::CallToolRequest(request) => self
                .call_tool(request.params)
                .await
                .map(ServerResult::CallToolResult),
            ClientRequest::ListToolsRequest(request) => self
                .list_tools(request.params)
                .await
                .map(ServerResult::ListToolsResult),
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
    // handle requests
    fn initialize(
        &self,
        request: InitializeRequestParam,
    ) -> impl Future<Output = Result<InitializeResult, Error>> {
        std::future::ready(Ok(self.get_info()))
    }
    fn complete(
        &self,
        request: CompleteRequestParam,
    ) -> impl Future<Output = Result<CompleteResult, Error>> {
        std::future::ready(Err(Error::method_not_found::<CompleteRequestMethod>()))
    }
    fn get_prompt(
        &self,
        request: GetPromptRequestParam,
    ) -> impl Future<Output = Result<GetPromptResult, Error>> {
        std::future::ready(Err(Error::method_not_found::<GetPromptRequestMethod>()))
    }
    fn list_prompts(
        &self,
        request: Option<PaginatedRequestParam>,
    ) -> impl Future<Output = Result<ListPromptsResult, Error>> {
        std::future::ready(Ok(ListPromptsResult::default()))
    }
    fn list_resources(
        &self,
        request: Option<PaginatedRequestParam>,
    ) -> impl Future<Output = Result<ListResourcesResult, Error>> {
        std::future::ready(Ok(ListResourcesResult::default()))
    }
    fn list_resource_templates(
        &self,
        request: Option<PaginatedRequestParam>,
    ) -> impl Future<Output = Result<ListResourceTemplatesResult, Error>> {
        std::future::ready(Ok(ListResourceTemplatesResult::default()))
    }
    fn read_resource(
        &self,
        request: ReadResourceRequestParam,
    ) -> impl Future<Output = Result<ReadResourceResult, Error>> {
        std::future::ready(Err(Error::method_not_found::<ReadResourceRequestMethod>()))
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
