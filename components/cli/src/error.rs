use o63::carddav::error::ProxyError;
use o63::vcard::parser::ParseError;

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("carddav proxy encountered an error: {0}")]
    Proxy(#[from] ProxyError),
    #[error("failed to parse vcard: {0}")]
    VCardParse(#[from] ParseError),
    #[error("{0}")]
    InvalidInput(String),
}
