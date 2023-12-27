use std::ops::Range;
framework::day!(19, parse => pt1, pt2);

#[derive(Debug, Clone)]
struct Input {
    workflows: Vec<Workflow>,
    parts: Vec<[u16; 4]>,
}

const IN: Ident = Ident([b'i', b'n', 0]);
const ACCEPT: Ident = Ident([b'A', 0, 0]);
const REJECT: Ident = Ident([b'R', 0, 0]);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ident([u8; 3]);

#[derive(Debug, Clone)]
struct Workflow {
    ident: Ident,
    rules: ArrayVec<Rule, 3>,
    fallthrough: Ident,
}

#[derive(Debug, Clone)]
struct Rule {
    n: u8,
    is_lt: bool,
    threshold: u16,
    target: Ident,
}

fn pt1(input: &Input) -> Result<u64> {
    let mut total = 0u64;
    for part in &input.parts {
        let mut ident = IN;
        'outer: while !matches!(ident, ACCEPT | REJECT) {
            let workflow = (input.workflows.iter().find(|w| w.ident == ident))
                .ok_or(Error::InvalidInput("no workflow with name"))?;
            for rule in &workflow.rules {
                let n = part[rule.n as usize];
                let matches = if rule.is_lt {
                    n < rule.threshold
                } else {
                    n > rule.threshold
                };
                if matches {
                    ident = rule.target;
                    continue 'outer;
                }
            }
            ident = workflow.fallthrough;
        }

        if ident == ACCEPT {
            total += part.iter().map(|&n| n as u64).sum::<u64>();
        }
    }
    Ok(total)
}

const RANGE: Range<u16> = 1..4001;
fn pt2(input: &Input) -> Result<u64> {
    count_combinations(&input.workflows, IN, [RANGE, RANGE, RANGE, RANGE])
}

fn count_combinations(
    workflows: &[Workflow],
    ident: Ident,
    ranges: [Range<u16>; 4],
) -> Result<u64> {
    match ident {
        ACCEPT => return Ok(ranges.iter().map(|r| r.len() as u64).product()),
        REJECT => return Ok(0),
        _ => (),
    }
    let workflow = (workflows.iter().find(|w| w.ident == ident))
        .ok_or(Error::InvalidInput("no workflow with name"))?;

    let mut combinations = 0;
    let mut remainder = ranges;
    for rule in &workflow.rules {
        let mut next_ranges = remainder.clone();
        let next_range = &mut next_ranges[rule.n as usize];
        let curr_range = &mut remainder[rule.n as usize];
        if rule.is_lt {
            if curr_range.start < rule.threshold {
                next_range.end = rule.threshold;
                curr_range.start = rule.threshold;
                combinations += count_combinations(workflows, rule.target, next_ranges)?;
            }
        } else {
            // 4..10, > 6, next = 4..7, remainder = 7..10
            let threshold_excl = rule.threshold + 1;
            if curr_range.end > threshold_excl {
                next_range.start = threshold_excl;
                curr_range.end = threshold_excl;
                combinations += count_combinations(workflows, rule.target, next_ranges)?;
            }
        }
    }
    combinations += count_combinations(workflows, workflow.fallthrough, remainder)?;
    Ok(combinations)
}

fn parse(input: &[u8]) -> Result<Input> {
    use parsers::*;
    let nr = number::<u16>();
    let ident = take_while(0, |n, c| {
        *n += 1;
        *n <= 3 && c.is_ascii_alphabetic()
    })
    .map(|n| match *n {
        [a] => [a, 0, 0],
        [a, b] => [a, b, 0],
        [a, b, c] => [a, b, c],
        _ => unreachable!(),
    })
    .map(Ident);

    let is_lt = token((b'<', true)).or(token((b'>', false)));
    let n = any().map_res(|c| {
        Ok(match c {
            b'x' => 0,
            b'm' => 1,
            b'a' => 2,
            b's' => 3,
            _ => return Err(ParseError::TokenDoesNotMatch),
        })
    });
    let rule = n
        .and(is_lt)
        .and(nr)
        .and(token(b':').then(ident))
        .map(|(((n, is_lt), threshold), target)| Rule {
            n,
            is_lt,
            threshold,
            target,
        })
        .trailed(token(b','));

    let workflow = ident
        .and(token(b'{').then(rule.repeat_into()))
        .and(ident.trailed(token(b'}')))
        .map(|((ident, rules), fallthrough)| Workflow {
            ident,
            rules,
            fallthrough,
        });

    let part = token(b"{x=")
        .then(nr)
        .and(token(b",m=").then(nr))
        .and(token(b",a=").then(nr))
        .and(token(b",s=").then(nr))
        .trailed(token(b'}'))
        .map(|(((a, b), c), d)| [a, b, c, d]);

    let workflows = workflow.sep_by(token(b'\n'));
    let parts = part.sep_by(token(b'\n'));
    workflows
        .and(token(b"\n\n").then(parts))
        .map(|(workflows, parts)| Input { workflows, parts })
        .execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
px{a<2006:qkq,m>2090:A,rfg}
pv{a>1716:R,A}
lnx{m>1548:A,A}
rfg{s<537:gd,x>2440:R,A}
qs{s>3448:A,lnx}
qkq{x<1416:A,crn}
crn{x>2662:A,R}
in{s<1351:px,qqz}
qqz{s>2770:qs,m<1801:hdj,R}
gd{a>3333:R,R}
hdj{m>838:A,pv}

{x=787,m=2655,a=1222,s=2876}
{x=1679,m=44,a=2067,s=496}
{x=2036,m=264,a=79,s=2244}
{x=2461,m=1339,a=466,s=291}
{x=2127,m=1623,a=2188,s=1013}";

    test_pt!(parse, pt1, EXAMPLE => 19114);
    test_pt!(parse, pt2, EXAMPLE => 167409079868000);
}
