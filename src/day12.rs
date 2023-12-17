use std::fmt::{self, Debug};

framework::day!(12, parse => pt1, pt2);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    cells: ArrayVec<Cell, 28>,
    sequences: ArrayVec<u8, 12>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Filled,
    Unknown,
}

const VALUES: u128 = (1u128 << 120) - 1;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Chunk(u128);
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ChunkRev(u128);
impl Chunk {
    fn from_iter(is_filled: impl Iterator<Item = bool>) -> Chunk {
        let mut len = 0;
        let mut v = 0u128;
        for is_filled in is_filled {
            len += 1;
            v <<= 1;
            if is_filled {
                v |= 1;
            }
        }
        assert!(len > 0 && len <= 120);
        Chunk(((len as u128) << 120) | v)
    }

    #[cfg(test)]
    fn from_str(str: &[u8]) -> Chunk {
        Chunk::from_iter(str.iter().map(|c| match c {
            b'#' => true,
            b'?' => false,
            _ => panic!("unsupported char {c}"),
        }))
    }
}

trait ChunkView: Copy + Debug {
    type Rev: ChunkView;
    fn raw(self) -> u128;
    fn rev(self) -> Self::Rev;
    fn split_at(self, index: u8) -> (Self, Self);
    fn leading_zeros(self) -> u8;
    fn leading_ones(self) -> u8;
    fn get(self, index: u8) -> bool;

    fn to_chunk(self) -> Chunk {
        Chunk(self.raw())
    }
    fn len(self) -> u8 {
        (self.raw() >> 120) as u8
    }
    fn bounds(self) -> Vec2<u8> {
        Vec2::new((self.raw() & VALUES).count_ones() as u8, self.len())
    }
    fn split_first(self) -> (bool, Self) {
        let (fst, rem) = self.split_at(1);
        debug_assert!(fst.len() == 1 && rem.len() + 1 == self.len());
        (((fst.raw() & 1) == 1).into(), rem)
    }
    fn to_string(self) -> String {
        let len = self.len();
        let mut s = String::with_capacity(len as usize);
        let mut remainder = self;
        for _ in 0..len {
            let fst;
            (fst, remainder) = remainder.split_first();
            s.push(if fst { '#' } else { '?' });
        }
        s
    }
}

impl ChunkView for Chunk {
    type Rev = ChunkRev;
    fn raw(self) -> u128 {
        self.0
    }
    fn rev(self) -> ChunkRev {
        ChunkRev(self.0)
    }
    fn leading_zeros(self) -> u8 {
        let values = self.0 & VALUES;
        ((values << (128 - self.len())).leading_zeros() as u8).min(self.len())
    }
    fn leading_ones(self) -> u8 {
        let values = self.0 & VALUES;
        ((values << (128 - self.len())).leading_ones() as u8).min(self.len())
    }
    fn get(self, index: u8) -> bool {
        debug_assert!(index < self.len());
        (((self.0 & VALUES) >> (self.len() - 1 - index)) & 1 == 1).into()
    }

    fn split_at(self, index: u8) -> (Chunk, Chunk) {
        let len = self.len();
        debug_assert!(index <= len);
        let rem = len - index;
        let values = self.0 & VALUES;
        let mask = (1u128 << rem) - 1;
        let start = values >> rem;
        let end = values & mask;
        (
            Chunk(start | ((index as u128) << 120)),
            Chunk(end | ((rem as u128) << 120)),
        )
    }
}

impl ChunkView for ChunkRev {
    type Rev = Chunk;
    fn raw(self) -> u128 {
        self.0
    }
    fn rev(self) -> Chunk {
        Chunk(self.0)
    }
    fn leading_zeros(self) -> u8 {
        (self.0.trailing_zeros() as u8).min(self.len())
    }
    fn leading_ones(self) -> u8 {
        (self.0.trailing_ones() as u8).min(self.len())
    }
    fn get(self, index: u8) -> bool {
        debug_assert!(index < self.len());
        (((self.0 & VALUES) >> index) & 1 == 1).into()
    }

