use bitvec::prelude::*;

framework::day!(13, parse => pt1, pt2);

fn count_lines_before_reflection_point(grid: &BitGrid) -> Option<u64> {
    'outer: for y in 0..grid.height() - 1 {
        for offset in 1..(grid.height() - y).min(y + 2) {
            if grid.row(y + 1 - offset) != grid.row(y + offset) {
                continue 'outer;
            }
        }
        return Some(y as u64 + 1);
    }
    None
}

fn count_lines_before_reflection_point_with_smudge(grid: &BitGrid) -> Option<u64> {
    let mut temp = bitarr![u64, LocalBits; 0; 64];
    let temp = &mut temp[..grid.width() as usize];
    'outer: for y in 0..grid.height() - 1 {
        let mut differences = 0;
        for offset in 1..(grid.height() - y).min(y + 2) {
            temp.copy_from_bitslice(grid.row(y + 1 - offset));
            *temp ^= grid.row(y + offset);
            differences += temp.count_ones();
            if differences > 1 {
                continue 'outer;
            }
        }
        if differences == 1 {
            return Some(y as u64 + 1);
        }
    }
    None
}

fn pts(grids: &[BitGrid], f: impl Fn(&BitGrid) -> Option<u64>) -> Result<u64> {
    (grids.iter())
        .map(|grid| f(grid).map(|n| n * 100).or_else(|| f(&grid.transpose())))
        .sum::<Option<u64>>()
        .ok_or(Error::InvalidInput("no reflection exists"))
}

fn pt1(grids: &[BitGrid]) -> Result<u64> {
    pts(grids, count_lines_before_reflection_point)
}

fn pt2(grids: &[BitGrid]) -> Result<u64> {
    pts(grids, count_lines_before_reflection_point_with_smudge)
}

fn parse(input: &[u8]) -> Result<Vec<BitGrid>> {
    use parsers::*;
    let cell = token((b'.', false)).or(token((b'#', true)));
    let grid = grid(cell, token(b'\n'));
    grid.sep_by(token(b'\n')).execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.

#...##..#
#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#";

    test_pt!(parse, pt1, EXAMPLE => 405);
    test_pt!(parse, pt2, EXAMPLE => 400);
}
