use rand::prelude::*;
use std::iter::once;

framework::day!(25, parse => pt1, pt2);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Ident([u8; 3]);

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(std::str::from_utf8(&self.0).unwrap())
    }
}

impl std::fmt::Debug for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Ident")
            .field(&std::str::from_utf8(&self.0).unwrap())
            .finish()
    }
}

#[derive(Debug, Clone)]
struct Wiring {
    a: Ident,
    b: ArrayVec<Ident, 8>,
}

const NONE: u32 = !0;

// The idea of the algorithm is to iteratively simplify the graph, by collapsing
// multiple nodes into super nodes.
#[derive(Debug, Clone)]
struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    alive_edges: Vec<u32>,
}

#[derive(Debug, Clone)]
struct Node {
    node_count: u32, // temp: only for debugging
    // Doubly-linked list to edges
    first: u32,
    last: u32,
}

#[derive(Debug, Clone, Copy)]
struct Links {
    node_idx: u32,
    prev: u32,
    next: u32,
}

#[derive(Debug, Clone)]
struct Edge {
    a: Links,
    b: Links,
    alive_idx: u32,
}

impl Edge {
    fn get_links(&self, node_idx: u32) -> &Links {
        if node_idx == self.a.node_idx {
            &self.a
        } else {
            debug_assert_eq!(node_idx, self.b.node_idx);
            &self.b
        }
    }

    fn get_links_mut(&mut self, node_idx: u32) -> &mut Links {
        if node_idx == self.a.node_idx {
            &mut self.a
        } else {
            debug_assert_eq!(node_idx, self.b.node_idx);
            &mut self.b
        }
    }

    fn get_opposite_links(&self, node_idx: u32) -> &Links {
        if node_idx == self.a.node_idx {
            &self.b
        } else {
            debug_assert_eq!(node_idx, self.b.node_idx);
            &self.a
        }
    }
}

impl Graph {
    fn from_wiring(wiring: &[Wiring]) -> Graph {
        let mut nodes = Vec::new();

        let mut node_map = HashMap::new();

        // Create a node, and map entry for each mentioned identifier
        for node in wiring
            .iter()
            .flat_map(|wiring| once(wiring.a).chain(wiring.b.iter().cloned()))
        {
            node_map.entry(node).or_insert_with(|| {
                let index = nodes.len() as u32;
                nodes.push(Node {
                    node_count: 1,
                    first: NONE,
                    last: NONE,
                });
                index
            });
        }

        // Set up the edges between all nodes
        let mut edges = Vec::new();
        for wire in wiring {
            let a_idx = node_map[&wire.a];
            for b in &wire.b {
                let b_idx = node_map[b];
                let (a, b) = nodes.get_two_mut(a_idx as usize, b_idx as usize).unwrap();

                let edge_idx = edges.len() as u32;
                edges.push(Edge {
                    a: Links {
                        node_idx: a_idx,
                        prev: a.last,
                        next: NONE,
                    },
                    b: Links {
                        node_idx: b_idx,
                        prev: b.last,
                        next: NONE,
                    },
                    alive_idx: edge_idx,
                });

                macro_rules! insert_into_linked_list {
                    ($node:ident, $node_idx:ident) => {
                        if $node.first == NONE {
                            $node.first = edge_idx;
                        } else {
                            edges[$node.last as usize].get_links_mut($node_idx).next = edge_idx;
                        }
                        $node.last = edge_idx;
                    };
                }
                insert_into_linked_list!(a, a_idx);
                insert_into_linked_list!(b, b_idx);
            }
        }

        let alive_edges = (0..edges.len() as u32).collect();
        let g = Graph {
            nodes,
            edges,
            alive_edges,
        };
        g.assert_links();
        g
    }

    fn collapse_random_edge(&mut self, rng: &mut impl Rng) {
        let edge_idx = self.alive_edges[rng.gen_range(0..self.alive_edges.len())];
        let edge = &self.edges[edge_idx as usize];
        self.combine_nodes(edge.a.node_idx, edge.b.node_idx);
    }

