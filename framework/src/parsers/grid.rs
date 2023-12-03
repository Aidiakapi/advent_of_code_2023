use super::Parser;
use crate::grid::{Grid, GridBuilder};
use std::marker::PhantomData;

pub fn grid<G, PC, PN>(cell: PC, line_separator: PN) -> GridParser<G, PC, PN> {
    GridParser {
        cell,
        line_separator,
        _g: PhantomData,
    }
}

#[derive(Clone, Copy)]
pub struct GridParser<G, PC, PN> {
    cell: PC,
    line_separator: PN,
    _g: PhantomData<G>,
}

impl<'s, G, T, PC, PN, NO> Parser<'s> for GridParser<G, PC, PN>
where
    G: Grid<T> + 's,
    PC: Parser<'s, Output = T>,
    PN: Parser<'s, Output = NO>,
{
    type Output = G;

    fn parse(&self, input: &'s [u8]) -> super::ParseResult<'s, Self::Output> {
        let mut builder = G::Builder::new();

        let (cell, mut remainder) = self.cell.parse(input)?;
        builder.push_cell(cell).map_err(|e| (e, input))?;

        let mut any_cells_parsed = true;
        loop {
            while let Ok((cell, new_remainder)) = self.cell.parse(remainder) {
                any_cells_parsed = true;
                builder.push_cell(cell).map_err(|e| (e, input))?;
                remainder = new_remainder;
            }

            let before_newline = remainder;
            let mut parse_next_line = || {
                self.line_separator
                    .parse(remainder)
                    .map(|(_, new_remainder)| {
                        remainder = new_remainder;
                    })
                    .is_ok()
            };
            if !any_cells_parsed || !parse_next_line() {
                return builder
                    .finish()
                    .map(|v| (v, before_newline))
                    .map_err(|e| (e, input));
            }
            builder.advance_next_line().map_err(|e| (e, input))?;
            any_cells_parsed = false;
        }
    }
}
