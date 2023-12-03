use crate::vecs::Vec2;
use num::{CheckedAdd, CheckedSub, One};
use std::fmt::{self, Debug};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Offset {
    value: u8,
}

impl Debug for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Offset(")?;
        let mut has_any_flags = false;
        let mut write = |s: &'static str| {
            if has_any_flags {
                write!(f, ", ")?;
            }
            has_any_flags = true;
            write!(f, "{s}")
        };
        if self.value & Offset::X_POS.value != 0 {
            write("X+")?;
        }
        if self.value & Offset::X_NEG.value != 0 {
            write("X-")?;
        }
        if self.value & Offset::Y_POS.value != 0 {
            write("Y+")?;
        }
        if self.value & Offset::Y_NEG.value != 0 {
            write("Y-")?;
        }

        if has_any_flags {
            write!(f, ")")
        } else {
            write!(f, "NONE)")
        }
    }
}

impl Offset {
    pub const NONE: Offset = Offset { value: 0b0000 };
    pub const X_POS: Offset = Offset { value: 0b0001 };
    pub const Y_POS: Offset = Offset { value: 0b0010 };
    pub const X_NEG: Offset = Offset { value: 0b0100 };
    pub const Y_NEG: Offset = Offset { value: 0b1000 };
    pub const X_POS_Y_POS: Offset = Offset { value: 0b0011 };
    pub const X_POS_Y_NEG: Offset = Offset { value: 0b1001 };
    pub const X_NEG_Y_POS: Offset = Offset { value: 0b0110 };
    pub const X_NEG_Y_NEG: Offset = Offset { value: 0b1100 };
    pub const ORTHOGONAL: [Offset; 4] =
        [Offset::X_POS, Offset::X_NEG, Offset::Y_POS, Offset::Y_NEG];
    pub const DIAGONAL: [Offset; 4] = [
        Offset::X_POS_Y_POS,
        Offset::X_POS_Y_NEG,
        Offset::X_NEG_Y_POS,
        Offset::X_NEG_Y_NEG,
    ];
    pub const ALL: [Offset; 8] = [
        Offset::X_POS,
        Offset::X_NEG,
        Offset::Y_POS,
        Offset::Y_NEG,
        Offset::X_POS_Y_POS,
        Offset::X_POS_Y_NEG,
        Offset::X_NEG_Y_POS,
        Offset::X_NEG_Y_NEG,
    ];

    /// Rotates a positive X to positive Y
    pub const fn rot_90(self) -> Offset {
        Offset {
            value: (self.value << 1) & 0b1110 | (self.value >> 3) & 0b0001,
        }
    }
    pub const fn rot_180(self) -> Offset {
        Offset {
            value: (self.value << 2) & 0b1100 | (self.value >> 2) & 0b0011,
        }
    }
    /// Rotates a positive X to negative Y
    pub const fn rot_270(self) -> Offset {
        Offset {
            value: (self.value << 3) & 0b1000 | (self.value >> 1) & 0b0111,
        }
    }

    pub const fn has_x(self) -> bool {
        (self.value & 0b0101) != 0
    }
    pub const fn has_y(self) -> bool {
        (self.value & 0b1010) != 0
    }

    pub const fn transpose(self) -> Offset {
        Offset {
            value: ((self.value << 1) & 0b1010) | ((self.value >> 1) & 0b0101),
        }
    }

    pub const fn flip_x(self) -> Offset {
        if self.has_x() {
            Offset {
                value: self.value ^ 0b0101,
            }
        } else {
            self
        }
    }
    pub const fn flip_y(self) -> Offset {
        if self.has_y() {
            Offset {
                value: self.value ^ 0b1010,
            }
        } else {
            self
        }
    }