    /// Combines two nodes. Edges between the two nodes are removed, but
    /// all other edges are retained. This results in there potentially being
    /// multiple edges.
    fn combine_nodes(&mut self, a_idx: u32, b_idx: u32) {
        {
            let (a, b) = (self.nodes)
                .get_two_mut(a_idx as usize, b_idx as usize)
                .unwrap();

            a.node_count += b.node_count;
            b.node_count = 0;
            // a.idents.extend_from_slice(&b.idents);
            // b.idents.clear();
        }

        // Iterate over all the edges in b, and if they also connect to a,
        // remove them, otherwise, retarget them to a.
        let mut next_edge_idx = self.nodes[b_idx as usize].first;
        let a = &mut self.nodes[a_idx as usize];
        while next_edge_idx != NONE {
            let edge_idx = next_edge_idx;
            let edge = &self.edges[edge_idx as usize];
            next_edge_idx = edge.get_links(b_idx).next;
            let links_other = *edge.get_opposite_links(b_idx);

            // Interior edge, remove it from a's edge list, and unalive it
            if links_other.node_idx == a_idx {
                if links_other.prev == NONE {
                    debug_assert_eq!(edge_idx, a.first);
                    a.first = links_other.next;
                } else {
                    debug_assert_eq!(
                        edge_idx,
                        self.edges[links_other.prev as usize]
                            .get_links_mut(a_idx)
                            .next
                    );
                    self.edges[links_other.prev as usize]
                        .get_links_mut(a_idx)
                        .next = links_other.next;
                }
                if links_other.next == NONE {
                    debug_assert_eq!(edge_idx, a.last);
                    a.last = links_other.prev;
                } else {
                    debug_assert_eq!(
                        edge_idx,
                        self.edges[links_other.next as usize]
                            .get_links_mut(a_idx)
                            .prev
                    );
                    self.edges[links_other.next as usize]
                        .get_links_mut(a_idx)
                        .prev = links_other.prev;
                }

                let edge = &mut self.edges[edge_idx as usize];
                let alive_idx = edge.alive_idx;
                edge.alive_idx = NONE;

                if alive_idx as usize == self.alive_edges.len() - 1 {
                    self.alive_edges.pop();
                    continue;
                }
                self.edges[*self.alive_edges.last().unwrap() as usize].alive_idx = alive_idx;
                self.alive_edges.swap_remove(alive_idx as usize);
                continue;
            }

            // Exterior edge, insert it into a's edge list
            let links_own = self.edges[edge_idx as usize].get_links_mut(b_idx);
            links_own.prev = a.last;
            links_own.next = NONE;
            links_own.node_idx = a_idx;
            a.last = edge_idx;
            let prev = links_own.prev;
            if prev != NONE {
                self.edges[prev as usize].get_links_mut(a_idx).next = edge_idx;
            }
            if a.first == NONE {
                a.first = edge_idx;
            }
        }
        self.assert_links();
    }

    #[cfg(not(debug_assertions))]
    fn assert_links(&self) {}

