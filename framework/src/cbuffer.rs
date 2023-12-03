use num::{Bounded, CheckedAdd, Integer, Zero};
use std::{fmt::Debug, ops::Range};

pub trait Indexer = Integer + Bounded + Clone;
pub trait Value = Clone + Eq;

pub struct CBuffer<I: Indexer, V: Value> {
    data: Vec<(I, V)>,
}

pub struct CBufferMutator<I: Indexer, V: Value> {
    data: Vec<(I, V)>,
}

pub struct CBufferRangeIter<'c, I: Indexer, V: Value> {
    windows: std::slice::ArrayWindows<'c, (I, V), 2>,
    last: Option<&'c (I, V)>,
}

impl<'c, I: Indexer, V: Value> Iterator for CBufferRangeIter<'c, I, V> {
    type Item = (Range<I>, &'c V);

    fn next(&mut self) -> Option<Self::Item> {
        self.windows
            .next()
            .map(|[from, to]| (from.0.clone()..to.0.clone(), &from.1))
            .or_else(|| {
                self.last
                    .take()
                    .map(|last| (last.0.clone()..I::max_value(), &last.1))
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.windows.size_hint().0 + 1;
        (size, Some(size))
    }
}

impl<I: Indexer + Debug, V: Value + Debug> CBuffer<I, V> {
    pub fn mutator() -> CBufferMutator<I, V> {
        CBufferMutator { data: Vec::new() }
    }

    pub fn new(initial_value: V) -> Self {
        Self {
            data: vec![(I::min_value(), initial_value)],
        }
    }

    pub fn ranges(&self) -> CBufferRangeIter<'_, I, V> {
        CBufferRangeIter {
            windows: self.data.array_windows(),
            last: self.data.last(),
        }
    }

    pub fn set(&mut self, range: Range<I>, value: V, mutator: &mut CBufferMutator<I, V>) {
        if range.start >= range.end {
            return;
        }

        debug_assert!(mutator.data.is_empty());
        mutator.data.reserve(self.data.len() + 2);

        let mut data = self.data.drain(..).peekable();
        // Insert all entries that come **before** the start point
        while let Some(entry) = data.next_if(|item| item.0 < range.start) {
            mutator.data.push(entry);
        }

        // Insert new entry if it's not the same as it was beforehand
        let range_started_the_same =
            matches!(mutator.data.last(), Some((_, last_value)) if last_value == &value);
        if !range_started_the_same {
            mutator.data.push((range.start.clone(), value));
        }

        // Skip all entries until the end-point is reached
        let mut latest_value = None;
        while let Some(entry) = data.next_if(|entry| entry.0 < range.end) {
            latest_value = Some(entry.1);
        }

        // If there was an entry at the end-point, it should be preserved
        // otherwise, we have to insert a new entry, that restored is back to
        // the value before this range started.
        if let Some(entry) = data.next_if(|entry| entry.0 == range.end) {
            Self::push_if_different(&mut mutator.data, entry.0, entry.1);
        } else if let Some(latest_value) = latest_value {
            // In this case, the latest value was started at a point at or after
            // the starting point, and we can simply restore it to that values.
            Self::push_if_different(&mut mutator.data, range.end, latest_value);
        } else if !range_started_the_same {
            // We did not have any new values inserted >= range.start, therefore
            // whatever the value was **before** we opened the range, is the
            // value we want it to have again.
            let index = mutator.data.len() - 2;
            let latest_value = mutator.data[index].1.clone();
            Self::push_if_different(&mut mutator.data, range.end, latest_value);
        }

        if let Some(entry) = data.next() {
            // The first item > range.end could now have ended up with the same
            // value as the value before our newly inserted range even started.
            // So it should be conditionally pushed.
            // The rest can remain as-is.
            Self::push_if_different(&mut mutator.data, entry.0, entry.1);
            mutator.data.extend(data);
        } else {
            std::mem::drop(data);
        }

        std::mem::swap(&mut mutator.data, &mut self.data);
        self.verify();
    }

    fn push_if_different(collection: &mut Vec<(I, V)>, position: I, value: V) {
        if let Some(last) = collection.last() && last.1 == value {
            return;
        }
        collection.push((position, value));
    }

    #[cfg(debug_assertions)]
    fn verify(&self) {
        assert!(!self.data.is_empty());
        assert!(self.data[0].0 == I::min_value());
        for [prev, next] in self.data.array_windows() {
            assert!(
                prev.0 != next.0,
                "Two entries at position {:?}, with values {:?} and {:?}",
                prev.0,
                prev.1,
                next.1
            );
            assert!(
                prev.1 != next.1,
                "Two with same value {:?}, at positions {:?} and {:?}",
                prev.1,
                prev.0,
                next.0
            );
        }
    }

    #[cfg(not(debug_assertions))]
    fn verify(&self) {}
}

impl<I: Indexer + CheckedAdd + Zero, V: Value> CBuffer<I, V> {
    /// Returns None if there is an overflow
    pub fn count_values(&self, value: &V) -> Option<I> {
        let mut total = I::zero();
        for [prev, next] in self.data.array_windows() {
            if &prev.1 == value {
                total = total.checked_add(&(next.0.clone() - prev.0.clone()))?;
            }
        }
        let last = self.data.last().unwrap();
        if &last.1 == value {
            total = total.checked_add(&(I::max_value() - last.0.clone()))?;
        }
        Some(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool_range() {
        let mutator = &mut CBuffer::mutator();
        let mut cbuffer = CBuffer::<usize, bool>::new(false);
        cbuffer.set(0..100, true, mutator);
        assert!(cbuffer.count_values(&true) == Some(100));
        cbuffer.set(0..100, true, mutator);
        assert!(cbuffer.count_values(&true) == Some(100));
        cbuffer.set(150..200, true, mutator);
        assert!(cbuffer.count_values(&true) == Some(150));
        assert_eq!(
            cbuffer.ranges().collect::<Vec<_>>(),
            vec![
                (0..100, &true),
                (100..150, &false),
                (150..200, &true),
                (200..usize::MAX, &false)
            ]
        );
        cbuffer.set(100..150, true, mutator);
        assert!(cbuffer.count_values(&true) == Some(200));
        assert_eq!(
            cbuffer.ranges().collect::<Vec<_>>(),
            vec![(0..200, &true), (200..usize::MAX, &false)]
        );
        cbuffer.set(20..50, true, mutator);
        assert!(cbuffer.count_values(&true) == Some(200));
        cbuffer.set(50..200, false, mutator);
        assert!(cbuffer.count_values(&true) == Some(50));
        cbuffer.set(0..25, false, mutator);
        assert!(cbuffer.count_values(&true) == Some(25));
    }
}
