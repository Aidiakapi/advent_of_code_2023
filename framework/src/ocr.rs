use crate::util::init_array;

// 4x6 for each character
pub const WIDTH: usize = 4;
pub const HEIGHT: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Char(u32);

impl Char {
    pub fn from_is_enabled(mut is_enabled: impl FnMut(usize, usize) -> bool) -> Char {
        let mut char = 0;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                char = char << 1 | is_enabled(x, y) as u32
            }
        }
        Char(char)
    }

    pub fn recognize(self) -> Option<u8> {
        for &(n, c) in &ALPHABET {
            if self.0 == n {
                return Some(c);
            }
        }
        None
    }
}

pub fn recognize_n<const N: usize>(
    mut is_enabled: impl FnMut(usize, usize) -> bool,
) -> Option<[u8; N]> {
    init_array(|index| {
        let ox = index * (WIDTH + 1);
        let c = Char::from_is_enabled(|x, y| is_enabled(ox + x, y));
        c.recognize().ok_or(())
    })
    .ok()
}

#[rustfmt::skip]
#[allow(clippy::identity_op)]
const ALPHABET: [(u32, u8); 14] = [
    (
        0b_0110 << 20 |
        0b_1001 << 16 |
        0b_1001 << 12 |
        0b_1111 <<  8 |
        0b_1001 <<  4 |
        0b_1001 <<  0,
        b'A'
    ),
    (
        0b_1110 << 20 |
        0b_1001 << 16 |
        0b_1110 << 12 |
        0b_1001 <<  8 |
        0b_1001 <<  4 |
        0b_1110 <<  0,
        b'B'
    ),
    (
        0b_0110 << 20 |
        0b_1001 << 16 |
        0b_1000 << 12 |
        0b_1000 <<  8 |
        0b_1001 <<  4 |
        0b_0110 <<  0,
        b'C'
    ),
    (
        0b_1111 << 20 |
        0b_1000 << 16 |
        0b_1110 << 12 |
        0b_1000 <<  8 |
        0b_1000 <<  4 |
        0b_1111 <<  0,
        b'E'
    ),
    (
        0b_1111 << 20 |
        0b_1000 << 16 |
        0b_1110 << 12 |
        0b_1000 <<  8 |
        0b_1000 <<  4 |
        0b_1000 <<  0,
        b'F'
    ),
    (
        0b_0110 << 20 |
        0b_1001 << 16 |
        0b_1000 << 12 |
        0b_1011 <<  8 |
        0b_1001 <<  4 |
        0b_0111 <<  0,
        b'G'
    ),
    (
        0b_1001 << 20 |
        0b_1001 << 16 |
        0b_1111 << 12 |
        0b_1001 <<  8 |
        0b_1001 <<  4 |
        0b_1001 <<  0,
        b'H'
    ),
    (
        0b_0011 << 20 |
        0b_0001 << 16 |
        0b_0001 << 12 |
        0b_0001 <<  8 |
        0b_1001 <<  4 |
        0b_0110 <<  0,
        b'J'
    ),
    (
        0b_1001 << 20 |
        0b_1010 << 16 |
        0b_1100 << 12 |
        0b_1010 <<  8 |
        0b_1010 <<  4 |
        0b_1001 <<  0,
        b'K'
    ),
    (
        0b_1000 << 20 |
        0b_1000 << 16 |
        0b_1000 << 12 |
        0b_1000 <<  8 |
        0b_1000 <<  4 |
        0b_1111 <<  0,
        b'L'
    ),
    (
        0b_1110 << 20 |
        0b_1001 << 16 |
        0b_1001 << 12 |
        0b_1110 <<  8 |
        0b_1000 <<  4 |
        0b_1000 <<  0,
        b'P'
    ),
    (
        0b_1110 << 20 |
        0b_1001 << 16 |
        0b_1001 << 12 |
        0b_1110 <<  8 |
        0b_1010 <<  4 |
        0b_1001 <<  0,
        b'R'
    ),
    (
        0b_1001 << 20 |
        0b_1001 << 16 |
        0b_1001 << 12 |
        0b_1001 <<  8 |
        0b_1001 <<  4 |
        0b_0110 <<  0,
        b'U'
    ),
    (
        0b_1111 << 20 |
        0b_0001 << 16 |
        0b_0010 << 12 |
        0b_0100 <<  8 |
        0b_1000 <<  4 |
        0b_1111 <<  0,
        b'Z'
    ),
];
