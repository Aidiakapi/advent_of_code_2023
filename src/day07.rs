framework::day!(07, parse => pt1, pt2);

type Input = Vec<(Hand, Bid)>;
type Bid = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Hand([Card; 5]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Card {
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    N10,
    Jack,
    Queen,
    King,
    Ace,
}
const JOKER: Card = Card::Jack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Type {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

fn get_frequencies(Hand(cards): Hand) -> [u8; 13] {
    let mut frequencies = [0; 13];
    for card in cards {
        frequencies[card as usize] += 1;
    }
    frequencies
}

fn order_frequencies(frequencies: [u8; 13]) -> ArrayVec<u8, 5> {
    let mut res: ArrayVec<u8, 5> = frequencies.into_iter().filter(|&f| f > 0).collect();
    res.sort_unstable_by(|a, b| b.cmp(a));
    res
}

fn frequencies_to_type(frequencies: ArrayVec<u8, 5>) -> Type {
    match frequencies.as_slice() {
        [5] => Type::FiveOfAKind,
        [4, 1] => Type::FourOfAKind,
        [3, 2] => Type::FullHouse,
        [3, 1, 1] => Type::ThreeOfAKind,
        [2, 2, 1] => Type::TwoPair,
        [2, 1, 1, 1] => Type::OnePair,
        [1, 1, 1, 1, 1] => Type::HighCard,
        _ => unreachable!(),
    }
}

fn get_type_pt1(hand: Hand) -> Type {
    frequencies_to_type(order_frequencies(get_frequencies(hand)))
}

fn pt1(input: &Input) -> u32 {
    input
        .iter()
        .map(|&(hand, bid)| (get_type_pt1(hand), hand, bid))
        .sorted()
        .enumerate()
        .map(|(i, (_, _, bid))| (i + 1) as u32 * bid)
        .sum()
}

fn get_type_pt2(hand: Hand) -> Type {
    let mut frequencies = get_frequencies(hand);
    let jokers = frequencies[JOKER as usize];
    if jokers == 5 {
        return Type::FiveOfAKind;
    }
    frequencies[JOKER as usize] = 0;
    let mut frequencies = order_frequencies(frequencies);
    frequencies[0] += jokers;
    frequencies_to_type(frequencies)
}

fn hand_to_value(Hand(cards): Hand) -> [u8; 5] {
    use std::cmp::Ordering::*;
    cards.map(|card| match card.cmp(&JOKER) {
        Less => card as u8 + 1,
        Equal => 0,
        Greater => card as u8,
    })
}

fn pt2(input: &Input) -> u32 {
    input
        .iter()
        .map(|&(hand, bid)| (get_type_pt2(hand), hand_to_value(hand), bid))
        .sorted()
        .enumerate()
        .map(|(i, (_, _, bid))| (i + 1) as u32 * bid)
        .sum()
}

fn parse(input: &[u8]) -> Result<Input> {
    use parsers::*;
    let card = any().map_res(|c| {
        Ok(match c {
            b'2' => Card::N2,
            b'3' => Card::N3,
            b'4' => Card::N4,
            b'5' => Card::N5,
            b'6' => Card::N6,
            b'7' => Card::N7,
            b'8' => Card::N8,
            b'9' => Card::N9,
            b'T' => Card::N10,
            b'J' => Card::Jack,
            b'Q' => Card::Queen,
            b'K' => Card::King,
            b'A' => Card::Ace,
            _ => return Err(ParseError::TokenDoesNotMatch),
        })
    });
    let hand = card.many_n().map(Hand);
    let bid = number::<u32>();
    let entry = hand.and(token(b' ').then(bid));
    entry.sep_by(token(b'\n')).execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483";

    test_pt!(parse, pt1, EXAMPLE => 6440);
    test_pt!(parse, pt2, EXAMPLE => 5905);
}
