use std::iter::Sum;

framework::day!(11, parse => pt1, pt2);

type Vec2 = framework::vecs::Vec2<u32>;

const EXPANSION_FACTOR: u64 = if_test!(100, 1_000_000);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Galaxy {
    base_pos: Vec2,
    empty_pos: Vec2,
}

fn get_galaxy_positions(grid: &BitGrid) -> Vec<Galaxy> {
    let mut empty_columns = ArrayVec::<u32, 16>::new();
    for x in 0..grid.width() {
        if (0..grid.height()).all(|y| !grid[(x, y).into()]) {
            empty_columns.push(x);
        }
    }

    let mut galaxy_positions = Vec::new();
    let mut empty_rows = 0;
    for y in 0..grid.height() {
        let row = grid.row(y);
        let mut empty_cols = 0;
        let mut col_index = 0;
        let mut any = false;
        for x in row.iter_ones() {
            any = true;
            let x = x as u32;
            while col_index < empty_columns.len() && empty_columns[col_index] < x {
                col_index += 1;
                empty_cols += 1;
            }
            galaxy_positions.push(Galaxy {
                base_pos: (x, y).into(),
                empty_pos: (empty_cols, empty_rows).into(),
            });
        }
        if !any {
            empty_rows += 1;
        }
    }
    galaxy_positions
}

fn pts<I, S: Sum<I>>(grid: &BitGrid, dist: impl Fn(Galaxy, Galaxy) -> I) -> S {
    let galaxy_positions = get_galaxy_positions(grid);
    (0..galaxy_positions.len())
        .flat_map(|i| (i + 1..galaxy_positions.len()).map(move |j| (i, j)))
        .map(|(i, j)| dist(galaxy_positions[i], galaxy_positions[j]))
        .sum()
}

fn pt1(grid: &BitGrid) -> u32 {
    pts(grid, |a, b| {
        (a.base_pos + a.empty_pos).manhathan_distance(b.base_pos + b.empty_pos)
    })
}

fn pt2(grid: &BitGrid) -> u64 {
    pts(grid, |a, b| {
        let a = a.base_pos.to_u64() + a.empty_pos.to_u64() * (EXPANSION_FACTOR - 1);
        let b = b.base_pos.to_u64() + b.empty_pos.to_u64() * (EXPANSION_FACTOR - 1);
        a.manhathan_distance(b)
    })
}

fn parse(input: &[u8]) -> Result<BitGrid> {
    use parsers::*;
    let cell = token((b'.', false)).or(token((b'#', true)));
    grid(cell, token(b'\n')).execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....";

    test_pt!(parse, pt1, EXAMPLE => 374);
    test_pt!(parse, pt2, EXAMPLE => 8410);
}
