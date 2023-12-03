pub trait IteratorExt: Iterator {
    /// Finds the item that is not the same as all the others in the collection
    fn find_distinct(&mut self) -> DistinctResult<Self::Item>
    where
        Self::Item: Eq,
    {
        use DistinctResult::*;
        let first = match self.next() {
            Some(v) => v,
            None => return TooFewElements,
        };
        let second = match self.next() {
            Some(v) => v,
            None => return TooFewElements,
        };
        let (mut index, common, mut distinct) = if first == second {
            (1, first, None)
        } else {
            let third = match self.next() {
                Some(v) => v,
                None => return TooFewElements,
            };
            if first == third {
                (2, first, Some((1, second)))
            } else if second == third {
                (2, second, Some((0, first)))
            } else {
                return MultipleDistinct;
            }
        };

        for value in self {
            index += 1;
            if value == common {
                continue;
            }
            if distinct.is_some() {
                return MultipleDistinct;
            }
            distinct = Some((index, value));
        }

        match distinct {
            Some((index, distinct)) => SingleDistinct(Distinct {
                index,
                common,
                distinct,
            }),
            None => Unique(common),
        }
    }

    fn next_array<const N: usize>(&mut self) -> Option<[Self::Item; N]> {
        crate::util::init_array(|_| self.next().ok_or(())).ok()
    }

    fn collect_array<const N: usize>(&mut self) -> Option<[Self::Item; N]> {
        let result = self.next_array()?;
        if self.next().is_some() {
            None
        } else {
            Some(result)
        }
    }
}

impl<T: Iterator + ?Sized> IteratorExt for T {}

pub trait SizedIteratorExt: Iterator + Sized {
    fn take_while_map<F>(self, map_fn: F) -> TakeWhileNext<Self, F> {
        TakeWhileNext { iter: self, map_fn }
    }
}

impl<T: Iterator + Sized> SizedIteratorExt for T {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DistinctResult<T> {
    /// There less than two elements, or exactly two distinct elements.
    TooFewElements,
    /// All values are the same.
    Unique(T),
    /// There is a single distinct value.
    SingleDistinct(Distinct<T>),
    /// There are multiple distinct values.
    MultipleDistinct,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Distinct<T> {
    pub index: usize,
    pub common: T,
    pub distinct: T,
}

pub struct TakeWhileNext<I, F> {
    iter: I,
    map_fn: F,
}

impl<I, F, O> Iterator for TakeWhileNext<I, F>
where
    I: Iterator,
    F: FnMut(I::Item) -> Option<O>,
{
    type Item = O;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().and_then(&mut self.map_fn)
    }
}

pub trait LendingIterator {
    type Item<'e> where Self: 'e;
    fn next(&mut self) -> Option<Self::Item<'_>>;
}
