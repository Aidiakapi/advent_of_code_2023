use std::num::NonZeroU8;

framework::day!(15, parse => pt1, pt2);

fn hash_label(input: &[u8]) -> u32 {
    (input.iter()).fold(0u32, |a, &n| (a + n as u32) * 17 % 256)
}

fn pt1(inputs: &[u8]) -> u32 {
    inputs.split(|&l| l == b',').map(hash_label).sum()
}

fn pt2(input: &[u8]) -> Result<u64> {
    let instructions = parse_instructions(input)?;
    let mut boxes: Vec<ArrayVec<(Label, NonZeroU8), 8>> = Vec::with_capacity(256);
    boxes.resize(256, ArrayVec::new());

    for &(label, focal_length) in &instructions {
        let hash = hash_label(label);
        let lenses = &mut boxes[hash as usize];
        let idx = (lenses.iter()).position(|&(lens_label, _)| lens_label == label);
        match (idx, focal_length) {
            (Some(idx), Some(focal_length)) => lenses[idx] = (label, focal_length),
            (Some(idx), None) => _ = lenses.remove(idx),
            (None, Some(focal_length)) => lenses.push((label, focal_length)),
            (None, None) => (),
        }
    }

    Ok(boxes
        .iter()
        .enumerate()
        .map(|(box_idx, lenses)| {
            let box_idx = box_idx as u64 + 1;
            lenses
                .iter()
                .enumerate()
                .map(|(lens_idx, &(_, focal_length))| {
                    let lens_idx = lens_idx as u64 + 1;
                    box_idx * lens_idx * focal_length.get() as u64
                })
                .sum::<u64>()
        })
        .sum::<u64>())
}

type Label<'s> = &'s [u8];

fn parse(input: &[u8]) -> Result<&[u8]> {
    Ok(input.trim_ascii())
}

fn parse_instructions(input: &[u8]) -> Result<Vec<(Label, Option<NonZeroU8>)>> {
    use parsers::*;
    let label = take_while((), |_, c| !matches!(c, b'-' | b'='));
    let set = token(b'=').then(pattern!(b'1'..=b'9').map(|n| NonZeroU8::new(n - b'0')));
    let remove = token((b'-', None));
    let instruction = label.and(set.or(remove));
    instruction.sep_by(token(b',')).execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";

    test_pt!(parse, pt1, EXAMPLE => 1320);
    test_pt!(parse, pt2, EXAMPLE => 145);
}
