use num::{Integer, Signed, ToPrimitive, Zero};
use std::fmt;
use std::iter::Sum;

macro_rules! substitute {
    ($name:ident, $($token:tt)+) => {
        $($token)+
    };
}

macro_rules! impl_cast_op {
    ($name:ident { $($component:ident),+ }, [($t:ident, $f:ident, $tf:ident), $($rest:tt)*]) => {
        pub fn $f(self) -> $name<$t> {
            $name {
                $($component: self.$component.$f().unwrap(),)+
            }
        }
        pub fn $tf(self) -> Option<$name<$t>> {
            Some($name {
                $($component: self.$component.$f()?,)+
            })
        }
        impl_cast_op!($name { $($component),+ }, [$($rest)*]);
    };
    ($name:ident { $($component:ident),+ }, []) => {};
}

macro_rules! impl_vec {
    ($name:ident, $neg_trait:ident, $($component:ident),+, $str_fmt:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub struct $name<T> {
            $(pub $component: T,)+
        }

        impl<T> $name<T> {
            pub const fn new($($component: T),+) -> Self {
                $name { $($component,)+ }
            }
        }

        impl<T> From<($(substitute!($component, T),)+)> for $name<T> {
            fn from(($($component,)+): ($(substitute!($component, T),)+)) -> Self {
                $name { $($component,)+ }
            }
        }

        impl<T: num::Zero> $name<T> {
            pub fn zero() -> Self {
                $name {
                    $($component: num::zero(),)+
                }
            }
        }

        impl<T: Clone> From<T> for $name<T> {
            fn from(v: T) -> Self {
                $name { $($component: v.clone(),)+ }
            }
        }

        impl<T: Signed + Copy> $name<T> {
            pub fn abs(self) -> Self {
                Self {
                    $($component: self.$component.abs(),)+
                }
            }
        }

        impl<T: Integer + Copy> $name<T> {
            pub fn abs_delta(self, other: Self) -> Self {
                Self {
                    $($component: self.$component.max(other.$component) - self.$component.min(other.$component),)+
                }
            }

            pub fn manhathan_distance(self, other: Self) -> T {
                let delta = self.abs_delta(other);
                T::zero() $(+ delta.$component)+
            }

            pub fn chebyshev_distance(self, other: Self) -> T {
                let delta = self.abs_delta(other);
                let mut max = T::zero();
                $(max = max.max(delta.$component);)+
                max
            }

            pub fn dot(self, other: Self) -> T {
                $(self.$component * other.$component + )+ <T as num::Zero>::zero()
            }

            pub fn clamp(self, min: T, max: T) -> Self {
                Self {
                    $($component: self.$component.clamp(min, max),)+
                }
            }

            pub fn min_comp(self, min: Self) -> Self {
                Self {
                    $($component: self.$component.min(min.$component),)+
                }
            }

            pub fn max_comp(self, max: Self) -> Self {
                Self {
                    $($component: self.$component.max(max.$component),)+
                }
            }

            pub fn minmax_comp(self, other: Self) -> (Self, Self) {
                use $crate::util::OrdExt;
                $(
                    let $component = self.$component.minmax(other.$component);
                )+
                (
                    Self {
                        $($component: $component.0,)+
                    },
                    Self {
                        $($component: $component.1,)+
                    }
                )
            }

            pub fn eq_comp(self, other: Self) -> $name<bool> {
                $name {
                    $($component: self.$component == other.$component,)+
                }
            }
        }

        auto trait $neg_trait {}
        impl<T> !$neg_trait for $name<T> {}
        macro_rules! impl_binary_op {
            ($trait:ident, $fn_name:ident, $assign_trait:ident, $assign_fn_name:ident) => {
                impl<T: std::ops::$trait<Rhs, Output = O>, Rhs, O> std::ops::$trait<$name<Rhs>>
                    for $name<T>
                {
                    type Output = $name<O>;

                    fn $fn_name(self, rhs: $name<Rhs>) -> Self::Output {
                        $name {
                            $($component: self.$component.$fn_name(rhs.$component),)+
                        }
                    }
                }

                impl<T: std::ops::$assign_trait<Rhs>, Rhs> std::ops::$assign_trait<$name<Rhs>>
                    for $name<T>
                {
                    fn $assign_fn_name(&mut self, rhs: $name<Rhs>) {
                        $(self.$component.$assign_fn_name(rhs.$component);)+
                    }
                }

                impl<T: std::ops::$trait<Rhs, Output = O>, Rhs: $neg_trait + Clone, O> std::ops::$trait<Rhs> for $name<T> {
                    type Output = $name<O>;

                    fn $fn_name(self, rhs: Rhs) -> Self::Output {
                        $name {
                            $($component: self.$component.$fn_name(rhs.clone()),)+
                        }
                    }
                }

                impl<T: std::ops::$assign_trait<Rhs>, Rhs: $neg_trait + Clone> std::ops::$assign_trait<Rhs>
                    for $name<T>
                {
                    fn $assign_fn_name(&mut self, rhs: Rhs) {
                        $(self.$component.$assign_fn_name(rhs.clone());)+
                    }
                }
            };
        }

        macro_rules! impl_unary_op {
            ($trait:ident, $fn_name:ident) => {
                impl<T: std::ops::$trait<Output = O>, O> std::ops::$trait for $name<T> {
                    type Output = $name<O>;
                    fn $fn_name(self) -> $name<O> {
                        $name {
                            $($component: self.$component.$fn_name(),)+
                        }
                    }
                }
            };
        }

        impl_binary_op!(Add, add, AddAssign, add_assign);
        impl_binary_op!(Sub, sub, SubAssign, sub_assign);
        impl_binary_op!(Mul, mul, MulAssign, mul_assign);
        impl_binary_op!(Div, div, DivAssign, div_assign);
        impl_binary_op!(Rem, rem, RemAssign, rem_assign);
        impl_binary_op!(BitAnd, bitand, BitAndAssign, bitand_assign);
        impl_binary_op!(BitOr, bitor, BitOrAssign, bitor_assign);
        impl_binary_op!(BitXor, bitxor, BitXorAssign, bitxor_assign);
        impl_binary_op!(Shl, shl, ShlAssign, shl_assign);
        impl_binary_op!(Shr, shr, ShrAssign, shr_assign);

        impl_unary_op!(Neg, neg);
        impl_unary_op!(Not, not);

        impl<T: Zero + std::ops::Add> Sum for $name<T> {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self {
                    $($component: T::zero(),)+
                }, std::ops::Add::add)
            }
        }

        impl<T: fmt::Display> fmt::Display for $name<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, $str_fmt, $(self.$component),+)
            }
        }

        impl<T: ToPrimitive> $name<T> {
            impl_cast_op!($name { $($component),+ }, [
                (u8, to_u8, try_to_u8),
                (u16, to_u16, try_to_u16),
                (u32, to_u32, try_to_u32),
                (u64, to_u64, try_to_u64),
                (u128, to_u128, try_to_u128),
                (usize, to_usize, try_to_usize),
                (i8, to_i8, try_to_i8),
                (i16, to_i16, try_to_i16),
                (i32, to_i32, try_to_i32),
                (i64, to_i64, try_to_i64),
                (i128, to_i128, try_to_i128),
                (isize, to_isize, try_to_isize),
                (f32, to_f32, try_to_f32),
                (f64, to_f64, try_to_f64),
            ]);
        }
    };
}

