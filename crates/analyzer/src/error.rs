use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalyzerError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("URL parsing failed: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("DNS resolution failed: {0}")]
    DnsError(String),

    #[error("TLS handshake failed: {0}")]
    TlsError(String),

    #[error("Invalid domain: {0}")]
    InvalidDomain(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Analysis error: {0}")]
    Analysis(String),
}

pub type Result<T> = std::result::Result<T, AnalyzerError>;
