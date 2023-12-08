framework::day!(05, parse => pt1, pt2);

type Range = std::ops::Range<u64>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Almanac {
    seeds: Vec<u64>,
    steps: [Vec<Mapping>; 7],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mapping {
    src: u64,
    dst: u64,
    count: u64,
}

fn pt1(almanac: &Almanac) -> u64 {
    (almanac.seeds.iter())
        .map(|&seed| get_lowest_location(almanac, seed))
        .min()
        .unwrap()
}

fn pt2(almanac: &Almanac) -> u64 {
    (almanac.seeds.array_chunks())
        .map(|&[start, count]| start..start + count)
        .map(|range| get_lowest_location_from_range(almanac, range))
        .min()
        .unwrap()
}

fn get_lowest_location(almanac: &Almanac, seed: u64) -> u64 {
    let mut value = seed;
    for step in almanac.steps.iter() {
        if let Some(mapping) = (step.iter())
            .find(|mapping| value >= mapping.src && value - mapping.src < mapping.count)
        {
            value = value - mapping.src + mapping.dst;
        }
    }
    value
}

fn get_lowest_location_from_range(almanac: &Almanac, range: Range) -> u64 {
    let mut current_ranges = ArrayVec::<Range, 32>::new();
    let mut next_ranges = ArrayVec::new();
    current_ranges.push(range);
    for step in almanac.steps.iter() {
        'current_ranges: while let Some(range) = current_ranges.pop() {
            for mapping in step {
                let src = mapping.src..mapping.src + mapping.count;
                if range.start >= src.end || range.end <= src.start {
                    continue;
                }

                // This mapping overlaps at least a portion of this range.
                // However, there may be parts of the range that fall before or
                // after the mapping. We push those portions back into the
                // current ranges, to try to process them with other rules, or
                // let them fall-through unmapped.
                let overlap = if range.start < src.start {
                    let before = range.start..src.start;
                    current_ranges.push(before);
                    src.start
                } else {
                    range.start
                }..if range.end > src.end {
                    let after = src.end..range.end;
                    current_ranges.push(after);
                    src.end
                } else {
                    range.end
                };
                let overlap_offset = overlap.start - mapping.src + mapping.dst
                    ..overlap.end - mapping.src + mapping.dst;
                next_ranges.push(overlap_offset);
                continue 'current_ranges;
            }

            // No mapping applied to this range, so it proceeds unchanged.
            next_ranges.push(range);
        }

        std::mem::swap(&mut current_ranges, &mut next_ranges);
    }

    (current_ranges.iter())
        .map(|range| range.start)
        .min()
        .unwrap()
}

fn parse(input: &[u8]) -> Result<Almanac> {
    use parsers::*;
    let nr = number::<u64>();
    let seeds = token(b"seeds: ").then(nr.sep_by(token(b' ')));
    let range = token(b'\n')
        .then(nr.and(token(b' ').then(nr)).and(token(b' ').then(nr)))
        .map(|((dst, src), count)| Mapping { dst, src, count });
    let mapping = range.repeat_into();

    let word = pattern!(b'a'..=b'z').repeat();
    let header = token(b"\n\n")
        .then(word)
        .then(token(b"-to-"))
        .then(word)
        .then(token(b" map:"));

    let steps = header.then(mapping).many_n();
    let almanac = seeds
        .and(steps)
        .map(|(seeds, steps)| Almanac { seeds, steps });

    almanac.execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4";

    test_pt!(parse, pt1, EXAMPLE => 35);
    test_pt!(parse, pt2, EXAMPLE => 46);
}
