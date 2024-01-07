use num::Integer;
use std::collections::VecDeque;
framework::day!(21, parse => pt1, pt2);

#[derive(Debug)]
struct Input {
    starting_position: Vec2<u32>,
    rocks: BitGrid,
}

const PT1_STEP_COUNT: u32 = if_test!(6, 64);

#[derive(Debug, Clone)]
struct Search<'a> {
    // BFS queue, (position, remaining steps from this point)
    rocks: &'a BitGrid,
    queue: VecDeque<(Vec2<u32>, u32)>,
    blocked: BitGrid, // either rock or visited
}

impl<'a> Search<'a> {
    fn new(rocks: &'a BitGrid) -> Search {
        Search {
            rocks,
            queue: VecDeque::new(),
            blocked: BitGrid::new(rocks.size(), false),
        }
    }

    fn count_reachable_blocks(&mut self, starting_point: Vec2<u32>, steps: u32) -> usize {
        self.blocked.data.copy_from_bitslice(&self.rocks.data);
        {
            let cell = (self.blocked)
                .get_mut(starting_point)
                .expect("starting_point out of range");
            assert!(!*cell, "starting_point on top of rock");
            cell.commit(true)
        }
        assert!(self.queue.is_empty());
        if steps == 0 {
            return 1; // Just the point we started at
        }
        self.queue.push_back((starting_point, steps));
        let mut count = 0;
        while let Some((pos, remaining)) = self.queue.pop_front() {
            if remaining.is_even() {
                count += 1;
            }
            if remaining == 0 {
                continue;
            }
            for neighbor in pos.neighbors(&Offset::ORTHOGONAL) {
                let Some(cell) = self.blocked.get_mut(neighbor) else {
                    continue;
                };
                let was_blocked = *cell;
                cell.commit(true);
                if !was_blocked {
                    self.queue.push_back((neighbor, remaining - 1))
                }
            }
        }
        count
    }
}

fn pt1(input: &Input) -> usize {
    let mut search = Search::new(&input.rocks);
    search.count_reachable_blocks(input.starting_position, PT1_STEP_COUNT)
}

fn pt2(input: &Input) -> Result<u64> {
    pt2_impl::<26501365>(input)
}

// See day21_vis.html for a visualization, and the source code contains notes
//
// The general idea is that as the diamond grows (by increasing steps), all
// cells in the inner repetitions of the map are fully covered, so we only have
// to do an actual graph test on the boundary, and the rest we can multiply out.
//
// I'm calculating the answer exactly for everything at the edge, and then a
// ring of cells that'd be reachable without any rocks, but might not be when
// considering the rocks. I'm assuming that a ring of size 1 is sufficient,
// because although long corridors could require more, those aren't present in
// the input.
fn pt2_impl<const STEPS: u32>(input: &Input) -> Result<u64> {
    assert!(input.rocks.width() == input.rocks.height());
    assert!(input.rocks.width().is_odd());
    assert!(input.starting_position == input.rocks.size() / 2);
    let width = input.rocks.width();
    // Removes many edge cases, but not strictly necessary
    assert!(STEPS > width * 3);
    let mut search = Search::new(&input.rocks);

    let full_cells_range = (STEPS + 1) / width;
    let full_cells_outer = full_cells_range as u64 * full_cells_range as u64;
    let full_cells_inner = (full_cells_range as u64 - 1) * (full_cells_range as u64 - 1);
    let full_cells_even_count = search.count_reachable_blocks(Vec2::zero(), STEPS) as u64;
    let full_cells_odd_count = search.count_reachable_blocks(Vec2::zero(), STEPS + 1) as u64;
    let (full_cells_outer_count, full_cells_inner_count) = if full_cells_range.is_odd() {
        (full_cells_even_count, full_cells_odd_count)
    } else {
        (full_cells_odd_count, full_cells_even_count)
    };

    let full_cells_count =
        full_cells_outer * full_cells_outer_count + full_cells_inner * full_cells_inner_count;

    let half_width = width / 2;
    let axis_to_position = |is_neg: bool, is_pos: bool| match (is_neg, is_pos) {
        (false, false) => half_width,
        (true, false) => 0,
        (false, true) => width - 1,
        (true, true) => unreachable!(),
    };
    let offset_to_position = |offset: Offset| {
        Vec2::new(
            axis_to_position(offset.has_x_neg(), offset.has_x_pos()),
            axis_to_position(offset.has_y_neg(), offset.has_y_pos()),
        )
    };

    let mut orthogonal_count = 0;
    let orthogonal_remaining_steps = STEPS - 1 - half_width - (full_cells_range - 1) * width;
    for offset in Offset::ORTHOGONAL {
        orthogonal_count += search
            .count_reachable_blocks(offset_to_position(offset), orthogonal_remaining_steps)
            as u64
    }
    if orthogonal_remaining_steps >= width {
        let orthogonal_overflow = orthogonal_remaining_steps - width;
        for offset in Offset::ORTHOGONAL {
            orthogonal_count += search
                .count_reachable_blocks(offset_to_position(offset), orthogonal_overflow)
                as u64;
        }
    }

    let mut diag_count = 0;
    let diag_main_cells = (full_cells_range - 1) / 2 * 2 + 1;
    let steps_into_main = (STEPS + 1 - width) % (width * 2);
    if diag_main_cells > 0 && steps_into_main >= 2 {
        let steps_into_off = steps_into_main - 2;
        for offset in Offset::DIAGONAL {
            diag_count += diag_main_cells as u64
                * search.count_reachable_blocks(offset_to_position(offset), steps_into_off) as u64;
        }
    }

    let diag_off_cells = (full_cells_range - 2) / 2 * 2 + 2;
    let steps_into_off = (STEPS + 1 - 2 * width) % (width * 2);
    if diag_off_cells > 0 && steps_into_off >= 2 {
        let steps_into_off = steps_into_off - 2;
        for offset in Offset::DIAGONAL {
            diag_count += diag_off_cells as u64
                * search.count_reachable_blocks(offset_to_position(offset), steps_into_off) as u64;
        }
    }

    let total_count = full_cells_count + orthogonal_count + diag_count;
    Ok(total_count)
}

