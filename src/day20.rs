use bitvec::prelude::*;
use num::Integer;
use std::{collections::VecDeque, ops::Range};

framework::day!(20, parse => pt1, pt2);

#[derive(Debug, Clone)]
struct ParseModule<'a> {
    ty: ParseModuleType,
    name: &'a [u8],
    outputs: ArrayVec<&'a [u8], 8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseModuleType {
    None,
    FlipFlop,
    Conjunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ident(u8);

#[derive(Debug, Clone)]
struct Module {
    ty: ModuleType,
    inputs: ArrayVec<Ident, 16>,
    outputs: ArrayVec<Ident, 8>,
}

#[derive(Debug, Clone)]
enum ModuleType {
    None,
    FlipFlop(u8),
    Conjunction(Range<u8>),
}

struct Processed {
    modules: Vec<Module>,
    states: BitVec<u64, LocalBits>,
    broadcaster: Ident,
    rx: Option<Ident>,
}

fn preprocess(modules: &[ParseModule]) -> Result<Processed> {
    let mut name_map = HashMap::new();
    for name in (modules.iter().map(|m| m.name))
        .chain(modules.iter().flat_map(|m| m.outputs.iter().cloned()))
    {
        let next_idx = name_map.len();
        assert!(next_idx < 256);
        _ = name_map.try_insert(name, Ident(next_idx as u8));
    }
    let modules_len = name_map.len();
    let mut processed = (0..modules_len)
        .map(|_| Module {
            ty: ModuleType::None,
            inputs: ArrayVec::new(),
            outputs: ArrayVec::new(),
        })
        .collect_vec();
    let resolve = |name: &[u8]| {
        (name_map.get(name).cloned()).ok_or(Error::InvalidInput("module name does not exist"))
    };

    for (idx, module) in modules.iter().enumerate() {
        for &target in &module.outputs {
            let target = resolve(target)?;
            processed[idx].outputs.push(target);
            processed[target.0 as usize].inputs.push(Ident(idx as u8));
        }
    }

    let mut states = BitVec::new();
    for (idx, module) in modules.iter().enumerate() {
        match module.ty {
            ParseModuleType::None => (),
            ParseModuleType::FlipFlop => {
                processed[idx].ty = ModuleType::FlipFlop(states.len() as u8);
                states.push(false);
            }
            ParseModuleType::Conjunction => {
                let processed = &mut processed[idx];
                let start = states.len();
                let end = start + processed.inputs.len();
                assert!(end < 256);
                let range = start as u8..end as u8;
                states.extend(range.clone().map(|_| false));
                processed.ty = ModuleType::Conjunction(range);
            }
        }
    }

    let broadcaster = resolve(b"broadcaster")?;
    let rx = name_map.get(b"rx".as_slice()).cloned();
    Ok(Processed {
        modules: processed,
        states,
        broadcaster,
        rx,
    })
}

type Pulse = (bool, Ident, Ident);
type PulseQueue = VecDeque<Pulse>;

#[derive(Debug, Clone, Default)]
struct Pulses {
    queue: PulseQueue,
    lo_count: u64,
    hi_count: u64,
}

impl Pulses {
    fn send(&mut self, hi: bool, source: Ident, target: Ident) {
        if hi {
            self.hi_count += 1;
        } else {
            self.lo_count += 1;
        }
        self.queue.push_back((hi, source, target));
    }
    fn send_many(&mut self, hi: bool, source: Ident, targets: &[Ident]) {
        for &target in targets {
            self.send(hi, source, target);
        }
    }
    fn recv(&mut self) -> Option<(bool, Ident, Ident)> {
        self.queue.pop_front()
    }
}

fn pt1(modules: &[ParseModule]) -> Result<MulOutput<[u64; 2]>> {
    let Processed {
        modules,
        mut states,
        broadcaster,
        ..
    } = preprocess(modules)?;
    let mut pulses = Pulses::default();

    for _ in 0..1000 {
        pulses.send(false, broadcaster, broadcaster);
        while let Some((hi, source, target)) = pulses.recv() {
            let module = &modules[target.0 as usize];
            let output = match &module.ty {
                ModuleType::None => hi,
                &ModuleType::FlipFlop(state) if !hi => {
                    let last_state = states.get_mut(state as usize).unwrap();
                    let new_state = !*last_state;
                    last_state.commit(new_state);
                    new_state
                }
                ModuleType::Conjunction(state_range) => {
                    let input_idx = module.inputs.iter().position(|&m| m == source).unwrap();
                    let states = &mut states[state_range.start as usize..state_range.end as usize];
                    states.set(input_idx, hi);
                    !states.all()
                }
                _ => continue,
            };
            pulses.send_many(output, target, &module.outputs);
        }
    }

    Ok(MulOutput([pulses.lo_count, pulses.hi_count]))
}

