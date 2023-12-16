use crate::{parsers::ParseError, vecs::Vec2};
use bitvec::prelude::*;
use std::{
    iter::TrustedLen,
    ops::{Index, IndexMut},
    slice::{Iter, IterMut},
};

/// A 2D grid
pub trait Grid<T>: Sized + Index<Self::Indexer> {
    type Indexer;
    type Builder: GridBuilder<T, Output = Self>;
}

pub trait GridBuilder<T> {
    type Output;
    fn new() -> Self;
    fn is_empty(&self) -> bool;
    fn push_cell(&mut self, cell: T) -> Result<(), ParseError>;
    fn advance_next_line(&mut self) -> Result<(), ParseError>;
    fn finish(self) -> Result<Self::Output, ParseError>;
}

macro_rules! impl_continuous_grid {
    ($ty_name:ident, $builder_ty_name:ident, $index:ty, $storage:ty, $element:ty, [$($generics:tt)*]) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $ty_name$($generics)* {
            size: Vec2<$index>,
            data: $storage,
        }

        #[derive(Debug, Clone)]
        pub struct $builder_ty_name$($generics)* {
            width: Option<$index>,
            x: $index,
            data: $storage,
        }

        impl$($generics)* Grid<$element> for $ty_name$($generics)* {
            type Indexer = Vec2<$index>;
            type Builder = $builder_ty_name$($generics)*;
        }

        impl$($generics)* $ty_name$($generics)* {
            #[inline]
            pub fn size(&self) -> Vec2<$index> {
                self.size
            }

            #[inline]
            pub fn width(&self) -> $index {
                self.size.x
            }

            #[inline]
            pub fn height(&self) -> $index {
                self.size.y
            }
        }

        impl$($generics)* GridBuilder<$element> for $builder_ty_name$($generics)* {
            type Output = $ty_name$($generics)*;
            fn new() -> Self {
                Self {
                    width: None,
                    x: 0,
                    data: <$storage>::new(),
                }
            }

            fn is_empty(&self) -> bool {
                self.data.is_empty()
            }

            fn push_cell(&mut self, cell: $element) -> Result<(), ParseError> {
                if let Some(width) = self.width {
                    if self.x >= width {
                        return Err(ParseError::GridCellAfterEndOfRowReached);
                    }
                }
                self.data.push(cell);
                self.x += 1;
                Ok(())
            }

            fn advance_next_line(&mut self) -> Result<(), ParseError> {
                if let Some(width) = self.width {
                    if self.x != width {
                        return Err(ParseError::GridIncompleteRow);
                    }
                } else {
                    self.width = Some(self.x);
                }
                self.x = 0;
                Ok(())
            }

            fn finish(mut self) -> Result<Self::Output, ParseError> {
                if self.width.is_none() {
                    self.advance_next_line()?;
                }
                let width = self.width.unwrap();
                if self.x != 0 && self.x != width {
                    return Err(ParseError::GridIncompleteRow);
                }
                debug_assert!(self.data.len() as $index % width == 0);
                let height = (self.data.len() / width as usize) as $index;
                Ok(Self::Output {
                    size: Vec2::new(width, height),
                    data: self.data,
                })
            }
        }
    };
}

impl_continuous_grid!(VecGrid, VecGridBuilder, usize, Vec<T>, T, [<T>]);

impl<T> VecGrid<T> {
    fn new_impl(size: Vec2<usize>, mut initializer: impl FnMut(Vec2<usize>) -> T) -> Self {
        assert!(size.x > 0);
        assert!(size.y > 0);
        let capacity = size.x.checked_mul(size.y).expect("overflow");
        let mut data = Vec::with_capacity(capacity);
        for y in 0..size.y {
            for x in 0..size.x {
                data.push(initializer(Vec2::new(x, y)));
            }
        }
        VecGrid { size, data }
    }

    pub fn new(size: impl Into<Vec2<usize>>, initializer: impl FnMut(Vec2<usize>) -> T) -> Self {
        Self::new_impl(size.into(), initializer)
    }

    #[inline]
    fn get_impl(&self, index: Vec2<usize>) -> Option<&T> {
        if index.x < self.size.x && index.y < self.size.y {
            unsafe { Some(self.get_unchecked(index)) }
        } else {
            None
        }
    }

    #[inline]
    pub fn get<V: Into<Vec2<usize>>>(&self, index: V) -> Option<&T> {
        self.get_impl(index.into())
    }

    #[inline]
    fn get_mut_impl(&mut self, index: Vec2<usize>) -> Option<&mut T> {
        if index.x < self.size.x && index.y < self.size.y {
            unsafe { Some(self.get_unchecked_mut(index)) }
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut<V: Into<Vec2<usize>>>(&mut self, index: V) -> Option<&mut T> {
        self.get_mut_impl(index.into())
    }

    /// # Safety
    /// Calling this method with an out-of-bounds index is undefined behavior even if the resulting reference is not used.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: Vec2<usize>) -> &T {
        unsafe { self.data.get_unchecked(index.y * self.size.x + index.x) }
    }

    /// # Safety
    /// Calling this method with an out-of-bounds index is undefined behavior even if the resulting reference is not used.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: Vec2<usize>) -> &mut T {
        unsafe { self.data.get_unchecked_mut(index.y * self.size.x + index.x) }
    }

    #[inline]
    pub fn cells(&self) -> &[T] {
        &self.data
    }

    #[inline]
    pub fn cells_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    #[inline]
    pub fn iter(&self) -> VecGridIter<'_, T> {
        VecGridIter {
            data: self.data.iter(),
            size: self.size,
            next: Vec2::zero(),
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> VecGridIterMut<'_, T> {
        VecGridIterMut {
            data: self.data.iter_mut(),
            size: self.size,
            next: Vec2::zero(),
        }
    }

