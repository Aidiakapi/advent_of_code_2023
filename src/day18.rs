framework::day!(18, parse => pt1, pt2);

#[derive(Debug, Clone)]
struct Instruction {
    direction: Offset,
    distance: u32,
}

// The key insights are that we're keeping track of which corner of the # symbol
// we're on. This can either be the inner corner, or the outer corner. Example:
// ....↙ at position (4,0)
// ####.
// ..↗#. at position (3,1)
// ...#.
//
// In isolation, we cannot tell which corner is the exterior or interior,
// however, we arbitrarily choose one, and then flip it over to consider the
// alternative. We calculate the area of either of them, and then take whichever
// one is the largest.
//
// The second insight is that when going around a corner, is that a move between
// two points can never result in a shift in both axis. So we can pick an
// arbitrary corner, and just flip it if it moved in the opposite axis. Example:
// ↘...... The previous corner on the top-left (1,1)
// .#####. The 90° corner is on the bottom-left (5,2), but this is invalid,
// .#..↗#. because we moved in the X-direction, but the Y coordinate changes,
//         this is illegal, so we instead swap the corner 180°. Resulting in:
// ↘.....↙
// .#####.
// .#...#.
fn iter_corners(instructions: &[Instruction]) -> impl Iterator<Item = (Vec2<i32>, Offset)> + '_ {
    let mut base_pos = Vec2::zero();
    let mut dir_in = instructions[instructions.len() - 1].direction;
    let mut corner_in = Offset::X_NEG_Y_NEG;
    instructions.iter().map(move |i| {
        let dir_out = i.direction;
        let mut corner = Offset::from_raw(dir_in.rot_180().raw() | dir_out.raw());
        let flip_corner = if dir_in.has_x() {
            corner.has_y_pos() != corner_in.has_y_pos()
        } else {
            corner.has_x_pos() != corner_in.has_x_pos()
        };
        if flip_corner {
            corner = corner.rot_180();
        }

        let pos = base_pos;
        base_pos = base_pos.offset(dir_out, i.distance as i32).unwrap();
        dir_in = dir_out;
        corner_in = corner;
        (pos, corner)
    })
}

fn get_corner_pos(pos: Vec2<i32>, corner: Offset) -> Vec2<i32> {
    pos + Vec2::<i32>::new(corner.has_x_pos().into(), corner.has_y_pos().into())
}

fn pts(instructions: &[Instruction]) -> i64 {
    let mut a1 = 0i64;
    let mut a2 = 0i64;

    let (mut l1, mut l2) = (Vec2::zero(), Vec2::zero());
    for (pos, corner) in iter_corners(instructions) {
        let c1 = get_corner_pos(pos, corner).to_i64();
        let c2 = get_corner_pos(pos, corner.rot_180()).to_i64();
        a1 += (c1.x * l1.y) - (c1.y * l1.x);
        a2 += (c2.x * l2.y) - (c2.y * l2.x);
        (l1, l2) = (c1, c2);
    }
    a1.abs().max(a2.abs()) / 2
}

fn pt1((instructions, _): &(Vec<Instruction>, Vec<Instruction>)) -> i64 {
    pts(instructions)
}

fn pt2((_, instructions): &(Vec<Instruction>, Vec<Instruction>)) -> i64 {
    pts(instructions)
}

fn parse(input: &[u8]) -> Result<(Vec<Instruction>, Vec<Instruction>)> {
    use parsers::*;
    let direction_pt1 = any().map_res(|c| {
        Ok(match c {
            b'R' => Offset::X_POS,
            b'L' => Offset::X_NEG,
            b'D' => Offset::Y_POS,
            b'U' => Offset::Y_NEG,
            _ => return Err(ParseError::TokenDoesNotMatch),
        })
    });
    let instruction_pt1 = direction_pt1.and(token(b' ').then(number::<u32>()));
    let instruction_pt1 = instruction_pt1.map(|(direction, distance)| Instruction {
        direction,
        distance,
    });

    let direction_pt2 = pattern!(b'0'..=b'3').map(|d| match d {
        b'0' => Offset::X_POS,
        b'1' => Offset::Y_POS,
        b'2' => Offset::X_NEG,
        b'3' => Offset::Y_NEG,
        _ => unreachable!(),
    });
    let distance_pt2 = token(b" (#").then(take_n::<5>().map_res(|hex_str| {
        std::str::from_utf8(hex_str)
            .ok()
            .and_then(|s| u32::from_str_radix(s, 16).ok())
            .ok_or(ParseError::TokenDoesNotMatch)
    }));
    let instruction_pt2 = distance_pt2.and(direction_pt2).trailed(token(b')'));
    let instruction_pt2 = instruction_pt2.map(|(distance, direction)| Instruction {
        direction,
        distance,
    });

    (instruction_pt1.and(instruction_pt2).sep_by(token(b'\n')))
        .execute(input)
        .map(|instructions: Vec<(Instruction, Instruction)>| instructions.into_iter().unzip())
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)";

    test_pt!(parse, pt1, EXAMPLE => 62);
    test_pt!(parse, pt2, EXAMPLE => 952408144115);
}
