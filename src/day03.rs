use arrayvec::ArrayVec;

framework::day!(03, parse => pt1, pt2);

type Vec2 = framework::vecs::Vec2<usize>;
type Grid = VecGrid<u8>;

fn iter_ranges<'g>(grid: &'g Grid) -> impl Iterator<Item = (Vec2, usize)> + 'g {
    std::iter::from_coroutine(|| {
        for y in 0..grid.height() {
            let mut x = 0;
            while x < grid.width() {
                let cell = grid[(x, y)];
                if !cell.is_ascii_digit() {
                    x += 1;
                    continue;
                }
                let length = get_number_length(grid, Vec2::new(x, y));
                yield (Vec2::new(x, y), length);
                x += length;
            }
        }
    })
}

fn get_number_length(grid: &Grid, top_left: Vec2) -> usize {
    1 + (top_left.x + 1..grid.width())
        .take_while(|&x| grid[(x, top_left.y)].is_ascii_digit())
        .count()
}

fn range_to_number(grid: &Grid, (pos, length): (Vec2, usize)) -> u32 {
    let mut n = 0;
    for x in pos.x..pos.x + length {
        let digit = grid[(x, pos.y)] - b'0';
        n = n * 10 + digit as u32;
    }
    n
}

fn get_number(grid: &Grid, top_left: Vec2) -> u32 {
    range_to_number(grid, (top_left, get_number_length(grid, top_left)))
}

fn pt1(grid: &Grid) -> u32 {
    iter_ranges(grid)
        .filter(|&(top_left, length)| {
            let top_right = top_left + Vec2::new(length - 1, 0);
            let left_edge = top_left.neighbors(&Offset::ALL_X_NEG);
            let right_edge = top_right.neighbors(&Offset::ALL_X_POS);
            let other_edges = top_left
                .neighbors(&[Offset::Y_NEG, Offset::Y_POS])
                .flat_map(|p| (top_left.x..top_left.x + length).map(move |x| Vec2::new(x, p.y)));
            let mut boundary = left_edge.chain(right_edge).chain(other_edges);
            boundary.any(|p| matches!(grid.get(p), Some(b'!'..=b'-' | b'/' | b'=' | b'@')))
        })
        .map(|range| range_to_number(grid, range))
        .sum::<u32>()
}

fn pt2(grid: &Grid) -> u32 {
    let mut sum = 0;
    for (pos, cell) in grid.iter() {
        if *cell != b'*' {
            continue;
        }

        let mut neighboring_digits = pos
            .neighbors(&Offset::ALL)
            .filter(|p| matches!(grid.get(*p), Some(b'0'..=b'9')))
            .map(|mut p| {
                while let Some(previous_cell) = p.neighbor(Offset::X_NEG) {
                    if !matches!(grid.get(previous_cell), Some(b'0'..=b'9')) {
                        break;
                    }
                    p = previous_cell;
                }
                p
            })
            .collect::<ArrayVec<Vec2, 8>>();
        neighboring_digits.sort_unstable();
        let (unique_digits, _) = neighboring_digits.partition_dedup();
        if unique_digits.len() != 2 {
            continue;
        }

        let a = get_number(grid, neighboring_digits[0]);
        let b = get_number(grid, neighboring_digits[1]);
        let gear_ratio = a * b;
        sum += gear_ratio;
    }
    sum
}

fn parse(input: &[u8]) -> Result<Grid> {
    use parsers::*;
    grid(any().filter(|c| *c != b'\n'), token(b'\n')).execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598..";

    test_pt!(parse, pt1, EXAMPLE => 4361);
    test_pt!(parse, pt2, EXAMPLE => 467835);
}
