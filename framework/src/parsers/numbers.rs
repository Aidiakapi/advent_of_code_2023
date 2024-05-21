use std::marker::PhantomData;

use super::*;

macro_rules! impl_uint_parsing {
    ($kind:tt) => {
        impl $crate::parsers::numbers::IsParsableNumber for $kind {}
        impl<'s> $crate::parsers::Parser<'s> for NumberParser<$kind> {
            type Output = $kind;

            fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
                let first_char = *input.first().ok_or((ParseError::EmptyInput, input))?;
                if !matches!(first_char, b'0'..=b'9') {
                    return Err((ParseError::ExpectedDigit, input));
                }

                let mut remainder = &input[1..];

                let mut x = (first_char as $kind) - (b'0' as $kind);
                loop {
                    let next_digit = match remainder.first() {
                        Some(&c @ b'0'..=b'9') => (c as $kind) - (b'0' as $kind),
                        _ => break,
                    };
                    x = x
                        .checked_mul(10)
                        .and_then(|x| x.checked_add(next_digit))
                        .ok_or((ParseError::Overflow, input))?;
                    remainder = &remainder[1..];
                }

                Ok((x, remainder))
            }
        }
    };
}

macro_rules! impl_sint_parsing {
    ($kind:tt, $unsigned:tt) => {
        /// Parses an integer. Allows an optional + or - at the start to
        /// indicate a sign.
        impl $crate::parsers::numbers::IsParsableNumber for $kind {}
        impl<'s> $crate::parsers::Parser<'s> for NumberParser<$kind> {
            type Output = $kind;

            fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
                let (is_negative, remainder) = match input.first() {
                    Some(&b'-') => (true, &input[1..]),
                    Some(&b'+') => (false, &input[1..]),
                    _ => (false, input),
                };
                let (number, remainder) =
                    $crate::parsers::number::<$unsigned>().parse(remainder)?;
                const MAX: $unsigned = $kind::MAX as $unsigned;
                const MAX_PLUS_ONE: $unsigned = MAX + 1;
                let number = match (number, is_negative) {
                    (0..=MAX, false) => number as $kind,
                    (0..=MAX, true) => -(number as $kind),
                    (MAX_PLUS_ONE, true) => $kind::MIN,
                    _ => return Err((ParseError::Overflow, input)),
                };
                Ok((number, remainder))
            }
        }
    };
}

impl_uint_parsing!(u8);
impl_uint_parsing!(u16);
impl_uint_parsing!(u32);
impl_uint_parsing!(u64);
impl_uint_parsing!(u128);
impl_uint_parsing!(usize);

impl_sint_parsing!(i8, u8);
impl_sint_parsing!(i16, u16);
impl_sint_parsing!(i32, u32);
impl_sint_parsing!(i64, u64);
impl_sint_parsing!(i128, u128);
impl_sint_parsing!(isize, usize);

pub trait IsParsableNumber {}
#[derive(Debug, Clone, Copy)]
pub struct NumberParser<T: IsParsableNumber>(PhantomData<T>);

pub const fn number<T: IsParsableNumber>() -> NumberParser<T> {
    NumberParser(PhantomData)
}
