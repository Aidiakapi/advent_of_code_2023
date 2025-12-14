use ahash::RandomState;
use std::collections::VecDeque;
use std::hash::Hash;
use std::{collections::BinaryHeap, ops::Add};
use hashbrown::hash_map::RawEntryMut;

type HashMap<K, V> = hashbrown::HashMap<K, V, RandomState>;

macro_rules! nodes_container {
    ($name:ident, $underlying:ident, $push_fn:ident) => {
        #[repr(transparent)]
        pub struct $name<N> {
            data: $underlying<N>,
        }
        impl<N> $name<N> {
            #[inline]
            pub fn push(&mut self, value: N) {
                self.data.$push_fn(value);
            }
        }

        impl<N> Extend<N> for $name<N> {
            #[inline]
            fn extend<T: IntoIterator<Item = N>>(&mut self, iter: T) {
                self.data.extend(iter);
            }
        }
    };
}

nodes_container!(DfsNodes, Vec, push);
nodes_container!(BfsNodes, VecDeque, push_back);

/// Depth-first search, starting at a particular node, and visiting
/// all indicated neighbors.
/// Cyclic graphs will result in a hang.
/// Return Some(value) at any point to halt the process.
/// Child nodes added during each visit are visited in reverse insertion order.
pub fn dfs<N, O, F>(init: N, mut visit: F) -> Option<O>
where
    F: FnMut(N, &mut DfsNodes<N>) -> Option<O>,
{
    let mut nodes = DfsNodes { data: Vec::new() };
    nodes.data.push(init);
    while let Some(node) = nodes.data.pop() {
        if let Some(result) = visit(node, &mut nodes) {
            return Some(result);
        }
    }
    None
}

pub fn bfs<N, O, F>(init: N, mut visit: F) -> Option<O>
where
    F: FnMut(N, &mut BfsNodes<N>) -> Option<O>,
{
    let mut nodes = BfsNodes {
        data: VecDeque::new(),
    };
    nodes.push(init);
    while let Some(node) = nodes.data.pop_front() {
        if let Some(output) = visit(node, &mut nodes) {
            return Some(output);
        }
    }
    None
}

pub struct AStarInfo<N, C> {
    pub path: Vec<(N, C)>,
    pub total_cost: C,
}

pub fn astar<N, C, FN, FH, FC>(
    start: N,
    mut next: FN,
    mut heuristic: FH,
    mut is_target: FC,
) -> Option<AStarInfo<N, C>>
where
    N: Clone + Hash + Eq,
    C: Ord + Copy + Add<Output = C> + Default,
    FN: FnMut(&N, &mut Vec<(N, C)>),
    FH: FnMut(&N) -> C,
    FC: FnMut(&N) -> bool,
{
    struct Pending<N, C: Ord + Copy + Add<Output = C> + Default> {
        cost: C,
        cost_and_heuristic: C,
        node: N,
        previous: Option<N>,
    }

    impl<N, C: Ord + Copy + Add<Output = C> + Default> PartialEq for Pending<N, C> {
        fn eq(&self, other: &Self) -> bool {
            self.cost_and_heuristic == other.cost_and_heuristic
        }
    }
    impl<N, C: Ord + Copy + Add<Output = C> + Default> Eq for Pending<N, C> {}
    impl<N, C: Ord + Copy + Add<Output = C> + Default> PartialOrd for Pending<N, C> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl<N, C: Ord + Copy + Add<Output = C> + Default> Ord for Pending<N, C> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            other.cost_and_heuristic.cmp(&self.cost_and_heuristic)
        }
    }

    let mut pending = BinaryHeap::new();
    pending.push(Pending {
        cost: C::default(),
        cost_and_heuristic: heuristic(&start),
        node: start,
        previous: None,
    });
    let mut visited = HashMap::<N, (C, Option<N>)>::with_hasher(RandomState::new());
    let mut next_nodes = Vec::new();
    while let Some(entry) = pending.pop() {
        if is_target(&entry.node) {
            let total_cost = entry.cost;
            let mut path = Vec::new();
            path.push((entry.node, total_cost));
            let mut previous = entry.previous;
            while let Some(node) = previous {
                let cost;
                (cost, previous) = visited.remove(&node).unwrap();
                path.push((node, cost));
            }

            path.reverse();
            return Some(AStarInfo { total_cost, path });
        }
        let node = match visited.raw_entry_mut().from_key(&entry.node) {
            RawEntryMut::Occupied(mut previously_visited) => {
                let previous = previously_visited.get_mut();
                if previous.0 <= entry.cost {
                    continue;
                }
                previous.0 = entry.cost;
                previous.1 = entry.previous;
                previously_visited.insert_key(entry.node);
                previously_visited.into_key()
            }
            RawEntryMut::Vacant(slot) => slot.insert(entry.node, (entry.cost, entry.previous)).0,
        };
        next(node, &mut next_nodes);
        for (next_node, next_cost) in next_nodes.drain(..) {
            let cost = entry.cost + next_cost;
            pending.push(Pending {
                cost,
                cost_and_heuristic: cost + heuristic(&next_node),
                node: next_node,
                previous: Some(node.clone()),
            });
        }
    }
    None
}

pub fn astar_path_cost<N, C>(
    starts: impl FnOnce(&mut Vec<(N, C)>),
    mut next: impl FnMut(&N, &mut Vec<(N, C)>),
    mut heuristic: impl FnMut(&N) -> C,
    mut is_target: impl FnMut(&N) -> bool,
) -> Option<C>
where
    N: Hash + Eq,
    C: Ord + Copy + Add<Output = C> + Default,
{
    struct Pending<N, C: Ord + Copy + Add<Output = C> + Default> {
        cost: C,
        cost_and_heuristic: C,
        node: N,
    }

    impl<N, C: Ord + Copy + Add<Output = C> + Default> PartialEq for Pending<N, C> {
        fn eq(&self, other: &Self) -> bool {
            self.cost_and_heuristic == other.cost_and_heuristic
        }
    }
    impl<N, C: Ord + Copy + Add<Output = C> + Default> Eq for Pending<N, C> {}
    impl<N, C: Ord + Copy + Add<Output = C> + Default> PartialOrd for Pending<N, C> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl<N, C: Ord + Copy + Add<Output = C> + Default> Ord for Pending<N, C> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            other.cost_and_heuristic.cmp(&self.cost_and_heuristic)
        }
    }

    let mut next_nodes = Vec::new();
    let mut pending = BinaryHeap::new();
    starts(&mut next_nodes);
    for (start, cost) in next_nodes.drain(..) {
        pending.push(Pending {
            cost,
            cost_and_heuristic: cost + heuristic(&start),
            node: start,
        });
    }
    let mut visited = HashMap::<N, C>::with_hasher(RandomState::new());
    while let Some(entry) = pending.pop() {
        if is_target(&entry.node) {
            return Some(entry.cost);
        }
        let node = match visited.raw_entry_mut().from_key(&entry.node) {
            RawEntryMut::Occupied(mut previously_visited) => {
                let previous = previously_visited.get_mut();
                if *previous <= entry.cost {
                    continue;
                }
                *previous = entry.cost;
                previously_visited.insert_key(entry.node);
                previously_visited.into_key()
            }
            RawEntryMut::Vacant(slot) => slot.insert(entry.node, entry.cost).0,
        };
        next(node, &mut next_nodes);
        for (next_node, next_cost) in next_nodes.drain(..) {
            let cost = entry.cost + next_cost;
            pending.push(Pending {
                cost,
                cost_and_heuristic: cost + heuristic(&next_node),
                node: next_node,
            });
        }
    }
    None
}