fn pt2(modules: &[ParseModule]) -> Result<u64> {
    let Processed {
        modules,
        mut states,
        broadcaster,
        rx,
    } = preprocess(modules)?;
    let rx = rx.ok_or(Error::InvalidInput("missing rx"))?;
    let mut pulses = PulseQueue::new();

    let conjunction_idx = (modules.iter())
        .positions(|m| {
            matches!(m.ty, ModuleType::Conjunction(_)) && m.outputs.len() == 1 && m.outputs[0] == rx
        })
        .exactly_one()
        .map_err(|_| Error::InvalidInput("expected a single conjunction to wire into rx"))?;
    let conjunction_range = match &modules[conjunction_idx].ty {
        ModuleType::Conjunction(range) => range.clone(),
        _ => unreachable!(),
    };
    let conjunction_ident = Ident(conjunction_idx as u8);

    // During various button presses, it'll toggle on and off one of the inputs,
    // to the conjunction that wires into rx. We keep track of during which
    // button presses this happens, and then take the least-common-multiple to
    // calculate the button press during which all of the inputs will be true.
    let mut state_switches = vec![None; conjunction_range.len()];

    let mut button_presses = 0;
    loop {
        button_presses += 1;
        pulses.push_back((false, broadcaster, broadcaster));
        while let Some((hi, source, target)) = pulses.pop_front() {
            let module = &modules[target.0 as usize];
            let output = match &module.ty {
                ModuleType::None => hi,
                &ModuleType::FlipFlop(state) if !hi => {
                    let last_state = states.get_mut(state as usize).unwrap();
                    let new_state = !*last_state;
                    last_state.commit(new_state);
                    new_state
                }
                ModuleType::Conjunction(state_range) => {
                    let input_idx = module.inputs.iter().position(|&m| m == source).unwrap();
                    let states = &mut states[state_range.start as usize..state_range.end as usize];
                    if target == conjunction_ident && states[input_idx] != hi {
                        let state = &mut state_switches[input_idx];
                        if state.is_none() {
                            *state = Some(button_presses);
                            if state_switches.iter().all(|v| v.is_some()) {
                                return Ok((state_switches.iter())
                                    .map(|v| v.unwrap())
                                    .fold(1, |a, v| a.lcm(&v)));
                            }
                        }
                    }
                    states.set(input_idx, hi);
                    !states.all()
                }
                _ => continue,
            };
            pulses.extend(module.outputs.iter().map(|&out| (output, target, out)));
        }
    }
}

fn parse(input: &[u8]) -> Result<Vec<ParseModule>> {
    use parsers::*;
    let module_type = token((b'%', ParseModuleType::FlipFlop))
        .or(token((b'&', ParseModuleType::Conjunction)))
        .or(constant(ParseModuleType::None));
    let name = take_while((), |_, c| c.is_ascii_alphabetic());
    let targets = name.sep_by(token(b", "));

    let module = module_type.and(name).and(token(b" -> ").then(targets)).map(
        |((module_type, name), outputs)| ParseModule {
            ty: module_type,
            name,
            outputs,
        },
    );

    module.sep_by(token(b'\n')).execute(input)
}

tests! {
    const EXAMPLE1: &'static [u8] = b"\
broadcaster -> a, b, c
%a -> b
%b -> c
%c -> inv
&inv -> a";
    const EXAMPLE2: &'static [u8] = b"\
broadcaster -> a
%a -> inv, con
&inv -> b
%b -> con
&con -> output";

    test_pt!(parse, pt1,
        EXAMPLE1 => MulOutput([8000, 4000]),
        EXAMPLE2 => MulOutput([4250, 2750]),
    );
    // test_pt!(parse, pt2, EXAMPLE => 5);
}