    fn split_at(self, index: u8) -> (ChunkRev, ChunkRev) {
        let index = self.len() - index;
        let (a, b) = self.rev().split_at(index);
        (b.rev(), a.rev())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Seq(u128);
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SeqRev(u128);
impl Seq {
    fn new(lengths: &[u8]) -> Seq {
        assert!(lengths.len() <= 30);
        let len = (lengths.len() as u128) << 120;
        let mut v = 0;
        for &length in lengths.iter().rev() {
            assert!(length < 16);
            v = (v << 4) | length as u128;
        }
        Seq(len | v)
    }
}

trait SeqView: Copy + Debug {
    type Rev: SeqView;
    fn raw(self) -> u128;
    fn split_at(self, index: u8) -> (Self, Self);
    fn rev(self) -> Self::Rev;

    fn to_seq(self) -> Seq {
        Seq(self.raw())
    }
    fn len(self) -> u8 {
        (self.raw() >> 120) as u8
    }
    fn sum(self) -> u32 {
        const ONES: u128 = 0x00111111_11111111_11111111_11111111;
        let raw = self.raw();
        let ones = (raw & ONES).count_ones();
        let twos = (raw & (ONES << 1)).count_ones();
        let fours = (raw & (ONES << 2)).count_ones();
        let eights = (raw & (ONES << 3)).count_ones();
        ones + (twos << 1) + (fours << 2) + (eights << 3)
    }
    fn split_first(self) -> (u8, Self) {
        let (fst, rem) = self.split_at(1);
        ((fst.raw() & 0b1111) as u8, rem)
    }

    fn to_string(self) -> String {
        let len = self.len();
        let mut s = String::with_capacity(len as usize * 3 + 2);
        s.push('[');
        let mut remainder = self;
        for i in 0..len {
            if i != 0 {
                s.push(',');
            }
            let nr;
            (nr, remainder) = remainder.split_first();
            if nr >= 10 {
                s.push('1');
                s.push((nr - 10 + b'0') as char);
            } else {
                s.push((nr + b'0') as char);
            }
        }
        s.push(']');
        s
    }
}

impl SeqView for Seq {
    type Rev = SeqRev;

    fn raw(self) -> u128 {
        self.0
    }
    fn rev(self) -> SeqRev {
        SeqRev(self.0)
    }
    fn split_at(self, index: u8) -> (Self, Self) {
        let len = self.len();
        debug_assert!(index <= len);
        let values = self.0 & VALUES;
        let mask = (1u128 << (index * 4)) - 1;
        let start = values & mask;
        let end = values >> (index * 4);
        (
            Seq(start | ((index as u128) << 120)),
            Seq(end | (((len - index) as u128) << 120)),
        )
    }
}

impl SeqView for SeqRev {
    type Rev = Seq;

    fn raw(self) -> u128 {
        self.0
    }
    fn rev(self) -> Seq {
        Seq(self.0)
    }
    fn split_at(self, index: u8) -> (Self, Self) {
        let index = self.len() - index;
        let (a, b) = self.rev().split_at(index);
        (b.rev(), a.rev())
    }
}

macro_rules! impl_debug {
    ($t:ty) => {
        impl Debug for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.to_string())
            }
        }
    };
}

impl_debug!(Chunk);
impl_debug!(ChunkRev);
impl_debug!(Seq);
impl_debug!(SeqRev);

#[derive(Debug, Default)]
struct DynamicState {
    combination_cache: HashMap<(Chunk, Seq), u64>,
}

impl DynamicState {
    fn count_combinations(&mut self, chunks: &[Chunk], seq: Seq) -> u64 {
        let bounds_sum = (chunks.iter())
            .map(|chunk| chunk.bounds().to_u32())
            .sum::<Vec2<u32>>();
        self.count_impl(chunks, seq, bounds_sum)
    }

