mod handler;
pub use handler::Handler;

mod server;
pub use server::Server;

pub use rmcp::handler::server::tool::schema_for_type;
pub use rmcp::model;
pub use rmcp::Error;