// Test implementation that was used for validation
// fn pt2_dummy<const STEP_COUNT: u32>(input: &Input) -> u64 {
//     let mut visited = HashSet::new();
//     visited.insert(input.starting_position.to_i32());
//     graph::bfs(
//         (input.starting_position.to_i32(), 0),
//         |(node, cost), next| -> Option<()> {
//             if cost == STEP_COUNT {
//                 return None;
//             }
//             for neighbor in node.neighbors(&Offset::ORTHOGONAL) {
//                 let size = input.rocks.width() as i32;
//                 let adjusted_pos =
//                     Vec2::new(neighbor.x.rem_euclid(size), neighbor.y.rem_euclid(size)).to_u32();
//                 if !input.rocks[adjusted_pos] && visited.insert(neighbor) {
//                     next.push((neighbor, cost + 1));
//                 }
//             }
//             None
//         },
//     );
//     visited
//         .iter()
//         .filter(|n| {
//             n.manhathan_distance(input.starting_position.to_i32())
//                 .is_even()
//                 == STEP_COUNT.is_even()
//         })
//         .count() as u64
// }

fn parse(input: &[u8]) -> Result<Input> {
    use parsers::*;
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Cell {
        GardenPlot,
        Rock,
        StartingPosition,
    }

    let cell = any().map_res(|c| {
        Ok(match c {
            b'.' => Cell::GardenPlot,
            b'#' => Cell::Rock,
            b'S' => Cell::StartingPosition,
            _ => return Err(ParseError::TokenDoesNotMatch),
        })
    });
    let grid: VecGrid<Cell> = grid(cell, token(b'\n')).execute(input)?;

    let starting_indices = (grid.cells().iter()).positions(|&c| c == Cell::StartingPosition);
    let starting_index = (starting_indices.exactly_one().ok())
        .ok_or(ParseError::Custom("must have a single starting position"))?;
    let starting_position = grid.index_to_position(starting_index);

    let mut rocks = BitGrid::new(grid.size().to_u32(), false);
    (grid.into_iter().filter(|&(_, c)| c == Cell::Rock))
        .for_each(|(p, _)| rocks.set(p.to_u32(), true));

    Ok(Input {
        starting_position: starting_position.to_u32(),
        rocks,
    })
}

tests! {
    const PT1_EXAMPLE: &'static [u8] = b"\
...........
.....###.#.
.###.##..#.
..#.#...#..
....#.#....
.##..S####.
.##..#...#.
.......##..
.##.#.####.
.##..##.##.
...........";
    const PT2_SIMPLE_EXAMPLE: &'static [u8] = b"\
.....
.....
..S..
.....
.....";
    const PT2_COMPLEX_EXAMPLE: &'static [u8] = b"\
...........
.####..#.#.
.#.##..#.#.
.#.....#.#.
.####..###.
.....S.....
.####..###.
.#.##..#.#.
.#.##..#.#.
.####..#.#.
...........";

    test_pt!(parse, pt1, PT1_EXAMPLE => 16);
    test_pt!(parse, pt2_50s, |input| { pt2_impl::<50>(&input) }, PT2_SIMPLE_EXAMPLE => 2601);
    test_pt!(parse, pt2_50m, |input| { pt2_impl::<50>(&input) }, PT2_COMPLEX_EXAMPLE => 1587);
}
