framework::day!(23, parse => pt1, pt2);

type Grid = VecGrid<Cell>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Wall,
    Slope(Offset),
}

#[derive(Debug, Clone)]
struct Node {
    position: Vec2<u32>,
    // (index, cost)
    next: ArrayVec<(u32, u32), 8>,
}

fn for_each_neighbor(grid: &Grid, pos: Vec2<u32>, mut callback: impl FnMut(Vec2<u32>, Offset)) {
    for offset in Offset::ORTHOGONAL {
        if let Some(neighbor) = pos
            .neighbor(offset)
            .filter(|n| matches!(grid.get(*n), Some(Cell::Empty | Cell::Slope(_))))
        {
            callback(neighbor, offset);
        }
    }
}

fn build_graph(grid: &Grid) -> Vec<Node> {
    let mut is_node = BitGrid::new(grid.size().to_u32(), false);
    let mut edge_node = None;
    for (pos, &cell) in grid.iter() {
        if cell == Cell::Wall {
            continue;
        }
        let pos = pos.to_u32();
        let mut neighbor_count = 0;
        for_each_neighbor(grid, pos, |_, _| neighbor_count += 1);
        if neighbor_count != 2 {
            if neighbor_count == 1 {
                edge_node.get_or_insert(pos);
            }
            is_node.set(pos, true);
        }
    }

    let node_count = is_node.data.count_ones();
    let mut nodes = Vec::with_capacity(node_count);
    let mut explored = Vec::with_capacity(node_count);
    for index in is_node.data.iter_ones() {
        let position = is_node.index_to_position(index);
        nodes.push(Node {
            position,
            next: ArrayVec::new(),
        });

        let exploration_directions = match grid[position] {
            Cell::Empty => 0,
            Cell::Wall => unreachable!(),
            Cell::Slope(slope) => 0b1111 ^ slope.raw(),
        };
        explored.push(exploration_directions);
    }

    let mut explore = |index: usize| {
        let pos = nodes[index].position;
        let already_explored = explored[index];
        for_each_neighbor(grid, pos, |mut pos, mut dir| {
            // Already explored in this direction
            if dir.raw() & already_explored != 0 {
                return;
            }

            let mut can_fwd = true;
            let mut can_bwd = true;
            let mut dist = 1;

            while !is_node[pos] {
                let mut next = None;
                for_each_neighbor(grid, pos, |next_pos, next_dir| {
                    if next_dir == dir.rot_180() {
                        return;
                    }
                    assert!(next.is_none());
                    next = Some((next_pos, next_dir));
                });
                (pos, dir) = next.unwrap();
                if let Cell::Slope(next_dir) = grid[pos] {
                    can_fwd &= dir == next_dir;
                    can_bwd &= dir != next_dir;
                }
                dist += 1;
            }

            // Connect up the two nodes
            let neighbor_index = is_node.data[..is_node.position_to_index(pos)].count_ones();
            if can_fwd {
                nodes[index].next.push((neighbor_index as u32, dist));
            }
            if can_bwd {
                nodes[neighbor_index].next.push((index as u32, dist));
            }
            // Flag path as already explored
            explored[neighbor_index] |= dir.rot_180().raw();
        });
        explored[index] = 0b1111;
    };

    for i in 0..node_count {
        explore(i);
    }

    nodes
}

fn pt1(grid: &Grid) -> u32 {
    let nodes = build_graph(grid);
    assert!(nodes.len() < 64);
    fn dfs(index: u32, visited: u64, length: u32, nodes: &[Node], longest_path: &mut u32) {
        if index as usize == nodes.len() - 1 {
            *longest_path = length.max(*longest_path);
            return;
        }
        let visited = visited | (1 << index);
        for &(next_idx, next_dist) in &nodes[index as usize].next {
            if visited & (1 << next_idx) != 0 {
                continue;
            }
            dfs(next_idx, visited, length + next_dist, nodes, longest_path);
        }
    }

    let mut longest_path = 0;
    dfs(0, 0, 0, &nodes, &mut longest_path);
    longest_path
}

fn pt2(grid: &Grid) -> u32 {
    let mut grid = grid.clone();
    grid.cells_mut().iter_mut().for_each(|cell| {
        if *cell != Cell::Wall {
            *cell = Cell::Empty;
        }
    });
    pt1(&grid)
}

fn parse(input: &[u8]) -> Result<Grid> {
    use parsers::*;
    let cell = any().map_res(|c| {
        Ok(match c {
            b'.' => Cell::Empty,
            b'#' => Cell::Wall,
            b'>' => Cell::Slope(Offset::X_POS),
            b'<' => Cell::Slope(Offset::X_NEG),
            b'v' => Cell::Slope(Offset::Y_POS),
            b'^' => Cell::Slope(Offset::Y_NEG),
            _ => return Err(ParseError::TokenDoesNotMatch),
        })
    });
    grid(cell, token(b'\n')).execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
#.#####################
#.......#########...###
#######.#########.#.###
###.....#.>.>.###.#.###
###v#####.#v#.###.#.###
###.>...#.#.#.....#...#
###v###.#.#.#########.#
###...#.#.#.......#...#
#####.#.#.#######.#.###
#.....#.#.#.......#...#
#.#####.#.#.#########v#
#.#...#...#...###...>.#
#.#.#v#######v###.###v#
#...#.>.#...>.>.#.###.#
#####v#.#.###v#.#.###.#
#.....#...#...#.#.#...#
#.#########.###.#.#.###
#...###...#...#...#.###
###.###.#.###v#####v###
#...#...#.#.>.>.#.>.###
#.###.###.#.###.#.#v###
#.....###...###...#...#
#####################.#";

    test_pt!(parse, pt1, EXAMPLE => 94);
    test_pt!(parse, pt2, EXAMPLE => 154);
}
