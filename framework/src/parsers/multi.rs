use std::{fmt::Debug, marker::PhantomData};

use super::*;

pub trait ParserMultiExt<'s>: Sized + Parser<'s> {
    /// Repeatedly applies the parser, interspersing applications of `separator`.
    /// Fails if parser cannot be applied at least once.
    fn sep_by<S, C: Default + Extend<Self::Output>>(self, separator: S) -> SepBy<Self, S, C>
    where
        S: Parser<'s>,
    {
        SepBy {
            parser: self,
            separator,
            _collection: PhantomData,
        }
    }

    /// Repeatedly applies the parser, repeatedly invoking `func` with the
    /// output value, updating the accumulator which starts out as `initial`.
    fn fold<A, F>(self, initial: A, func: F) -> Fold<Self, A, F>
    where
        A: Clone,
        F: Fn(A, Self::Output) -> A,
    {
        Fold {
            parser: self,
            initial,
            func,
        }
    }

    /// Repeatedly applies the parser, repeatedly invoking `func` with the
    /// output value, updating the accumulator which starts out as `initial`.
    fn fold_mut<A, F>(self, initial: A, func: F) -> FoldMut<Self, A, F>
    where
        A: Clone,
        F: Fn(&mut A, Self::Output),
    {
        FoldMut {
            parser: self,
            initial,
            func,
        }
    }

    /// Repeatedly applies the parser, until failure, returning the last
    /// successful output, or an error if it fails to apply even once.
    fn repeat(self) -> Repeat<Self> {
        Repeat { parser: self }
    }

    /// Repeatedly applies the parser, until failure, returning a collection
    /// of all successfully applied values.
    fn repeat_into<C: Default + Extend<Self::Output>>(self) -> RepeatInto<Self, C> {
        RepeatInto {
            parser: self,
            _collection: PhantomData,
        }
    }

    fn many_n<const N: usize>(self) -> Many<Self, N> {
        Many { parser: self }
    }
}

impl<'s, P: Parser<'s>> ParserMultiExt<'s> for P {}

#[derive(Debug, Clone, Copy)]
pub struct TakeWhile<C, F>(C, F);
impl<'s, C, F> Parser<'s> for TakeWhile<C, F>
where
    C: Clone,
    F: Fn(&mut C, u8) -> bool,
{
    type Output = &'s [u8];

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
        let mut index = 0;
        let mut ctx = self.0.clone();
        loop {
            match input.get(index) {
                Some(&c) if (self.1)(&mut ctx, c) => index += 1,
                _ => break,
            }
        }
        if index == 0 {
            Err((ParseError::UnexpectedChar, input))
        } else {
            Ok((&input[0..index], &input[index..]))
        }
    }
}
pub fn take_while<C, F>(ctx: C, f: F) -> TakeWhile<C, F>
where
    C: Clone,
    F: Fn(&mut C, u8) -> bool,
{
    TakeWhile(ctx, f)
}

#[derive(Debug, Clone, Copy)]
pub struct SepBy<P, S, C> {
    parser: P,
    separator: S,
    _collection: PhantomData<C>,
}

#[derive(Debug, Clone, Copy)]
pub struct Fold<P, A, F> {
    parser: P,
    initial: A,
    func: F,
}

#[derive(Debug, Clone, Copy)]
pub struct FoldMut<P, A, F> {
    parser: P,
    initial: A,
    func: F,
}

#[derive(Debug, Clone, Copy)]
pub struct Repeat<P> {
    parser: P,
}

#[derive(Debug, Clone, Copy)]
pub struct RepeatInto<P, C> {
    parser: P,
    _collection: PhantomData<C>,
}

#[derive(Debug, Clone, Copy)]
pub struct Many<P, const N: usize> {
    parser: P,
}

impl<'s, P, S, C> Parser<'s> for SepBy<P, S, C>
where
    P: Parser<'s>,
    S: Parser<'s>,
    C: 's + Default + Extend<P::Output>,
{
    type Output = C;

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
        let (element, mut remainder) = self.parser.parse(input)?;
        let mut elements = C::default();
        elements.extend(Some(element));
        loop {
            let after_sep = match self.separator.parse(remainder) {
                Ok((_, after_sep)) => after_sep,
                Err(_) => return Ok((elements, remainder)),
            };
            match self.parser.parse(after_sep) {
                Ok((element, after_value)) => {
                    remainder = after_value;
                    elements.extend(Some(element));
                }
                Err(_) => return Ok((elements, remainder)),
            };
        }
    }
}

impl<'s, P, A, F> Parser<'s> for Fold<P, A, F>
where
    P: Parser<'s>,
    A: 's + Clone,
    F: Fn(A, P::Output) -> A,
{
    type Output = A;

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
        let mut accumulator = self.initial.clone();
        let mut remainder = input;
        while let Ok((value, new_remainder)) = self.parser.parse(remainder) {
            accumulator = (self.func)(accumulator, value);
            remainder = new_remainder;
        }
        Ok((accumulator, remainder))
    }
}

impl<'s, P, A, F> Parser<'s> for FoldMut<P, A, F>
where
    P: Parser<'s>,
    A: 's + Clone,
    F: Fn(&mut A, P::Output),
{
    type Output = A;

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
        let mut accumulator = self.initial.clone();
        let mut remainder = input;
        while let Ok((value, new_remainder)) = self.parser.parse(remainder) {
            (self.func)(&mut accumulator, value);
            remainder = new_remainder;
        }
        Ok((accumulator, remainder))
    }
}

impl<'s, P> Parser<'s> for Repeat<P>
where
    P: Parser<'s>,
{
    type Output = P::Output;

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
        let (mut last_value, mut remainder) = self.parser.parse(input)?;
        while let Ok((value, new_remainder)) = self.parser.parse(remainder) {
            last_value = value;
            remainder = new_remainder;
        }
        Ok((last_value, remainder))
    }
}

impl<'s, P: Parser<'s>, C: 's + Default + Extend<P::Output>> Parser<'s> for RepeatInto<P, C> {
    type Output = C;

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
        let mut c = C::default();

        let (first_value, mut remainder) = self.parser.parse(input)?;
        c.extend(Some(first_value));
        while let Ok((value, new_remainder)) = self.parser.parse(remainder) {
            c.extend(Some(value));
            remainder = new_remainder;
        }
        Ok((c, remainder))
    }
}

impl<'s, P: Parser<'s>, const N: usize> Parser<'s> for Many<P, N> {
    type Output = [P::Output; N];

    fn parse(&self, input: &'s [u8]) -> ParseResult<'s, Self::Output> {
        let mut remainder = input;
        crate::util::init_array(|_| {
            let (result, new_remainder) = self.parser.parse(remainder)?;
            remainder = new_remainder;
            Ok(result)
        })
        .map(|array| (array, remainder))
        .map_err(|(e, _)| (e, input))
    }
}
