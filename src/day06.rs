use num::{integer::Roots, traits::NumOps, BigInt, Integer, Signed};

framework::day!(06, parse => pt1, pt2);

#[derive(Debug, Clone)]
struct Input {
    times: ArrayVec<u32, 4>,
    distances: ArrayVec<u32, 4>,
}

fn pt1(input: &Input) -> MulOutput<ArrayVec<i32, 4>> {
    MulOutput(
        input
            .times
            .iter()
            .zip(input.distances.iter())
            .map(|(&time, &distance)| calculate_win_count(time as i32, distance as i32))
            .collect(),
    )
}

fn calculate_win_count<T: Integer + Signed + Roots + Clone + NumOps<i32, T>>(
    time: T,
    distance: T,
) -> T {
    let discriminant = time.clone() * time.clone() - distance * 4 - 4;
    if discriminant < T::zero() {
        return T::zero();
    }
    let sqrt = discriminant.sqrt();
    let min = (time.clone() - sqrt.clone() + 1) / 2;
    let max = (time + sqrt) / 2;
    T::one() + max - min
}

fn pt2(input: &Input) -> String {
    let time = concat_numbers(&input.times);
    let distance = concat_numbers(&input.distances);
    calculate_win_count(BigInt::from(time), BigInt::from(distance)).to_string()
}

fn concat_numbers(nrs: &[u32]) -> u64 {
    let mut res = 0;
    for nr in nrs {
        visit(&mut res, *nr as u64);
    }

    fn visit(res: &mut u64, n: u64) {
        if n == 0 {
            return;
        }
        visit(res, n / 10);
        *res = *res * 10 + n % 10;
    }

    res
}

fn parse(input: &[u8]) -> Result<Input> {
    use parsers::*;
    let seq = token(b' ').repeat().then(number::<u32>()).repeat_into();
    let times = token(b"Time:").then(seq.clone());
    let distances = token(b"Distance:").then(seq);
    (times.and(token(b'\n').then(distances)))
        .map(|(times, distances)| Input { times, distances })
        .map_res(|input| {
            if input.times.len() == input.distances.len() {
                Ok(input)
            } else {
                Err(ParseError::Custom("times.len() != distances.len()"))
            }
        })
        .execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
Time:      7  15   30
Distance:  9  40  200";

    test_pt!(parse, pt1, EXAMPLE => MulOutput([4, 8, 9].into_iter().collect()));
    test_pt!(parse, pt2, EXAMPLE => "71503");
}
