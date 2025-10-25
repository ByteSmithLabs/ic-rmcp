//! ic-rmcp: A lightweight Rust SDK for building Model Context Protocol (MCP) servers on the Internet Computer (IC).
//!
//! This crate focuses on the core MCP tools capability over the IC Streamable HTTP transport.
//!
//! Quick start:
//! - Implement the [`Handler`] trait for your server logic
//! - Expose your canister's HTTP endpoints and call [`Server::handle`] or [`Server::handle_with_oauth`]
//! - Describe tools using [`schema_for_type`] and respond with types from the re-exported [`model`]
//!
//! See the README for end-to-end examples and guidance.

mod handler;
/// Per-request context and the main trait you implement to define your MCP server behavior.
pub use handler::{Context, Handler};

mod server;
/// Entry points for handling Streamable HTTP requests to your MCP server.
pub use server::Server;

mod state;

/// OAuth configuration types for protecting your MCP server and advertising metadata.
pub use handler::oauth::{IssuerConfig, OAuthConfig};
/// Helper to generate a JSON Schema for a Rust type to describe tool parameters.
pub use rmcp::handler::server::tool::schema_for_type;
/// Re-export of MCP model types (requests, responses, capabilities, etc.).
pub use rmcp::model;
/// Common error type returned by handler methods.
pub use rmcp::Error;