    #[cfg(debug_assertions)]
    fn assert_links(&self) {
        for (node_idx, node) in self.nodes.iter().enumerate() {
            // Destroyed nodes
            if node.node_count == 0 {
                continue;
            }
            if node.first == NONE {
                assert_eq!(
                    NONE, node.last,
                    "node {node_idx} has no first, but does have last"
                );
                continue;
            }
            let node_idx = node_idx as u32;
            assert_ne!(NONE, node.last);
            let mut prev_idx = NONE;
            let mut curr_idx = node.first;
            loop {
                let edge = &self.edges[curr_idx as usize];
                let c = edge.get_links(node_idx);
                assert_eq!(node_idx, c.node_idx);
                assert_eq!(prev_idx, c.prev);
                if c.next == NONE {
                    assert_eq!(node.last, curr_idx);
                    break;
                }
                prev_idx = curr_idx;
                curr_idx = c.next;
            }
        }

        for (edge_idx, edge) in self.edges.iter().enumerate() {
            if edge.alive_idx != NONE {
                assert_eq!(edge_idx as u32, self.alive_edges[edge.alive_idx as usize]);
            }
        }
        for (alive_idx, edge_idx) in self.alive_edges.iter().cloned().enumerate() {
            assert_eq!(alive_idx as u32, self.edges[edge_idx as usize].alive_idx);
        }

        for (edge_idx, edge) in self.edges.iter().enumerate() {
            // Don't care about destroyed edges
            if edge.alive_idx == NONE {
                continue;
            }
            let edge_idx = edge_idx as u32;
            if edge.a.prev != NONE {
                assert_eq!(
                    edge_idx,
                    self.edges[edge.a.prev as usize]
                        .get_links(edge.a.node_idx)
                        .next
                );
            }
            if edge.a.next != NONE {
                assert_eq!(
                    edge_idx,
                    self.edges[edge.a.next as usize]
                        .get_links(edge.a.node_idx)
                        .prev
                );
            }
            if edge.b.prev != NONE {
                assert_eq!(
                    edge_idx,
                    self.edges[edge.b.prev as usize]
                        .get_links(edge.b.node_idx)
                        .next
                );
            }
            if edge.b.next != NONE {
                assert_eq!(
                    edge_idx,
                    self.edges[edge.b.next as usize]
                        .get_links(edge.b.node_idx)
                        .prev
                );
            }
        }
    }
}

// impl std::fmt::Display for Graph {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut edges = HashMap::<(u32, u32), u32>::new();
//         for (idx, node) in self.nodes.iter().enumerate() {
//             if node.idents.is_empty() {
//                 continue;
//             }
//             write!(f, "{idx: >2}: [{}", node.idents[0])?;

//             for ident in &node.idents[1..] {
//                 write!(f, ", {}", ident)?;
//             }

//             let mut head = node.first;
//             while head != NONE {
//                 let edge = &self.edges[head as usize];
//                 let pair = edge.a.node_idx.minmax(edge.b.node_idx);
//                 *edges.entry(pair).or_default() += 1;
//                 head = edge.get_links(idx as u32).next;
//             }

//             write!(f, "] ")?;
//         }
//         writeln!(f)?;
//         for ((a, b), count) in edges.into_iter().sorted() {
//             writeln!(f, "{a: >2} -- {b: >2} x {count}")?;
//         }

//         Ok(())
//     }
// }

fn pt1(wiring: &[Wiring]) -> Result<MulOutput<[u32; 2]>> {
    let g = Graph::from_wiring(wiring);

    let mut rng = rand::rngs::StdRng::seed_from_u64(/* e */ 271828);
    loop {
        let mut g = g.clone();
        while g.alive_edges.len() > 3 {
            g.collapse_random_edge(&mut rng);
        }
        if g.alive_edges.len() != 3 {
            continue;
        }
        let edge = &g.edges[g.alive_edges[0] as usize];
        let a = g.nodes[edge.a.node_idx as usize].node_count;
        let b = g.nodes[edge.b.node_idx as usize].node_count;
        let (a, b) = a.minmax(b);
        return Ok(MulOutput([a, b]));
    }
}

fn pt2(_: &[Wiring]) -> &'static AStr {
    b"gg"
}

fn parse(input: &[u8]) -> Result<Vec<Wiring>> {
    use parsers::*;
    let ident = take_n().map(|ident: &[u8; 3]| Ident(*ident));
    let wiring = ident.and(token(b": ").then(ident.sep_by(token(b' '))));
    let wiring = wiring.map(|(a, b)| Wiring { a, b });
    wiring.sep_by(token(b'\n')).execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
jqt: rhn xhk nvd
rsh: frs pzl lsr
xhk: hfx
cmg: qnr nvd lhk bvb
rhn: xhk bvb hfx
bvb: xhk hfx
pzl: lsr hfx nvd
qnr: nvd
ntq: jqt hfx bvb xhk
nvd: lhk
lsr: lhk
rzs: qnr cmg lsr rsh
frs: qnr lhk lsr";

    test_pt!(parse, pt1, EXAMPLE => MulOutput([6, 9]));
}