    pub const fn from_coordinate(value: Vec2<i32>) -> Option<(Offset, usize)> {
        let (ax, ay) = (value.x.abs(), value.y.abs());
        if ax == 0 {
            if ay == 0 {
                return Some((Offset::NONE, 0));
            }
            return Some((
                if value.y > 0 {
                    Offset::Y_POS
                } else {
                    Offset::Y_NEG
                },
                ay as usize,
            ));
        }
        if ay == 0 {
            return Some((
                if value.x > 0 {
                    Offset::X_POS
                } else {
                    Offset::X_NEG
                },
                ay as usize,
            ));
        }
        if ax != ay {
            return None;
        }

        Some((
            match (value.x > 0, value.y > 0) {
                (true, true) => Offset::X_POS_Y_POS,
                (true, false) => Offset::X_POS_Y_NEG,
                (false, true) => Offset::X_NEG_Y_POS,
                (false, false) => Offset::X_NEG_Y_NEG,
            },
            ax as usize,
        ))
    }
}

pub trait CompatibleNumber = Clone + CheckedAdd + CheckedSub + One;

pub trait Neighbor: Sized {
    fn neighbor(self, offset: Offset) -> Option<Self>;
}

pub trait Neighbors: Neighbor + Clone {
    fn neighbors<const N: usize>(self, offsets: &'static [Offset; N]) -> NeighborIter<Self, N> {
        NeighborIter {
            base: self,
            offsets,
            index: 0,
        }
    }
}

impl<T: Neighbor + Clone> Neighbors for T {}

pub struct NeighborIter<T: Clone + Neighbor, const N: usize> {
    base: T,
    offsets: &'static [Offset; N],
    index: usize,
}

impl<T: Clone + Neighbor, const N: usize> Iterator for NeighborIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= N {
                return None;
            }
            let value = self.base.clone().neighbor(self.offsets[self.index]);
            self.index += 1;
            if let Some(value) = value {
                return Some(value);
            }
        }
    }
}

pub trait NeighborsAlong: Neighbor + Clone {
    fn neighbors_along(self, direction: Offset) -> NeighborsAlongIter<Self> {
        NeighborsAlongIter {
            value: Some(self),
            direction,
        }
    }
}

impl<T: Neighbor + Clone> NeighborsAlong for T {}

pub struct NeighborsAlongIter<T> {
    value: Option<T>,
    direction: Offset,
}

impl<T: Neighbor + Clone> Iterator for NeighborsAlongIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.value.clone() {
            self.value = value.neighbor(self.direction);
        }
        self.value.clone()
    }
}

impl<T: CompatibleNumber> Neighbor for Vec2<T> {
    fn neighbor(self, offset: Offset) -> Option<Self> {
        let one = T::one();
        let x = match offset.value & 0b0101 {
            0b0001 => self.x.checked_add(&one)?,
            0b0100 => self.x.checked_sub(&one)?,
            _ => self.x,
        };
        let y = match offset.value & 0b1010 {
            0b0010 => self.y.checked_add(&one)?,
            0b1000 => self.y.checked_sub(&one)?,
            _ => self.y,
        };
        Some(Vec2 { x, y })
    }
}

#[cfg(test)]
mod test {
    use super::Offset;

    #[test]
    fn rotations() {
        assert_eq!(Offset::Y_POS, Offset::X_POS.rot_90());
        assert_eq!(Offset::X_NEG, Offset::Y_POS.rot_90());
        assert_eq!(Offset::Y_NEG, Offset::X_NEG.rot_90());
        assert_eq!(Offset::X_POS, Offset::Y_NEG.rot_90());

        assert_eq!(Offset::X_NEG, Offset::X_POS.rot_180());
        assert_eq!(Offset::X_POS, Offset::X_NEG.rot_180());
        assert_eq!(Offset::Y_NEG, Offset::Y_POS.rot_180());
        assert_eq!(Offset::Y_POS, Offset::Y_NEG.rot_180());

        assert_eq!(Offset::Y_NEG, Offset::X_POS.rot_270());
        assert_eq!(Offset::X_NEG, Offset::Y_NEG.rot_270());
        assert_eq!(Offset::Y_POS, Offset::X_NEG.rot_270());
        assert_eq!(Offset::X_POS, Offset::Y_POS.rot_270());
    }
}
