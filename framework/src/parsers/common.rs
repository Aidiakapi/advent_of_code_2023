pub use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Digit;
pub const fn digit() -> Digit {
    Digit
}
impl<'s> Parser<'s> for Digit {
    type Output = u8;

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
        match input.first().cloned() {
            None => Err((ParseError::EmptyInput, input)),
            Some(d @ b'0'..=b'9') => Ok((d - b'0', &input[1..])),
            Some(_) => Err((ParseError::ExpectedDigit, input)),
        }
    }
}

pub macro pattern($p:pat) {{
    #[derive(Debug, Clone, Copy)]
    struct PatternParser;
    impl<'s> $crate::parsers::Parser<'s> for PatternParser {
        type Output = u8;

        fn parse(&self, input: &'s [u8]) -> $crate::parsers::ParseResult<'s, Self::Output> {
            match input.first().cloned() {
                None => Err((ParseError::EmptyInput, input)),
                Some(v @ $p) => Ok((v, &input[1..])),
                Some(_) => Err((ParseError::UnexpectedChar, input)),
            }
        }
    }

    PatternParser
}}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Token<T> {
    value: T,
}

impl<'s> Parser<'s> for Token<u8> {
    type Output = ();
    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, ()> {
        if let Some(&c) = input.first() {
            if c == self.value {
                return Ok(((), &input[1..]));
            }
        }
        Err((ParseError::TokenDoesNotMatch, input))
    }
}

impl<'s, T: 's + Clone> Parser<'s> for Token<(u8, T)> {
    type Output = T;
    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, T> {
        if let Some(&c) = input.first() {
            if c == self.value.0 {
                return Ok((self.value.1.clone(), &input[1..]));
            }
        }
        Err((ParseError::TokenDoesNotMatch, input))
    }
}

impl<'s, 't: 's> Parser<'s> for Token<&'t [u8]> {
    type Output = ();
    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, ()> {
        if input.starts_with(self.value) {
            Ok(((), &input[self.value.len()..]))
        } else {
            Err((ParseError::TokenDoesNotMatch, input))
        }
    }
}

impl<'s, 't: 's, T: 's + Clone> Parser<'s> for Token<(&'t [u8], T)> {
    type Output = T;
    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, T> {
        if input.starts_with(self.value.0) {
            Ok((self.value.1.clone(), &input[self.value.0.len()..]))
        } else {
            Err((ParseError::TokenDoesNotMatch, input))
        }
    }
}

impl<'s, 't: 's, const N: usize> Parser<'s> for Token<&'t [u8; N]> {
    type Output = ();
    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, ()> {
        if input.starts_with(self.value) {
            Ok(((), &input[self.value.len()..]))
        } else {
            Err((ParseError::TokenDoesNotMatch, input))
        }
    }
}

impl<'s, 't: 's, T: 's + Clone, const N: usize> Parser<'s> for Token<(&'t [u8; N], T)> {
    type Output = T;
    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, T> {
        if input.starts_with(self.value.0) {
            Ok((self.value.1.clone(), &input[self.value.0.len()..]))
        } else {
            Err((ParseError::TokenDoesNotMatch, input))
        }
    }
}

pub fn token<T>(token: T) -> Token<T> {
    Token { value: token }
}

#[derive(Debug, Clone, Copy)]
pub struct Any;
impl<'s> Parser<'s> for Any {
    type Output = u8;

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
        match input.first() {
            Some(&c) => Ok((c, &input[1..])),
            None => Err((ParseError::EmptyInput, input)),
        }
    }
}
pub fn any() -> Any {
    Any
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn unsigned_numbers() {
        assert_eq!( Ok((0,                         &b""    [..])), number::<u8>().parse(b"0"    ));
        assert_eq!( Ok((128,                       &b""    [..])), number::<u8>().parse(b"128"  ));
        assert_eq!( Ok((255,                       &b""    [..])), number::<u8>().parse(b"255"  ));
        assert_eq!( Ok((10,                        &b"abc" [..])), number::<u8>().parse(b"10abc"));
        assert_eq!(Err((ParseError::Overflow,      &b"300" [..])), number::<u8>().parse(b"300"  ));
        assert_eq!(Err((ParseError::Overflow,      &b"256a"[..])), number::<u8>().parse(b"256a" ));
        assert_eq!(Err((ParseError::EmptyInput,    &b""    [..])), number::<u8>().parse(b""     ));
        assert_eq!(Err((ParseError::ExpectedDigit, &b"-1"  [..])), number::<u8>().parse(b"-1"   ));
    }

    #[test]
    #[rustfmt::skip]
    fn signed_numbers() {
        assert_eq!( Ok((0,                         &b""    [..])), number::<i8>().parse(b"0"    ));
        assert_eq!( Ok((127,                       &b""    [..])), number::<i8>().parse(b"127"  ));
        assert_eq!( Ok((127,                       &b""    [..])), number::<i8>().parse(b"+127" ));
        assert_eq!( Ok((-128,                      &b""    [..])), number::<i8>().parse(b"-128" ));
        assert_eq!( Ok((10,                        &b"abc" [..])), number::<i8>().parse(b"10abc"));
        assert_eq!(Err((ParseError::Overflow,      &b"+128"[..])), number::<i8>().parse(b"+128" ));
        assert_eq!(Err((ParseError::Overflow,      &b"-129"[..])), number::<i8>().parse(b"-129" ));
        assert_eq!(Err((ParseError::EmptyInput,    &b""    [..])), number::<i8>().parse(b""     ));
    }
}
