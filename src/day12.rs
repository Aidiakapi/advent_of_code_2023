framework::day!(12, parse => pt1, pt2);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    cells: Vec<Cell>,
    sequences: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Cell {
    Empty,
    Filled,
    Unknown,
}

#[derive(Default)]
struct State<'a> {
    cache: HashMap<(&'a [Cell], &'a [u8]), u64>,
}

impl<'a> State<'a> {
    fn count(&mut self, cells: &'a [Cell], seq: &'a [u8]) -> u64 {
        let (first_cell, rem_cells) = match cells.split_first() {
            Some((&fst, rem)) => (fst, rem),
            None => return seq.is_empty().into(),
        };

        let (first_seq, rem_seq) = match seq.split_first() {
            Some((&fst, rem)) => (fst as usize, rem),
            None => return (!cells.contains(&Cell::Filled)).into(),
        };

        if let Some(&cached) = self.cache.get(&(cells, seq)) {
            return cached;
        }

        let mut result = 0;

        // Empty case
        if first_cell != Cell::Filled {
            result += self.count(rem_cells, seq);
        }

        // Filled case
        if first_cell != Cell::Empty {
            let is_valid = first_seq <= cells.len()
                && !cells[..first_seq].contains(&Cell::Empty)
                && !matches!(cells.get(first_seq), Some(Cell::Filled));
            if is_valid {
                result += self.count(&cells[(first_seq + 1).min(cells.len())..], rem_seq);
            }
        }

        self.cache.insert((cells, seq), result);
        result
    }
}

fn pt1(rows: &[Row]) -> u64 {
    let mut state = State::default();
    rows.iter()
        .map(|row| state.count(&row.cells, &row.sequences))
        .sum::<u64>()
}

fn pt2(rows: &[Row]) -> u64 {
    use std::iter::repeat_n;
    let rows = rows
        .iter()
        .map(|row| Row {
            cells: Itertools::intersperse(repeat_n(row.cells.iter(), 5), [Cell::Unknown].iter())
                .flatten()
                .cloned()
                .collect(),
            sequences: repeat_n(row.sequences.iter().cloned(), 5)
                .flatten()
                .collect(),
        })
        .collect_vec();
    pt1(&rows)
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
    test_pt!(parse, pt1,
        b"..?????.. 1,2" => 3,
        b"..?????.. 2,2" => 1,
        b"..?????.. 3,2" => 0,
        b"..###.. 3" => 1,
        b"..####.. 3" => 0,
        b"???.### 1,1,3" => 1,
        b".??..??...?##. 1,1,3" => 4,
        b"?#?#?#?#?#?#?#? 1,3,1,6" => 1,
        b"????.#...#... 4,1,1" => 1,
        b"????.######..#####. 1,6,5" => 4,
        b"?###???????? 3,2,1" => 10,
        b"#?????????#???? 1,3,2,2" => 17,
        b"??.?????.? 3,1" => 4,
        b"?.?.?#?##????.?.?# 1,1,3,2,1" => 2,
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
