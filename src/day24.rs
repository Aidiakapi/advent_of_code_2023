framework::day!(24, parse => pt1, pt2);

use num::Signed;
use std::fmt::Debug;

type Vec2 = framework::vecs::Vec2<i64>;
type Vec2r = framework::vecs::Vec2<Ratio>;
type Vec3 = framework::vecs::Vec3<i64>;
type Ratio = num::rational::Ratio<i128>;

#[derive(Debug, Clone)]
struct Hailstone {
    pos: Vec3,
    vel: Vec3,
}

const AREA_MIN: i64 = if_test!(7, 200_000_000_000_000);
const AREA_MAX: i64 = if_test!(27, 400_000_000_000_000);

const AREA_SIZE: Ratio = Ratio::new_raw(AREA_MAX as i128 - AREA_MIN as i128, 1);

// Turning vector notation into line equation:
// v.y * x - v.x * y + v.x * p.y - v.y * p.x = 0
//
// This uses the homogenous coordinate equation:
// https://en.wikipedia.org/wiki/Line%E2%80%93line_intersection#Using_homogeneous_coordinates
fn collision_point(pos_a: Vec2, vel_a: Vec2, pos_b: Vec2, vel_b: Vec2) -> Option<Vec2r> {
    let a1 = vel_a.y as i128;
    let a2 = vel_b.y as i128;
    let b1 = -vel_a.x as i128;
    let b2 = -vel_b.x as i128;
    let cp = a1 * b2 - a2 * b1;
    if cp == 0 {
        return None;
    }
    let c1 = vel_a.x as i128 * pos_a.y as i128 - vel_a.y as i128 * pos_a.x as i128;
    let c2 = vel_b.x as i128 * pos_b.y as i128 - vel_b.y as i128 * pos_b.x as i128;
    let ap = b1 * c2 - b2 * c1;
    let bp = a2 * c1 - a1 * c2;
    let point = Vec2r::new(Ratio::new(ap, cp), Ratio::new(bp, cp));

    let ta = time_from_point(pos_a, vel_a, point);
    let tb = time_from_point(pos_b, vel_b, point);
    if ta.is_negative() || tb.is_negative() {
        None
    } else {
        Some(point)
    }
}

fn time_from_point(pos: Vec2, vel: Vec2, point: Vec2r) -> Ratio {
    let (point, pos, vel) = if vel.x == 0 {
        (point.y, pos.y, vel.y)
    } else {
        (point.x, pos.x, vel.x)
    };
    (point - Ratio::from_integer(pos as i128)) / Ratio::from_integer(vel as i128)
}

fn pt1(stones: &[Hailstone]) -> usize {
    stones
        .iter()
        .tuple_combinations()
        .filter(|(a, b)| {
            let pos_a = a.pos.xy() - AREA_MIN;
            let pos_b = b.pos.xy() - AREA_MIN;
            match collision_point(pos_a, a.vel.xy(), pos_b, b.vel.xy()) {
                Some(point) => {
                    (!point.x.is_negative() && !point.y.is_negative())
                        && (point.x <= AREA_SIZE && point.y <= AREA_SIZE)
                }
                None => false,
            }
        })
        .count()
}

fn pt2(stones: &[Hailstone]) -> Result<i64> {
    // Take any hailstone R, and consider it as a frame of reference. That is to
    // say, we only consider positions and velocities relative to it, and that
    // means that its own position and velocity become zero, and it always stays
    // a single point at the origin. All other trajectories, rock and hailstone
    // alike, remain a line.
    //
    // We know that the rock's line must intersect with the point or lines of
    // every hailstone. In other words, the rock will pass through the origin,
    // because that is where hailstone R is, and it will also pass through the
    // lines of all other hailstones. We can take an arbitrary hailstone A, and
    // from any two points on its line, we define a plane through the origin and
    // those two points. The rock itself has to travel along this plane,
    // otherwise, it would not hit A.
    //
    // Another hailstone B also needs to intersect with the rock, therefore, the
    // line it forms must intersect the plane formed by the origin and A. This
    // intersection point gives us the intersection time of the rock with B.
    //
    // We can then swap A and B, repeat the process, and get the intersection
    // time of A. Using this time, we can compute the position and velocity of
    // the rock.
    let [r, a, b] = stones.iter().cloned().next_array().unwrap();
    let sub = |s: &Hailstone| Hailstone {
        pos: s.pos - r.pos,
        vel: s.vel - r.vel,
    };
    let [ra, rb] = [sub(&a), sub(&b)];
    let time_a = intersection_time(&rb, &ra)?;
    let time_b = intersection_time(&ra, &rb)?;

    let rock_pos_at_a = a.pos + a.vel * time_a;
    let rock_pos_at_b = b.pos + b.vel * time_b;
    let delta_time = time_b - time_a;
    let delta_pos = rock_pos_at_b - rock_pos_at_a;
    if delta_pos % delta_time != Vec3::zero() {
        return Err(Error::InvalidInput("rock velocity is not integer"));
    }
    let rock_velocity = delta_pos / delta_time;
    let rock_position = rock_pos_at_a - rock_velocity * time_a;

    Ok(rock_position.x + rock_position.y + rock_position.z)
}

fn intersection_time(reference_line: &Hailstone, stone: &Hailstone) -> Result<i64> {
    let a_pos = reference_line.pos.to_i128();
    let a_vel = reference_line.vel.to_i128();
    let plane_normal = a_pos.cross(a_pos + a_vel);

    let denominator = stone.vel.to_i128().dot(plane_normal);
    if denominator == 0 {
        return Err(Error::InvalidInput("line and plane are parallel"));
    }
    let numerator = -stone.pos.to_i128().dot(plane_normal);
    if numerator % denominator != 0 {
        return Err(Error::InvalidInput("intersection time was not an integer"));
    }
    (i64::try_from(numerator / denominator).ok())
        .ok_or(Error::InvalidInput("intersection time overflow"))
}

fn parse(input: &[u8]) -> Result<Vec<Hailstone>> {
    use parsers::*;

    let nr = number::<i64>();
    let sep_nr = token(b", ").then(nr);
    let vec3 = nr.and(sep_nr).and(sep_nr);
    let vec3 = vec3.map(|((x, y), z)| Vec3::new(x, y, z));

    let stone = vec3.and(token(b" @ ").then(vec3));
    let stone = stone.map(|(pos, vel)| Hailstone { pos, vel });
    let stone = stone.map_res(|stone| {
        if stone.vel.xy() == Vec2::zero() || stone.vel == Vec3::zero() {
            Err(ParseError::Custom("0-velocity"))
        } else {
            Ok(stone)
        }
    });

    stone.sep_by(token(b'\n')).execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
19, 13, 30 @ -2, 1, -2
18, 19, 22 @ -1, -1, -2
20, 25, 34 @ -2, -2, -4
12, 31, 28 @ -1, -2, -1
20, 19, 15 @ 1, -5, -3";

    test_pt!(parse, pt1, EXAMPLE => 2);
    test_pt!(parse, pt2, EXAMPLE => 24 + 13 + 10);
}
