framework::day!(17, parse => pt1, pt2);

type Grid = VecGrid<u8>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Node {
    pos: Vec2<i32>,
    dir: Offset,
    steps_taken: u8,
}

fn pts(
    grid: &Grid,
    can_move_forward: impl Fn(u8) -> bool,
    can_rotate: impl Fn(u8) -> bool,
    done: impl Fn(&Node, Vec2<i32>) -> bool,
) -> Result<u32> {
    let goal = (grid.size() - 1).to_i32();
    graph::astar_path_cost(
        |starts| {
            let mut push = |n| starts.push((n, 0));
            push(Node {
                pos: Vec2::zero(),
                dir: Offset::X_POS,
                steps_taken: 0,
            });
            push(Node {
                pos: Vec2::zero(),
                dir: Offset::Y_POS,
                steps_taken: 0,
            });
        },
        |node, nodes| {
            let mut push = |node: Node| {
                if let Some(&cost) = node.pos.try_to_usize().and_then(|pos| grid.get(pos)) {
                    nodes.push((node, cost as u32));
                }
            };

            // Step forward
            if can_move_forward(node.steps_taken) {
                push(Node {
                    pos: node.pos.neighbor(node.dir).unwrap(),
                    dir: node.dir,
                    steps_taken: node.steps_taken + 1,
                });
            }
            if can_rotate(node.steps_taken) {
                // Step turning clockwise
                push(Node {
                    pos: node.pos.neighbor(node.dir.rot_90()).unwrap(),
                    dir: node.dir.rot_90(),
                    steps_taken: 1,
                });
                // Step turning counter clockwise
                push(Node {
                    pos: node.pos.neighbor(node.dir.rot_270()).unwrap(),
                    dir: node.dir.rot_270(),
                    steps_taken: 1,
                });
            }
        },
        |node| {
            let abs = (node.pos - goal).abs();
            (abs.x + abs.y) as u32
        },
        |node| done(node, goal),
    )
    .ok_or(Error::NoSolution)
}

fn pt1(grid: &Grid) -> Result<u32> {
    pts(
        grid,
        |steps_taken| steps_taken < 3,
        |_| true,
        |node, goal| node.pos == goal,
    )
}

fn pt2(grid: &Grid) -> Result<u32> {
    pts(
        grid,
        |steps_taken| steps_taken < 10,
        |steps_taken| steps_taken >= 4,
        |node, goal| node.steps_taken >= 4 && node.pos == goal,
    )
}

fn parse(input: &[u8]) -> Result<Grid> {
    use parsers::*;
    let cell = pattern!(b'1'..=b'9').map(|c| c - b'0');
    grid(cell, token(b'\n')).execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
2413432311323
3215453535623
3255245654254
3446585845452
4546657867536
1438598798454
4457876987766
3637877979653
4654967986887
4564679986453
1224686865563
2546548887735
4322674655533";

    test_pt!(parse, pt1, EXAMPLE => 102);
    test_pt!(parse, pt2, EXAMPLE => 94, b"\
111111111111
999999999991
999999999991
999999999991
999999999991" => 71);
}