    pub fn stringify(&self, mut to_char: impl FnMut(&T) -> char) -> String {
        let mut str = String::with_capacity((self.size.x + 1) * self.size.y - 1);
        for y in 0..self.size.y {
            if y != 0 {
                str.push('\n')
            }
            for x in 0..self.size.x {
                let c = unsafe { self.get_unchecked((x, y).into()) };
                str.push(to_char(c));
            }
        }
        str
    }
}

impl<T, V: Into<Vec2<usize>>> Index<V> for VecGrid<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: V) -> &Self::Output {
        let index = index.into();
        assert!(index.x < self.size.x);
        assert!(index.y < self.size.y);
        unsafe { self.data.get_unchecked(index.y * self.size.x + index.x) }
    }
}

impl<T, V: Into<Vec2<usize>>> IndexMut<V> for VecGrid<T> {
    #[inline]
    fn index_mut(&mut self, index: V) -> &mut Self::Output {
        let index = index.into();
        assert!(index.x < self.size.x);
        assert!(index.y < self.size.y);
        unsafe { self.data.get_unchecked_mut(index.y * self.size.x + index.x) }
    }
}

impl<T> IntoIterator for VecGrid<T> {
    type IntoIter = VecGridIntoIter<T>;
    type Item = (Vec2<usize>, T);

    fn into_iter(self) -> Self::IntoIter {
        VecGridIntoIter {
            data: self.data.into_iter(),
            size: self.size,
            next: Vec2::<usize>::zero(),
        }
    }
}

impl<'a, T> IntoIterator for &'a VecGrid<T> {
    type IntoIter = VecGridIter<'a, T>;
    type Item = (Vec2<usize>, &'a T);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut VecGrid<T> {
    type IntoIter = VecGridIterMut<'a, T>;
    type Item = (Vec2<usize>, &'a mut T);

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct VecGridIntoIter<T> {
    data: <Vec<T> as IntoIterator>::IntoIter,
    size: Vec2<usize>,
    next: Vec2<usize>,
}

pub struct VecGridIter<'g, T> {
    data: Iter<'g, T>,
    size: Vec2<usize>,
    next: Vec2<usize>,
}

pub struct VecGridIterMut<'g, T> {
    data: IterMut<'g, T>,
    size: Vec2<usize>,
    next: Vec2<usize>,
}

macro impl_iter() {
    fn next(&mut self) -> Option<Self::Item> {
        let point = self.next;
        if point.y >= self.size.y {
            return None;
        }
        let item = unsafe { self.data.next().unwrap_unchecked() };
        self.next.x += 1;
        if self.next.x >= self.size.x {
            self.next.x = 0;
            self.next.y += 1;
        }
        return Some((point, item));
    }
}

impl<T> Iterator for VecGridIntoIter<T> {
    type Item = (Vec2<usize>, T);
    impl_iter!();
}

impl<'g, T> Iterator for VecGridIter<'g, T> {
    type Item = (Vec2<usize>, &'g T);
    impl_iter!();
}

impl<'g, T> Iterator for VecGridIterMut<'g, T> {
    type Item = (Vec2<usize>, &'g mut T);
    impl_iter!();
}

impl_continuous_grid!(BitGrid, BitGridBuilder, u32, BitVec<u64, LocalBits>, bool, []);
impl BitGrid {
    fn new_impl(size: Vec2<u32>, value: bool) -> Self {
        assert!(size.x > 0);
        assert!(size.y > 0);
        let capacity = size.x as usize * size.y as usize;
        let mut data = BitVec::with_capacity(capacity);
        data.resize(capacity, value);
        Self { data, size }
    }

    pub fn new(size: impl Into<Vec2<u32>>, value: bool) -> Self {
        Self::new_impl(size.into(), value)
    }

    pub fn row(&self, y: u32) -> &BitSlice<u64, LocalBits> {
        let start = y as usize * self.size.x as usize;
        let end = start + self.size.x as usize;
        &self.data[start..end]
    }

    pub fn row_mut(&mut self, y: u32) -> &mut BitSlice<u64, LocalBits> {
        let start = y as usize * self.size.x as usize;
        let end = start + self.size.x as usize;
        &mut self.data[start..end]
    }

    pub fn rows(
        &self,
    ) -> impl DoubleEndedIterator<Item = &BitSlice<u64, LocalBits>> + ExactSizeIterator + TrustedLen + '_
    {
        (0..self.size.y).map(move |y| self.row(y))
    }

    pub fn set(&mut self, position: Vec2<u32>, value: bool) {
        self.data.set(
            position.y as usize * self.size.x as usize + position.x as usize,
            value,
        )
    }

    pub fn fill(&mut self, value: bool) {
        self.data.fill(value);
    }

    pub fn transpose(&self) -> BitGrid {
        let mut res = BitGrid::new_impl(self.size.transpose(), false);
        for idx in self.data.iter_ones() {
            let y = idx / self.size.x as usize;
            let x = idx - y * self.size.x as usize;
            res.set(Vec2::new(y as u32, x as u32), true);
        }
        res
    }
}

impl Index<Vec2<u32>> for BitGrid {
    type Output = bool;

    fn index(&self, index: Vec2<u32>) -> &Self::Output {
        assert!(index.x < self.size.x);
        assert!(index.y < self.size.y);
        let index = index.y as usize * self.size.x as usize + index.x as usize;
        if unsafe { self.data.get_unchecked(index) } == true {
            &true
        } else {
            &false
        }
    }
}
