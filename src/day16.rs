framework::day!(16, parse => pt1, pt2);

type Grid = VecGrid<Cell>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    MirrorNeg, // \, +X <=> -Y, -X <=> +Y
    MirrorPos, // /, +X <=> +Y, -X <=> -Y
    SplitHor,  // - Y => -X & +X
    SplitVer,  // | X => -Y & +Y
}

fn pt1(grid: &Grid) -> usize {
    let mut continuations = Vec::new();
    continuations.push((Vec2::new(0, 0), Offset::X_POS));
    let mut visited = VecGrid::new(grid.size(), |_| 0u8);
    get_energized_tiles(grid, &mut continuations, &mut visited)
}

fn pt2(grid: &Grid) -> usize {
    let mut continuations = Vec::new();
    let mut visited = VecGrid::new(grid.size(), |_| 0u8);
    let top = (0..grid.width()).map(|n| (Vec2::new(n, 0), Offset::Y_POS));
    let bot = (0..grid.width()).map(|n| (Vec2::new(n, grid.height() - 1), Offset::Y_NEG));
    let lft = (0..grid.height()).map(|n| (Vec2::new(0, n), Offset::X_POS));
    let rgt = (0..grid.height()).map(|n| (Vec2::new(grid.width() - 1, n), Offset::X_NEG));
    let sides = top.chain(bot).chain(lft).chain(rgt);
    sides
        .map(|start| {
            continuations.clear();
            continuations.push(start);
            visited.cells_mut().fill(0);
            get_energized_tiles(grid, &mut continuations, &mut visited)
        })
        .max()
        .unwrap()
}

fn get_energized_tiles(
    grid: &Grid,
    continuations: &mut Vec<(Vec2<usize>, Offset)>,
    visited: &mut VecGrid<u8>,
) -> usize {
    'outer: while let Some((mut pos, mut dir)) = continuations.pop() {
        loop {
            // Out of bounds
            let visited_flags = match visited.get_mut(pos) {
                Some(v) => v,
                None => continue 'outer,
            };
            // Already visited
            if *visited_flags & dir.raw() != 0 {
                continue 'outer;
            }
            *visited_flags |= dir.raw();

            let cell = grid[pos];
            match cell {
                Cell::MirrorPos => dir = dir.transpose().rot_180(),
                Cell::MirrorNeg => dir = dir.transpose(),
                Cell::SplitHor if dir.has_y() => {
                    dir = Offset::X_POS;
                    if let Some(p) = pos.neighbor(Offset::X_NEG) {
                        continuations.push((p, Offset::X_NEG));
                    }
                }
                Cell::SplitVer if dir.has_x() => {
                    dir = Offset::Y_POS;
                    if let Some(p) = pos.neighbor(Offset::Y_NEG) {
                        continuations.push((p, Offset::Y_NEG));
                    }
                }
                _ => (),
            }

            if let Some(p) = pos.neighbor(dir) {
                pos = p;
                continue;
            }

            break;
        }
    }

    visited.cells().iter().filter(|&&v| v != 0).count()
}

fn parse(input: &[u8]) -> Result<Grid> {
    use parsers::*;
    let cell = any().map_res(|c| {
        Ok(match c {
            b'.' => Cell::Empty,
            b'\\' => Cell::MirrorNeg,
            b'/' => Cell::MirrorPos,
            b'-' => Cell::SplitHor,
            b'|' => Cell::SplitVer,
            _ => return Err(ParseError::TokenDoesNotMatch),
        })
    });
    grid(cell, token(b'\n')).execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = br"
.|...\....
|.-.\.....
.....|-...
........|.
..........
.........\
..../.\\..
.-.-/..|..
.|....-|.\
..//.|....".trim_ascii();

    test_pt!(parse, pt1, EXAMPLE => 46);
    test_pt!(parse, pt2, EXAMPLE => 51);
}
