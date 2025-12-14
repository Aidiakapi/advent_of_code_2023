use ahash::RandomState;
use hashbrown::hash_map::RawEntryMut;

framework::day!(14, parse => pt1, pt2);

type HashMap<K, V> = hashbrown::hash_map::HashMap<K, V, RandomState>;

type Grid = VecGrid<Cell>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Wall,
    Ball,
}

fn pt1(grid: &Grid) -> u32 {
    let mut grid = grid.clone();
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            if grid[(x, y)] != Cell::Ball {
                continue;
            }
            let target_y = ((0..y).rev())
                .take_while(|&ny| grid[(x, ny)] == Cell::Empty)
                .last();
            if let Some(target_y) = target_y {
                grid[(x, y)] = Cell::Empty;
                grid[(x, target_y)] = Cell::Ball;
            }
        }
    }
    grid.iter()
        .filter(|&(_, &cell)| cell == Cell::Ball)
        .map(|(pos, _)| grid.height() - pos.y)
        .sum()
}

fn copy_set_bits_rotated_cw(from: &BitGrid, into: &mut BitGrid) {
    debug_assert!(from.size() == into.size() && from.width() == from.height());
    for y in 0..from.height() {
        let row = from.row(y);
        for x in row.iter_ones() {
            into.set(Vec2::new(into.height() - 1 - y, x as u32), true);
        }
    }
}

fn rotate_cw_with_temp(target: &mut BitGrid, temp: &mut BitGrid) {
    temp.fill(false);
    copy_set_bits_rotated_cw(&*target, temp);
    std::mem::swap(target, temp);
}

fn pt2_core(grid: &Grid, mut should_stop: impl FnMut(&BitGrid) -> bool) {
    let mut walls = BitGrid::new(Vec2::from(grid.width()), false);
    let mut balls = walls.clone();

    for (pos, &cell) in grid {
        match cell {
            Cell::Empty => (),
            Cell::Wall => walls.set(pos.to_u32(), true),
            Cell::Ball => balls.set(pos.to_u32(), true),
        }
    }

    // Precalculate all the orientations of the walls in the grid, so that we
    // can efficiently sample them.
    let mut r90 = BitGrid::new(walls.size(), false);
    let mut r180 = r90.clone();
    let mut r270 = r90.clone();
    copy_set_bits_rotated_cw(&walls, &mut r90);
    copy_set_bits_rotated_cw(&r90, &mut r180);
    copy_set_bits_rotated_cw(&r180, &mut r270);
    let walls = [r270, walls, r90, r180];

    // We could introduce a rotate_ccw function, but that's too much effort, so
    // just rotate it 3 times.
    let mut temp = BitGrid::new(balls.size(), false);
    for _ in 0..3 {
        rotate_cw_with_temp(&mut balls, &mut temp);
    }

    for i in 0.. {
        let walls = &walls[i % 4];
        for y in 0..walls.height() {
            let (mut walls, mut balls) = (walls.row(y), balls.row_mut(y));
            while !walls.is_empty() {
                // Skip over the walls
                let wall_count = walls.leading_ones();
                walls = &walls[wall_count..];
                balls = &mut balls[wall_count..];
                // Size of the empty space
                let empty_count = walls.leading_zeros();
                let balls_in_segment = &mut balls[..empty_count];
                let ball_count = balls_in_segment.count_ones();
                // Move all the balls to the start of the segment
                balls_in_segment[..ball_count].fill(true);
                balls_in_segment[ball_count..].fill(false);
                // Advance to the next section
                walls = &walls[empty_count..];
                balls = &mut balls[empty_count..];
            }
        }

        // At the end of each cycle (after rolling east, before going north
        // again), check for the exit condition.
        if i % 4 == 3 && should_stop(&balls) {
            return;
        }

        rotate_cw_with_temp(&mut balls, &mut temp);
    }
}

fn calculate_load(balls: &BitGrid) -> u32 {
    (0..balls.height())
        .map(|y| balls.row(y).count_ones() as u32 * (y + 1))
        .sum()
}

const TARGET_CYCLES: u32 = 1000000000;
fn pt2(grid: &Grid) -> u32 {
    // This uses raw hash entries to prevent having to clone the state of the
    // balls into both the vec and the set. The set stores the index into
    // cycle_states.
    let mut cycle_states = Vec::new();
    let mut seen_states = HashMap::<u32, ()>::with_hasher(RandomState::new());

    let mut result = None;
    pt2_core(grid, |balls| {
        let balls_hash = seen_states.hasher().hash_one(balls);
        let current_cycle = cycle_states.len() as u32;
        match seen_states
            .raw_entry_mut()
            .from_hash(balls_hash, |&existing_index| {
                &cycle_states[existing_index as usize] == balls
            }) {
            RawEntryMut::Occupied(slot) => {
                let previous_cycle = *slot.key();
                let cycle_length = current_cycle - previous_cycle;
                let remaining_offset = (TARGET_CYCLES - current_cycle - 1) % cycle_length;
                let goal_cycle = previous_cycle + remaining_offset;
                let goal_balls = &cycle_states[goal_cycle as usize];
                result = Some(calculate_load(goal_balls));
                true
            }
            RawEntryMut::Vacant(slot) => {
                cycle_states.push(balls.clone());
                slot.insert_hashed_nocheck(balls_hash, current_cycle, ());
                false
            }
        }
    });

    result.unwrap()
}

fn parse(input: &[u8]) -> Result<Grid> {
    use parsers::*;
    let cell = token((b'.', Cell::Empty))
        .or(token((b'#', Cell::Wall)))
        .or(token((b'O', Cell::Ball)));
    grid(cell, token(b'\n'))
        .filter(|grid: &Grid| grid.width() == grid.height())
        .execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....";

    test_pt!(parse, pt1, EXAMPLE => 136);
    test_pt!(parse, pt2, EXAMPLE => 64);
}
