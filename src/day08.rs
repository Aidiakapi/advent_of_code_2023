use num::Integer;

framework::day!(08, parse => pt1, pt2);

#[derive(Debug, Clone)]
struct Input {
    directions: Vec<Direction>,
    connections: HashMap<Node, Connection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Connection {
    left: Node,
    right: Node,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Node([u8; 3]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Left,
    Right,
}

impl Input {
    fn map(&self, node: Node, direction: Direction) -> Node {
        let connection = self.connections[&node];
        match direction {
            Direction::Left => connection.left,
            Direction::Right => connection.right,
        }
    }
}

fn pt1(input: &Input) -> Result<u32> {
    let mut step = 0;
    let mut curr = Node(*b"AAA");
    let mut directions = DirIter::new(&input.directions);
    loop {
        step += 1;
        curr = input.map(curr, directions.next());
        if curr == Node(*b"ZZZ") {
            break;
        }
    }

    Ok(step)
}

// This problem kept getting easier and easier, because the inputs make it so.
//
// In the general case, considering a single initial node, each cycle could
// have multiple termination nodes.
// 11A -> 11Z
// 11Z -> 12Z
// 12Z -> 11A
// Where you could stop after any of the following number of cycles:
// 1, 2, 4, 5, 7, 8, 10, 11, 13, ...
// Mathmatically, we can express this in which cycles can be "terminal", for
// this sequence, given that `n` is the cycle number, that requires two equations:
// 3n+1 or 3n+2
//
// However, in both the test input, and the actual input, this does not occur.
// The consequence is that we can only ever have a single equation.
//
// Another complicating scenario is if it takes a different amount of time to
// get to the exit condition the first time, compared to the second. Example:
// 11A -> 11Z
// 11Z -> 11B
// 11B -> 11A
// This allows us to stop after the following number of cycles:
// 1, 4, 7, 10, 13, ...
// Mathmatically: 3n+1
//
// However, this also never happens, and instead, the exit conditions are always
// spaced out evenly.
// 11A -> 11B
// 11B -> 11Z
// 11Z -> 11B
// Where you can stop at:
// 2, 4, 6, 8, ...
// Or 2n
//
// Because there is no offset, just a coefficient on cycles, the answer becomes
// trivial, namely lcm(coefficients).
//
// The `get_cycle_info` method still validates that these invariants are upheld,
// so that if you give it the more general input, it will spit out an error, but
// it does not solve them.
fn pt2(input: &Input) -> Result<u64> {
    let mut cache = HashSet::new();
    let initial_nodes = (input.connections)
        .keys()
        .filter(|key| matches!(key, Node([_, _, b'A'])));

    let mut lcm = 1u64;
    for &initial_node in initial_nodes {
        let cycle_length = get_cycle_info(input, &mut cache, initial_node)? as u64;
        lcm = lcm.lcm(&cycle_length);
    }

    Ok(lcm)
}

struct DirIter<'a> {
    directions: &'a [Direction],
    index: usize,
}
impl DirIter<'_> {
    fn new(directions: &[Direction]) -> DirIter {
        DirIter {
            directions,
            index: directions.len() - 1,
        }
    }
    fn next(&mut self) -> Direction {
        self.index += 1;
        if self.index >= self.directions.len() {
            self.index = 0;
        }
        self.directions[self.index]
    }
}

fn get_cycle_info(input: &Input, cache: &mut HashSet<(Node, usize)>, initial: Node) -> Result<u32> {
    cache.clear();

    let mut step = 0;
    let mut curr = initial;

    let mut directions = DirIter::new(&input.directions);
    while !matches!(curr, Node([_, _, b'Z'])) {
        step += 1;
        curr = input.map(curr, directions.next());
    }

    let time_till_start = step;
    cache.insert((curr, directions.index));
    step = 0;

    loop {
        step += 1;
        curr = input.map(curr, directions.next());
        if !matches!(curr, Node([_, _, b'Z'])) {
            continue;
        }

        if step != time_till_start {
            return Err(Error::InvalidInput(
                "the input is not a simplified form of the problem",
            ));
        }
        step = 0;
        if !cache.insert((curr, directions.index)) {
            return Ok(time_till_start);
        }
    }
}

fn parse(input: &[u8]) -> Result<Input> {
    use parsers::*;

    let direction = token((b'L', Direction::Left)).or(token((b'R', Direction::Right)));
    let node = any().many_n().map(Node);
    let connection = node
        .and(token(b" = (").then(node))
        .and(token(b", ").then(node))
        .trailed(token(b')'))
        .map(|((from, left), right)| (from, Connection { left, right }));

    (direction.repeat_into())
        .and(token(b"\n\n").then(connection.sep_by(token(b'\n'))))
        .map(|(directions, connections)| Input {
            directions,
            connections,
        })
        .execute(input)
}

tests! {
    test_pt!(parse, pt1, b"\
RL

AAA = (BBB, CCC)
BBB = (DDD, EEE)
CCC = (ZZZ, GGG)
DDD = (DDD, DDD)
EEE = (EEE, EEE)
GGG = (GGG, GGG)
ZZZ = (ZZZ, ZZZ)" => 2, b"\
LLR

AAA = (BBB, BBB)
BBB = (AAA, ZZZ)
ZZZ = (ZZZ, ZZZ)" => 6);
    test_pt!(parse, pt2, b"\
LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)" => 6);
}
