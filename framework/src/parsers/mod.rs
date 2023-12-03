mod combi;
mod common;
pub mod error;
mod grid;
mod multi;
pub mod numbers;
mod util;

pub use combi::ParserCombiExt;
pub use common::{any, digit, pattern, token};
pub use error::{ParseError, ParseResult};
pub use grid::grid;
pub use multi::{take_while, ParserMultiExt};
pub use numbers::number;
pub use util::AStrExt;

pub trait Parser<'s> {
    type Output: 's;
    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output>;
}

pub trait Execute<'s, T> {
    fn execute(&self, input: &'s [u8]) -> crate::result::Result<T>;
}

impl<'s, P: Parser<'s, Output = T>, T> Execute<'s, T> for P {
    fn execute(&self, input: &'s [u8]) -> crate::result::Result<T> {
        Err(match self.parse(input) {
            Ok((x, [] | [b'\n'])) => return Ok(x),
            Ok((_, remainder)) => {
                ParseError::InputNotConsumed(String::from_utf8_lossy(remainder).into_owned())
            }
            Err((e, remainder)) => ParseError::WithRemainder(
                Box::new(e),
                String::from_utf8_lossy(remainder).into_owned(),
            ),
        }
        .into())
    }
}

impl<'s, F, O> Parser<'s> for F
where
    F: Fn(&'s [u8]) -> ParseResult<'s, O>,
    O: 's,
{
    type Output = O;
    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
        (self)(input)
    }
}
