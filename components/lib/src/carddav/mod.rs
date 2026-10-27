//! CardDAV module grouping client operations, server interfaces, protocol definitions, and error handling.

pub mod client;
pub mod error;
pub mod protocol;
pub mod server;
pub mod server_handler_in_memory;
pub mod server_handler_proxy;
