use crate::iter::{DoubleEndedLendingIterator, LendingIterator};
use std::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

pub fn init_array<T, E, const N: usize>(
    mut f: impl FnMut(usize) -> Result<T, E>,
) -> Result<[T; N], E> {
    struct DropGuard<'r, T, const N: usize> {
        result: &'r mut [MaybeUninit<T>; N],
        initialized_count: usize,
    }

    impl<T, const N: usize> Drop for DropGuard<'_, T, N> {
        fn drop(&mut self) {
            for i in (0..self.initialized_count).rev() {
                unsafe {
                    self.result[i].assume_init_drop();
                }
            }
        }
    }

    let mut result = MaybeUninit::<T>::uninit_array::<N>();
    let mut drop_guard = DropGuard {
        result: &mut result,
        initialized_count: 0,
    };

    for i in 0..N {
        drop_guard.result[i].write(f(i)?);
        drop_guard.initialized_count += 1;
    }

    std::mem::forget(drop_guard);
    Ok(unsafe { MaybeUninit::array_assume_init(result) })
}

pub trait SliceExt<T> {
    fn trim_start(&self, element: &T) -> &Self
    where
        T: Eq;
    fn trim_end(&self, element: &T) -> &Self
    where
        T: Eq;
    fn trim(&self, element: &T) -> &Self
    where
        T: Eq,
    {
        self.trim_start(element).trim_end(element)
    }

    fn get_two_mut(&mut self, a: usize, b: usize) -> Option<(&mut T, &mut T)>;
    fn windows_mut<const N: usize>(&mut self) -> WindowsMut<'_, T, N>;
}

impl<T> SliceExt<T> for [T] {
    fn get_two_mut(&mut self, a: usize, b: usize) -> Option<(&mut T, &mut T)> {
        if a >= self.len() || b >= self.len() {
            return None;
        }
        use std::cmp::Ordering::*;
        match a.cmp(&b) {
            Less => {
                let (n, m) = self.split_at_mut(b);
                Some((&mut n[a], &mut m[0]))
            }
            Equal => None,
            Greater => {
                let (n, m) = self.split_at_mut(a);
                Some((&mut m[0], &mut n[b]))
            }
        }
    }

    fn windows_mut<const N: usize>(&mut self) -> WindowsMut<'_, T, N> {
        WindowsMut {
            ptr: unsafe { NonNull::new_unchecked(self.as_mut_ptr()) },
            len: self.len(),
            _marker: PhantomData,
        }
    }

    fn trim_start(&self, element: &T) -> &Self
    where
        T: Eq,
    {
        let mut res = self;
        while !res.is_empty() {
            unsafe {
                if res.get_unchecked(0) != element {
                    break;
                }
                (_, res) = res.split_at_unchecked(1);
            }
        }
        res
    }

    fn trim_end(&self, element: &T) -> &Self
    where
        T: Eq,
    {
        let mut res = self;
        while !res.is_empty() {
            unsafe {
                if res.get_unchecked(res.len() - 1) != element {
                    break;
                }
                (res, _) = res.split_at_unchecked(res.len() - 1);
            }
        }
        res
    }
}

pub struct WindowsMut<'a, T, const N: usize> {
    ptr: NonNull<T>,
    len: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T: 'a, const N: usize> LendingIterator for WindowsMut<'a, T, N> {
    type Item<'e> = &'e mut [T; N] where Self: 'e, T: 'e;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.len < N {
            return None;
        }
        unsafe {
            let result = &mut *(self.ptr.as_ptr() as *mut [T; N]);
            self.ptr = self.ptr.add(1);
            self.len -= 1;
            Some(result)
        }
    }
}

impl<'a, T: 'a, const N: usize> DoubleEndedLendingIterator for WindowsMut<'a, T, N> {
    fn next_back(&mut self) -> Option<Self::Item<'_>> {
        if self.len < N {
            return None;
        }
        unsafe {
            let result = &mut *(self.ptr.as_ptr().add(self.len - N) as *mut [T; N]);
            self.len -= 1;
            Some(result)
        }
    }
}

pub trait OrdExt: Sized + Ord {
    fn minmax(self, other: Self) -> (Self, Self) {
        if self <= other {
            (self, other)
        } else {
            (other, self)
        }
    }
}

impl<T: Sized + Ord> OrdExt for T {}

#[macro_export]
macro_rules! __private__impl_eq_ord_by {
    ([$($generic_def:tt)+] $ty:ident[$($generic_use:tt)+], $($field:ident),+$(,)*) => {
        $crate::__private__impl_eq_ord_by!(@impl, $ty, [<$($generic_def)+>], [<$($generic_use)+>], $($field),+);
    };
    ($ty:ident, $($field:ident),+$(,)*) => {
        $crate::__private__impl_eq_ord_by!(@impl, $ty, [], [], $($field),+);
    };

    (@impl, $ty:ident, [$($impl_post:tt)*], [$($ty_post:tt)*], $($field:ident),+) => {
        impl $($impl_post)* PartialEq for $ty $($ty_post)* {
            fn eq(&self, other: &Self) -> bool {
                $(self.$field == other.$field)&&+
            }
        }
        impl $($impl_post)* Eq for $ty $($ty_post)* {
        }
        impl $($impl_post)* PartialOrd for $ty $($ty_post)* {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        impl $($impl_post)* Ord for $ty $($ty_post)* {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                $crate::__private__impl_eq_ord_by!(@cmp_impl, self, other, $($field,)+)
            }
        }
    };

    (@cmp_impl, $self:ident, $other:ident, $field:ident, ) => {
        $self.$field.cmp(&$other.$field)
    };

    (@cmp_impl, $self:ident, $other:ident, $field:ident, $($remainder:ident,)+) => {
        $self.$field.cmp(&$other.$field)
            .then_with(|| $crate::__private__impl_eq_ord_by!(@cmp_impl, $self, $other, $($remainder,)+))
    };
}

pub macro impl_eq_ord_by($($token:tt)+) {
    $crate::__private__impl_eq_ord_by!($($token)+);
}