    fn count_impl(&mut self, chunks: &[Chunk], seq: Seq, bounds_sum: Vec2<u32>) -> u64 {
        let (first, last) = match chunks {
            [] if seq.len() == 0 => return 1,
            [] => return 0,
            &[sole_chunk] => return self.get_chunk_seq_combinations(sole_chunk, seq),
            &[first, .., last] => (first, last),
        };

        let seq_sum = seq.sum();
        // Special-case, the minimum amount of cells that must be filled in all
        //               is already more than the sequence contains.
        if bounds_sum.x > seq_sum {
            return 0;
        }

        let first_bounds = first.bounds().to_u32();
        let bounds_except_first = bounds_sum - first_bounds;
        let first_limits = Self::calc_limits(seq_sum, bounds_except_first);
        let mut possibilities_first = 0;
        self.foreach_seq(first, first_limits, seq, |_, _, _| possibilities_first += 1);
        if possibilities_first == 0 {
            return 0;
        }

        let last_bounds = last.bounds().to_u32();
        let bounds_except_last = bounds_sum - last_bounds;
        let last_limits = Self::calc_limits(seq_sum, bounds_except_last);
        let mut possibilities_last = 0;
        self.foreach_seq(last.rev(), last_limits, seq.rev(), |_, _, _| {
            possibilities_last += 1
        });
        if possibilities_last == 0 {
            return 0;
        }

        let mut total_combinations = 0;
        let mut continuation =
            |this: &mut Self, rem_seq: Seq, c: u64, rem_chunks: &[Chunk], rem_bounds: Vec2<u32>| {
                total_combinations += c * this.count_impl(rem_chunks, rem_seq, rem_bounds);
            };
        if possibilities_first <= possibilities_last {
            let rem_chunks = &chunks[1..];
            self.foreach_seq(first, first_limits, seq, |this, rem_seq, c| {
                continuation(this, rem_seq, c, rem_chunks, bounds_except_first);
            });
        } else {
            let rem_chunks = &chunks[..chunks.len() - 1];
            self.foreach_seq(last, last_limits, seq.rev(), |this, rem_seq, c| {
                continuation(this, rem_seq.rev(), c, rem_chunks, bounds_except_last);
            });
        };
        return total_combinations;
    }

    fn foreach_seq<C: ChunkView, S: SeqView>(
        &mut self,
        chunk: C,
        chunk_limits: Vec2<u32>,
        seq: S,
        mut callback: impl FnMut(&mut Self, S, u64),
    ) {
        for i in 0..seq.len() + 1 {
            let (sub_seq, rem_seq) = seq.split_at(i);
            let sub_seq_sum = sub_seq.sum();
            // Needs a larger slice to fit this chunk
            if sub_seq_sum < chunk_limits.x {
                continue;
            }
            // Slice is too large for this chunk
            if sub_seq_sum > chunk_limits.y {
                break;
            }
            // Sequence slice can never fit
            if sub_seq_sum + i as u32 > chunk.len() as u32 + 1 {
                break;
            }
            let combinations = self.get_chunk_seq_combinations(chunk.to_chunk(), sub_seq.to_seq());
            if combinations > 0 {
                callback(self, rem_seq, combinations);
            }
        }
    }

    fn calc_limits(seq_sum: u32, bounds: Vec2<u32>) -> Vec2<u32> {
        Vec2::new(seq_sum.saturating_sub(bounds.y), seq_sum - bounds.x)
    }

    fn get_chunk_seq_combinations(&mut self, chunk: Chunk, seq: Seq) -> u64 {
        if let Some(&combinations) = self.combination_cache.get(&(chunk, seq)) {
            return combinations;
        }

        let combinations = self.calc_chunk_seq_combinations(chunk, seq);
        self.combination_cache.insert((chunk, seq), combinations);
        combinations
    }

