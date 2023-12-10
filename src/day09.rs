framework::day!(09, parse => pt1, pt2);

#[derive(Debug, Clone)]
struct Sequence {
    values: Vec<i32>,
}

impl Sequence {
    fn new(sequence: &[i32], mut buffer: Vec<i32>) -> Sequence {
        buffer.clear();
        buffer.extend_from_slice(sequence);
        for seq_len in (1..sequence.len() + 1).rev() {
            // Exit-condition, all values at this level are the same
            if buffer[buffer.len() - seq_len..].iter().all_equal() {
                let len = sequence.len() + 1 - seq_len;
                let mut src_idx = buffer.len() - 1;
                for i in 0..len {
                    buffer[i] = buffer[src_idx];
                    // This will overflow at the last entry
                    src_idx = src_idx.wrapping_sub(seq_len + i);
                }
                buffer.truncate(len);
                return Sequence { values: buffer };
            }
            for _ in 0..seq_len - 1 {
                let index = buffer.len() - seq_len;
                let delta = buffer[index + 1] - buffer[index];
                buffer.push(delta);
            }
        }
        unreachable!()
    }

    fn next(&mut self) -> i32 {
        let mut iter = self.values.windows_mut();
        while let Some([a, b]) = iter.next() {
            *b += *a;
        }
        *self.values.last().unwrap()
    }

    fn prev(&mut self) -> i32 {
        let mut iter = self.values.windows_mut();
        while let Some([a, b]) = iter.next_back() {
            *b -= *a;
        }
        *self.values.last().unwrap()
    }

    fn release(self) -> Vec<i32> {
        self.values
    }
}

fn pt1(sequences: &[Vec<i32>]) -> i32 {
    let mut buffer = Vec::new();
    let mut sum = 0;
    for sequence in sequences {
        let mut sequence = Sequence::new(sequence, buffer);
        sum += sequence.next();
        buffer = sequence.release();
    }
    sum
}

fn pt2(sequences: &[Vec<i32>]) -> i32 {
    let mut buffer = Vec::new();
    let mut sum = 0;
    for sequence in sequences {
        let len = sequence.len();
        let mut sequence = Sequence::new(sequence, buffer);
        for _ in 1..len {
            sequence.prev();
        }
        sum += sequence.prev();
        buffer = sequence.release();
    }
    sum
}

fn parse(input: &[u8]) -> Result<Vec<Vec<i32>>> {
    use parsers::*;
    number::<i32>()
        .sep_by(token(b' '))
        .sep_by(token(b'\n'))
        .execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45";

    test_pt!(parse, pt1, EXAMPLE => 114);
    test_pt!(parse, pt2, EXAMPLE => 2);
}
