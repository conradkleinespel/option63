use o63::carddav::error::ProxyError;
use o63::config::ConfigError;
use o63::vcard::parser::ParseError;
use o63::webdav::error::DavError;

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("config error: {0}")]
    Config(#[from] ConfigError),
    #[error("carddav proxy encountered an error: {0}")]
    Proxy(#[from] ProxyError),
    #[error("failed to parse vcard: {0}")]
    VCardParse(#[from] ParseError),
    #[error("dav error: {0}")]
    Dav(#[from] DavError),
    #[error("{0}")]
    InvalidInput(String),
}