    fn calc_chunk_seq_combinations(&mut self, chunk: Chunk, seq: Seq) -> u64 {
        let seq_len = seq.len();
        let bounds = chunk.bounds();

        if seq_len == 0 {
            return (bounds.x == 0).into();
        }

        let seq_sum = seq.sum();
        // The sequence can never fill the required amount of cells
        if (bounds.x as u32) > seq_sum {
            return 0;
        }

        // The sequence including its required empty spots is longer than
        // the chunk itself.
        let min_seq_len = seq_sum + seq_len as u32 - 1;
        if (bounds.y as u32) < min_seq_len {
            return 0;
        }

        // Entire chunk must be filled, only valid with a single sequence of
        // that size.
        if bounds.x == bounds.y {
            return (seq_len == 1 && seq_sum == bounds.x as u32).into();
        }

        // Entire chunk is optional, binomial calculates the amount of possible
        // ways to arrange the sequence in O(seq_len).
        if bounds.x == 0 {
            return num::integer::binomial(chunk.len() as u64 + 1 - seq_sum as u64, seq_len as u64);
        }

        let (mut front_count, mut back_count) = (0, 0);
        Self::foreach_pos(chunk, seq, |_, _| front_count += 1);
        if front_count == 0 {
            return 0;
        }
        Self::foreach_pos(chunk.rev(), seq.rev(), |_, _| back_count += 1);
        if back_count == 0 {
            return 0;
        }

        let mut combinations = 0;
        let front_first = front_count < back_count
            // When there's the same amount of possibilities, pop the larger
            // sequence first, as it's more likely for smaller sequences to end
            // up in the cache.
            || (front_count == back_count && seq.split_first().0 >= seq.rev().split_first().0);
        if front_first {
            Self::foreach_pos(chunk, seq, |rem_chunk, rem_seq| {
                combinations += self.get_chunk_seq_combinations(rem_chunk, rem_seq);
            });
        } else {
            Self::foreach_pos(chunk.rev(), seq.rev(), |rem_chunk, rem_seq| {
                combinations += self.get_chunk_seq_combinations(rem_chunk.rev(), rem_seq.rev());
            });
        }

        combinations
    }

    fn foreach_pos<C: ChunkView, S: SeqView>(chunk: C, seq: S, mut callback: impl FnMut(C, S)) {
        let (fst, rem) = seq.split_first();
        let leading_zeros = chunk.leading_zeros();
        // How many ways to place the first element in the sequence entirely
        // before the first "required" element
        let placed_before_possibilities = leading_zeros.saturating_sub(fst);
        for offset in 0..placed_before_possibilities {
            callback(chunk.split_at(offset + fst + 1).1, rem);
        }

        let low = leading_zeros.saturating_sub(fst - 1);
        let hi = (leading_zeros + fst).min(chunk.len()).saturating_sub(fst) + 1;
        for offset in low..hi {
            if offset + fst == chunk.len() || !chunk.get(offset + fst) {
                callback(chunk.split_at((offset + fst + 1).min(chunk.len())).1, rem);
            }
        }
    }
}

fn pt1(rows: &[Row]) -> u64 {
    let mut state = DynamicState::default();
    let mut chunks = Vec::new();
    rows.iter()
        .map(|row| {
            chunks.clear();
            chunks.extend(
                (row.cells.split(|&n| n == Cell::Empty))
                    .filter(|cells| !cells.is_empty())
                    .map(|cells| Chunk::from_iter(cells.iter().map(|&cell| cell == Cell::Filled))),
            );
            let seq = Seq::new(&row.sequences);
            state.count_combinations(&chunks, seq)
        })
        .sum()
}

fn pt2(rows: &[Row]) -> u64 {
    let mut state = DynamicState::default();
    let mut chunks = Vec::new();
    let mut cells_temp = Vec::new();
    let mut seq_temp = Vec::new();
    rows.iter()
        .map(|row| {
            cells_temp.clear();
            cells_temp.extend_from_slice(&row.cells);
            for _ in 0..4 {
                cells_temp.push(Cell::Unknown);
                cells_temp.extend_from_within(..row.cells.len());
            }
            chunks.clear();
            chunks.extend(
                (cells_temp.split(|&n| n == Cell::Empty))
                    .filter(|cells| !cells.is_empty())
                    .map(|cells| Chunk::from_iter(cells.iter().map(|&cell| cell == Cell::Filled))),
            );
            seq_temp.clear();
            for _ in 0..5 {
                seq_temp.extend_from_slice(&row.sequences);
            }
            let seq = Seq::new(&seq_temp);
            state.count_combinations(&chunks, seq)
        })
        .sum()
}

