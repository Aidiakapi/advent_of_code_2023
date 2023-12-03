use super::*;

pub trait ParserCombiExt<'s>: Sized + Parser<'s> {
    /// Evaluates two parsers sequentially, and returns a tuple of their outputs
    fn and<P2: Parser<'s>>(self, parser: P2) -> And<Self, P2> {
        And(self, parser)
    }
    /// Evaluates two parsers sequentially, returns the output of the second
    fn then<P2: Parser<'s>>(self, parser: P2) -> Then<Self, P2> {
        Then(self, parser)
    }
    /// Evaluates two parsers sequentially, returns the output of the first
    fn trailed<P2: Parser<'s>>(self, parser: P2) -> Trailed<Self, P2> {
        Trailed(self, parser)
    }

    /// Attempts the first parser, and upon failure attempts the second parser
    fn or<P2: Parser<'s, Output = Self::Output>>(self, parser: P2) -> Or<Self, P2> {
        Or(self, parser)
    }

    /// Takes the output of one parser, and transforms it into another type
    fn map<T, F: Fn(Self::Output) -> T>(self, f: F) -> Map<Self, F> {
        Map(self, f)
    }
    /// Takes the output of one parser, and transforms it into a `Result` of another type
    fn map_res<T, F: Fn(Self::Output) -> Result<T, ParseError>>(self, f: F) -> MapRes<Self, F> {
        MapRes(self, f)
    }

    /// Attempts to apply this parser, upon success, wraps the value in Some,
    /// upon failure, succeeds with value None and no input consumed.
    fn opt(self) -> Opt<Self> {
        Opt(self)
    }
}

impl<'s, P1: Parser<'s>> ParserCombiExt<'s> for P1 {}

#[derive(Debug, Clone, Copy)]
pub struct And<P1, P2>(P1, P2);
impl<'s, P1: Parser<'s>, P2: Parser<'s>> Parser<'s> for And<P1, P2> {
    type Output = (P1::Output, P2::Output);

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
        let (o1, remainder) = self.0.parse(input)?;
        let (o2, remainder) = self.1.parse(remainder)?;
        Ok(((o1, o2), remainder))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Then<P1, P2>(P1, P2);
impl<'s, P1: Parser<'s>, P2: Parser<'s>> Parser<'s> for Then<P1, P2> {
    type Output = P2::Output;

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
        let (_, remainder) = self.0.parse(input)?;
        self.1.parse(remainder)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Trailed<P1, P2>(P1, P2);
impl<'s, P1: Parser<'s>, P2: Parser<'s>> Parser<'s> for Trailed<P1, P2> {
    type Output = P1::Output;

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
        let (output, remainder) = self.0.parse(input)?;
        let (_, remainder) = self.1.parse(remainder)?;
        Ok((output, remainder))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Or<P1, P2>(P1, P2);
impl<'s, P1: Parser<'s>, P2: Parser<'s, Output = P1::Output>> Parser<'s> for Or<P1, P2> {
    type Output = P1::Output;

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
        self.0.parse(input).or_else(|_| self.1.parse(input))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Map<P, F>(P, F);
impl<'s, P: Parser<'s>, T: 's, F: Fn(P::Output) -> T> Parser<'s> for Map<P, F> {
    type Output = T;

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, T> {
        self.0
            .parse(input)
            .map(|(value, remainder)| ((self.1)(value), remainder))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MapRes<P, F>(P, F);
impl<'s, P: Parser<'s>, T: 's, F: Fn(P::Output) -> Result<T, ParseError>> Parser<'s>
    for MapRes<P, F>
{
    type Output = T;

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, T> {
        self.0
            .parse(input)
            .and_then(|(value, remainder)| match (self.1)(value) {
                Ok(value) => Ok((value, remainder)),
                Err(err) => Err((err, input)),
            })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Opt<P>(P);
impl<'s, P: Parser<'s>> Parser<'s> for Opt<P> {
    type Output = Option<P::Output>;

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
        Ok(match self.0.parse(input) {
            Ok((value, remainder)) => (Some(value), remainder),
            _ => (None, input),
        })
    }
}
