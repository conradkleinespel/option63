//! WebDAV resource representations encapsulating media type, entity tags, and binary content.

/// A DAV resource representation shared by the client (GET results / PUT
/// payloads) and the server handlers.
#[derive(Clone, Debug, PartialEq)]
pub struct Resource {
    pub content_type: String,
    /// ETag as returned by the server (e.g. `"abc"`), or what the client
    /// should send for conditional requests.
    /// TODO: Support proper ETag (RFC 9110) parsing/generation, including weak/strong?
    pub etag: Option<String>,
    pub body: Vec<u8>,
}