fn parse(input: &[u8]) -> Result<Vec<Row>> {
    use parsers::*;
    let cells = token((b'.', Cell::Empty))
        .or(token((b'#', Cell::Filled)))
        .or(token((b'?', Cell::Unknown)))
        .repeat_into();
    let sequences = number::<u8>()
        .filter(|&n| n > 0 && n < 64)
        .sep_by(token(b','));
    let row = cells
        .and(token(b' ').then(sequences))
        .map(|(cells, sequences)| Row { cells, sequences });
    row.sep_by(token(b'\n')).execute(input)
}

tests! {
    #[test]
    fn chunks() {
        let chunk = Chunk::from_str(b"#??###????");
        assert_eq!(chunk.to_string(), "#??###????");
        assert_eq!(chunk.rev().to_string(), "????###??#");
        assert_eq!(chunk.split_at(2), (Chunk::from_str(b"#?"), Chunk::from_str(b"?###????")));
        assert_eq!(chunk.rev().split_at(2), (Chunk::from_str(b"??").rev(), Chunk::from_str(b"#??###??").rev()));
        assert_eq!(chunk.bounds(), Vec2::new(4, 10));
        assert_eq!(chunk.rev().bounds(), Vec2::new(4, 10));

        assert_eq!(chunk.leading_zeros(), 0);
        assert_eq!(chunk.leading_ones(), 1);
        assert_eq!(chunk.rev().leading_zeros(), 4);
        assert_eq!(chunk.rev().leading_ones(), 0);

        assert_eq!(chunk.get(0), true);
        assert_eq!(chunk.get(1), false);
        assert_eq!(chunk.get(2), false);
        assert_eq!(chunk.get(3), true);
        assert_eq!(chunk.rev().get(6), true);
        assert_eq!(chunk.rev().get(7), false);
        assert_eq!(chunk.rev().get(8), false);
        assert_eq!(chunk.rev().get(9), true);
    }

    #[test]
    fn seq() {
        let seq = Seq::new(&[1,2,3,4]);
        assert_eq!(seq.to_string(), "[1,2,3,4]");
        assert_eq!(seq.rev().to_string(), "[4,3,2,1]");
        assert_eq!(seq.split_at(2), (Seq::new(&[1,2]), Seq::new(&[3,4])));
        assert_eq!(seq.rev().split_at(2), (Seq::new(&[3,4]).rev(), Seq::new(&[1,2]).rev()));
        assert_eq!(seq.sum(), 10);
        assert_eq!(seq.rev().sum(), 10);
    }

    test_pt!(parse, pt1,
        // b"..?????.. 1,2" => 3,
        // b"..?????.. 2,2" => 1,
        // b"..?????.. 3,2" => 0,
        // b"..###.. 3" => 1,
        // b"..####.. 3" => 0,
        // b"???.### 1,1,3" => 1,
        // b".??..??...?##. 1,1,3" => 4,
        // b"?#?#?#?#?#?#?#? 1,3,1,6" => 1,
        // b"????.#...#... 4,1,1" => 1,
        // b"????.######..#####. 1,6,5" => 4,
        // b"?###???????? 3,2,1" => 10,
        // b"#?????????#???? 1,3,2,2" => 17,
        b"??.?????.? 3,1" => 4,
        // b"?.?.?#?##????.?.?# 1,1,3,2,1" => 2,
    );
    test_pt!(parse, pt2,
        b"???.### 1,1,3" => 1,
        b".??..??...?##. 1,1,3" => 16384,
        b"?#?#?#?#?#?#?#? 1,3,1,6" => 1,
        b"????.#...#... 4,1,1" => 16,
        b"????.######..#####. 1,6,5" => 2500,
        b"?###???????? 3,2,1" => 506250,
    );
}
