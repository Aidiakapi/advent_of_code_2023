framework::day!(10, parse => pt1, pt2);

type Vec2 = framework::vecs::Vec2<u32>;
type Grid = VecGrid<u8>;

const EMPTY: u8 = 0;
const RIGHT: u8 = 0b0001;
const DOWN: u8 = 0b0010;
const LEFT: u8 = 0b0100;
const UP: u8 = 0b1000;

const START: u8 = 0b10000;

fn get_starting_position(grid: &Grid) -> Result<Vec2> {
    grid.iter()
        .find(|&(_, &cell)| cell == START)
        .map(|(position, _)| position)
        .ok_or(Error::InvalidInput("no starting point"))
}

fn for_each_path_pos(grid: &Grid, mut f: impl FnMut(Vec2, u8)) -> Result<()> {
    let starting_pos = get_starting_position(grid)?;

    let (dir_a, dir_b) = Offset::ORTHOGONAL
        .into_iter()
        .filter(|&offset| {
            let reversed = offset.rot_180().raw();
            (starting_pos.neighbor(offset))
                .and_then(|position| grid.get(position))
                .filter(|&&cell| cell & reversed == reversed)
                .is_some()
        })
        .collect_tuple()
        .ok_or(Error::InvalidInput(
            "exactly two pipes need to be connected to starting point",
        ))?;

    const OUT_OF_BOUNDS: Error = Error::InvalidInput("out of bounds");
    let mut current_dir = dir_a;
    let mut current_pos = starting_pos;
    f(current_pos, dir_a.raw() | dir_b.raw());
    loop {
        current_pos = current_pos.neighbor(current_dir).ok_or(OUT_OF_BOUNDS)?;
        if current_pos == starting_pos {
            break;
        }

        let cell = *grid.get(current_pos).ok_or(OUT_OF_BOUNDS)?;
        f(current_pos, cell);
        current_dir = Offset::from_raw(cell ^ current_dir.rot_180().raw());
    }
    Ok(())
}

fn pt1(grid: &Grid) -> Result<u32> {
    let mut loop_length = 0;
    for_each_path_pos(grid, |_, _| loop_length += 1)?;
    Ok(loop_length / 2)
}

fn pt2(grid: &Grid) -> Result<usize> {
    let w3 = grid.width() * 3;
    let h3 = grid.height() * 3;
    let mut mask = BitGrid::new((w3, h3), false);

    for_each_path_pos(grid, |position, cell| {
        let p3 = position * 3;
        mask.set((p3.x + 1, p3.y + 1), true);
        let mut set_if = |dir: u8, ox: u32, oy: u32| {
            if cell & dir == dir {
                mask.set((p3.x + ox, p3.y + oy), true);
            }
        };
        set_if(LEFT, 0, 1);
        set_if(RIGHT, 2, 1);
        set_if(UP, 1, 0);
        set_if(DOWN, 1, 2);
    })?;

    let mut stack = Vec::new();
    stack.push(Vec2::zero());
    while let Some(v) = stack.pop() {
        mask.set((v.x, v.y), true);
        for neighbor in v.neighbors(&Offset::ORTHOGONAL) {
            if matches!(mask.get(neighbor), Some(false)) {
                stack.push(neighbor);
            }
        }
    }

    let count = ((1..h3).step_by(3))
        .flat_map(|y| {
            let base = y * w3 + 1;
            (base..base + w3).step_by(3)
        })
        .filter(|&idx| !mask.data[idx as usize])
        .count();

    Ok(count)
}

fn parse(input: &[u8]) -> Result<Grid> {
    use parsers::*;
    let cell = any().map_res(|c| {
        Ok(match c {
            b'.' => EMPTY,
            b'|' => UP | DOWN,
            b'-' => LEFT | RIGHT,
            b'L' => RIGHT | UP,
            b'J' => LEFT | UP,
            b'7' => LEFT | DOWN,
            b'F' => RIGHT | DOWN,
            b'S' => START,
            _ => return Err(ParseError::TokenDoesNotMatch),
        })
    });
    grid(cell, token(b'\n')).execute(input)
}

tests! {
    test_pt!(parse, pt1, b"\
-L|F7
7S-7|
L|7||
-L-J|
L|-JF" => 4, b"\
7-F7-
.FJ|7
SJLL7
|F--J
LJ.LJ" => 8);
    test_pt!(parse, pt2, b"\
...........
.S-------7.
.|F-----7|.
.||.....||.
.||.....||.
.|L-7.F-J|.
.|..|.|..|.
.L--J.L--J.
..........." => 4, b"\
..........
.S------7.
.|F----7|.
.||....||.
.||....||.
.|L-7F-J|.
.|..||..|.
.L--JL--J.
.........." => 4, b"\
.F----7F7F7F7F-7....
.|F--7||||||||FJ....
.||.FJ||||||||L7....
FJL7L7LJLJ||LJ.L-7..
L--J.L7...LJS7F-7L7.
....F-J..F7FJ|L7L7L7
....L7.F7||L7|.L7L7|
.....|FJLJ|FJ|F7|.LJ
....FJL-7.||.||||...
....L---J.LJ.LJLJ..." => 8, b"\
FF7FSF7F7F7F7F7F---7
L|LJ||||||||||||F--J
FL-7LJLJ||||||LJL-77
F--JF--7||LJLJ7F7FJ-
L---JF-JLJ.||-FJLJJ7
|F|F-JF---7F7-L7L|7|
|FFJF7L7F-JF7|JL---7
7-L-JL7||F7|L7F-7F7|
L.L7LFJ|||||FJL7||LJ
L7JLJL-JLJLJL--JLJ.L" => 10);
}
