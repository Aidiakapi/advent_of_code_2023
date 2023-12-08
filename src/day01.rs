framework::day!(01, parse => pt1, pt2);

fn line_slices(line: &[u8]) -> impl DoubleEndedIterator<Item = &[u8]> {
    (0..line.len()).map(|i| &line[i..])
}

fn pts<F: Fn(&[u8]) -> Option<u8>>(input: &[&[u8]], f: F) -> Result<u32> {
    input
        .iter()
        .cloned()
        .map(|line| -> Option<u32> {
            let first = line_slices(line).find_map(&f)?;
            let last = line_slices(line).rev().find_map(&f)?;
            Some((first * 10 + last) as u32)
        })
        .sum::<Option<u32>>()
        .ok_or(Error::InvalidInput("no two digits in string"))
}

fn pt1(input: &[&[u8]]) -> Result<u32> {
    pts(input, starts_with_digit_pt1)
}

fn pt2(input: &[&[u8]]) -> Result<u32> {
    pts(input, starts_with_digit_pt2)
}

fn starts_with_digit_pt1(str: &[u8]) -> Option<u8> {
    str.first()
        .filter(|c| matches!(c, b'1'..=b'9'))
        .map(|c| c - b'0')
}

fn starts_with_digit_pt2(str: &[u8]) -> Option<u8> {
    if let Some(digit) = starts_with_digit_pt1(str) {
        return Some(digit);
    }
    const SPELLED_DIGITS: [&[u8]; 9] = [
        b"one", b"two", b"three", b"four", b"five", b"six", b"seven", b"eight", b"nine",
    ];
    SPELLED_DIGITS
        .iter()
        .enumerate()
        .find(|(_, digit)| str.starts_with(digit))
        .map(|(idx, _)| (idx + 1) as u8)
}

fn parse(input: &[u8]) -> Result<Vec<&[u8]>> {
    use parsers::*;
    Ok(input.lines().filter(|line| !line.is_empty()).collect())
}

tests! {
    test_pt!(parse, pt1, b"\
1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet" => 142);
    test_pt!(parse, pt2, b"\
two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen" => 281);
}
