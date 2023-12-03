use arrayvec::ArrayVec;

framework::day!(02, parse => pt1, pt2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone)]
struct Game {
    id: u32,
    hands: ArrayVec<ArrayVec<(u8, Color), 3>, 6>,
}

fn pt1(games: &[Game]) -> u32 {
    games
        .iter()
        .filter(|game| {
            game.hands
                .iter()
                .flat_map(|hand| hand.iter().cloned())
                .all(|(count, color)| {
                    count
                        <= match color {
                            Color::Red => 12,
                            Color::Green => 13,
                            Color::Blue => 14,
                        }
                })
        })
        .map(|game| game.id)
        .sum::<u32>()
}

fn pt2(games: &[Game]) -> u32 {
    games
        .iter()
        .map(|game| {
            let mut mins = [0, 0, 0];
            for (count, color) in game.hands.iter().flat_map(|hand| hand.iter().cloned()) {
                let min_count = &mut mins[color as usize];
                if *min_count < count {
                    *min_count = count;
                }
            }
            mins.iter().map(|&n| n as u32).product::<u32>()
        })
        .sum::<u32>()
}

fn parse(input: &[u8]) -> Result<Vec<Game>> {
    use parsers::*;
    let header = token(b"Game ").then(number::<u32>()).trailed(token(b": "));
    let color = token((b"red", Color::Red))
        .or(token((b"green", Color::Green)))
        .or(token((b"blue", Color::Blue)));
    let count_color = number::<u8>().and(token(b' ').then(color));
    let hand = count_color.sep_by(token(b", "));
    let hands = hand.sep_by(token(b"; "));
    let game = header.and(hands).map(|(id, hands)| Game { id, hands });
    game.sep_by(token(b'\n')).execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green";

    test_pt!(parse, pt1, EXAMPLE => 8);
    test_pt!(parse, pt2, EXAMPLE => 2286);
}