impl_vec!(Vec1, NotVec1, x, "({})");
impl_vec!(Vec2, NotVec2, x, y, "({}, {})");
impl_vec!(Vec3, NotVec3, x, y, z, "({}, {}, {})");
impl_vec!(Vec4, NotVec4, x, y, z, w, "({}, {}, {}, {})");

macro_rules! impl_transpose {
    ($vec:ident, $($a:ident: $b:ident),+$(,)?) => {
        impl<T> $vec<T> {
            pub fn transpose(self) -> Self {
                Self {
                    $($a: self.$b,)+
                }
            }
        }
    };
}

impl_transpose!(Vec2, x: y, y: x);
impl_transpose!(Vec3, x: z, y: y, z: x);
impl_transpose!(Vec4, x: w, y: z, z: y, w: x);

macro_rules! impl_swizzle {
    ($(@impl)? $into:ident [$from:ident  $(,$rest:ident)*], $($fn:ident [$($comp:ident),+]),+$(,)?) => {
        impl<T> $from<T> {
            $(pub fn $fn(self) -> $into<T> {
                $into::from((
                    $(self.$comp,)+
                ))
            })+
        }
        impl_swizzle!(@impl $into [$($rest),*], $($fn [$($comp),+]),+);
    };
    (@impl $into:ident [], $($fn:ident [$($comp:ident),+]),+$(,)?) => {
    };
}

impl_swizzle!(Vec2 [Vec3, Vec4],
    xy [x, y], xz [x, z],
    yz [y, x], yx [y, z],
    zx [z, x], zy [z, y],
);
impl_swizzle!(Vec3 [Vec4],
    xyz [x, y, z], xyw [x, y, w], xzy [x, z, y], xzw [x, z, w], xwy [x, w, y], xwz [x, w, z],
    yxz [y, x, z], yxw [y, x, w], yzx [y, z, x], yzw [y, z, w], ywx [y, w, x], ywz [y, w, z],
    zxy [z, x, y], zxw [z, x, w], zyx [z, y, x], zyw [z, y, w], zwx [z, w, x], zwy [z, w, y],
    wxy [w, x, y], wxz [w, x, z], wyx [w, y, x], wyz [w, y, z], wzx [w, z, x], wzy [w, z, y],
);

impl<T: Clone + std::ops::Sub<T, Output = T> + std::ops::Mul<T, Output = T>> Vec3<T> {
    #[rustfmt::skip]
    pub fn cross(self, other: Self) -> Self {
        Vec3::new(
            self.y.clone() * other.z.clone() - self.z.clone() * other.y.clone(),
            self.z         * other.x.clone() - self.x.clone() * other.z        ,
            self.x         * other.y         - self.y         * other.x        ,
        )
    }
}
