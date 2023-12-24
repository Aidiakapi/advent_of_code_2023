framework::day!(04, parse => pt1, pt2);

#[derive(Debug, Clone)]
struct Card {
    winning_numbers: [u8; if_test!(5, 10)],
    drawn_numbers: [u8; if_test!(8, 25)],
}

fn pt1(input: &[Card]) -> u32 {
    input.iter().map(|card| calculate_points(card) as u32).sum()
}

fn matching_card_count(card: &Card) -> usize {
    card.winning_numbers.iter().filter(|nr| card.drawn_numbers.contains(nr)).count()
}

fn calculate_points(card: &Card) -> u16 {
    let matching_card_count = matching_card_count(card);
    if matching_card_count == 0 {
        0
    } else {
        1 << (matching_card_count - 1)
    }
}

fn pt2(input: &[Card]) -> u32 {
    let mut card_count = vec![1; input.len()];
    for (i, card) in input.iter().enumerate() {
        let multiplier = card_count[i];
        for count in card_count.iter_mut().skip(i + 1).take(matching_card_count(card)) {
            *count += multiplier;
        }
    }
    card_count.iter().sum()
}

fn parse(input: &[u8]) -> Result<Vec<Card>> {
    use parsers::*;
    let header = token(b"Card")
        .trailed(token(b' ').repeat())
        .trailed(number::<u16>())
        .trailed(token(b':'));
    let number = token(b' ').then(token(b' ').opt()).then(number::<u8>());
    let winning_numbers = number.many_n();
    let drawn_numbers = number.many_n();
    let card = header
        .then(winning_numbers)
        .and(token(b" |").then(drawn_numbers))
        .map(|(winning_numbers, drawn_numbers)| Card {
            winning_numbers,
            drawn_numbers,
        });
    card.sep_by(token(b'\n')).execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11";

    test_pt!(parse, pt1, EXAMPLE => 13);
    test_pt!(parse, pt2, EXAMPLE => 30);
}
