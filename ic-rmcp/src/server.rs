use crate::handler::oauth::OAuthConfig;
use ic_http_certification::{HeaderField, HttpRequest, HttpResponse};
use std::future::Future;

/// Entry points for serving MCP over the IC Streamable HTTP interface.
///
/// This trait is blanket-implemented for all [`Handler`](crate::Handler) implementors.
/// Use one of the methods below from inside your canister's `http_request`/`http_request_update`
/// to process MCP JSON-RPC messages posted to `/mcp`.
pub trait Server {
    /// Handle a request using a caller-provided authorization predicate.
    ///
    /// - If `auth(headers)` returns `false`, a `401 Unauthorized` response is returned.
    /// - Otherwise, the request is processed. Only `POST` requests to paths ending with `/mcp`
    ///   are accepted; other methods or paths yield a `404` with a helpful message.
    ///
    /// Typical usage is API-key or custom header checks.
    fn handle(
        &self,
        req: &HttpRequest,
        auth: impl Fn(&[HeaderField]) -> bool,
    ) -> impl Future<Output = HttpResponse<'_>>;
    /// Handle a request with OAuth protection and metadata support.
    ///
    /// Behavior:
    /// - Serves resource metadata when the client performs `GET` on the metadata URL path.
    /// - Requires a `Bearer` token on protected endpoints; missing/invalid tokens result in `401`
    ///   with a `WWW-Authenticate` challenge referencing the provided metadata URL.
    /// - On success, forwards to the core MCP handler.
    fn handle_with_oauth(
        &self,
        req: &HttpRequest,
        cfg: OAuthConfig,
    ) -> impl Future<Output = HttpResponse<'_>>;
}
