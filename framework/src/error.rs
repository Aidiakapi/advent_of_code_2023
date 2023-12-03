use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("network error: {0}")]
    NetworkError(#[from] Box<ureq::Error>),
    #[error("parse error: {0}")]
    ParseError(#[from] crate::parsers::ParseError),
    #[error("not yet implemented")]
    NotImplemented,
    #[error("invalid input '{0}'")]
    InvalidInput(&'static str),
    #[error("no solution")]
    NoSolution,
}
