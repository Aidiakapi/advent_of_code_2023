use thiserror::Error;

pub type ParseResult<'s, T> = Result<(T, &'s [u8]), (ParseError, &'s [u8])>;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseError {
    #[error("input not fully consumed, remainder: {0:?}")]
    InputNotConsumed(String),
    #[error("{0}, remainder: {1:?}")]
    WithRemainder(Box<ParseError>, String),
    #[error("empty input")]
    EmptyInput,
    #[error("expected a digit")]
    ExpectedDigit,
    #[error("overflow")]
    Overflow,
    #[error("token does not match")]
    TokenDoesNotMatch,
    #[error("unexpected char")]
    UnexpectedChar,
    #[error("filter does not match")]
    FilterDoesNotMatch,
    #[error("a cell was parsed that is beyond established width of the grid")]
    GridCellAfterEndOfRowReached,
    #[error("a row was incomplete")]
    GridIncompleteRow,
    #[error("{0}")]
    Custom(&'static str),
}
