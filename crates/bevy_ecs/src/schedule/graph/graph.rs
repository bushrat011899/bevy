use super::NodeId;
use indexmap::{IndexMap, IndexSet};
use alloc::vec::Vec;
use bevy_utils::AHasher;
use core::hash::BuildHasherDefault;

pub type DiGraph = Graph<true>;
pub type UnGraph = Graph<false>;

/// A graph-map data structure based on [`petgraph`]'s [`GraphMap`].
///
/// [`petgraph`]: https://docs.rs/petgraph/0.6.5/petgraph/
/// [`GraphMap`]: https://docs.rs/petgraph/0.6.5/petgraph/graphmap/struct.GraphMap.html
#[derive(Clone)]
pub struct Graph<const DIRECTED: bool = false> {
    nodes: IndexMap<NodeId, Vec<(NodeId, Direction)>, BuildHasherDefault<AHasher>>,
    edges: IndexSet<(NodeId, NodeId), BuildHasherDefault<AHasher>>,
}

impl<const DIRECTED: bool> Default for Graph<DIRECTED> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const DIRECTED: bool> core::fmt::Debug for Graph<DIRECTED> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Graph {{")?;

        for (a, list) in self.nodes.iter() {
            let mut iter = list.iter();

            let Some((b, b_link)) = iter.next() else {
                writeln!(f, "\t{a:?},")?;
                continue;
            };

            let Some((c, c_link)) = iter.next() else {
                writeln!(f, "\t{a:?} {b_link} {b:?},")?;
                continue;
            };

            writeln!(f, "\t{a:?}:")?;
            writeln!(f, "\t\t{b_link} {b:?}")?;
            writeln!(f, "\t\t{c_link} {c:?}")?;

            for (b, b_link) in iter {
                writeln!(f, "\t\t{b_link} {b:?}")?;
            }
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}

impl<const DIRECTED: bool> Graph<DIRECTED> {
    pub fn new() -> Self {
        Self {
            nodes: IndexMap::with_hasher(Default::default()),
            edges: IndexSet::with_hasher(Default::default()),
        }
    }

    pub const fn is_directed(&self) -> bool {
        DIRECTED
    }

    const fn proper_edge(a: NodeId, b: NodeId) -> (NodeId, NodeId) {
        if !DIRECTED && matches!(a.cmp(&b), core::cmp::Ordering::Greater) {
            (b, a)
        } else {
            (a, b)
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }

    pub fn contains_node(&self, node: NodeId) -> bool {
        self.nodes.contains_key(&node)
    }

    pub fn contains_edge(&self, a: NodeId, b: NodeId) -> bool {
        self.edges.contains(&Self::proper_edge(a, b))
    }

    pub fn nodes(&self) -> impl DoubleEndedIterator<Item = NodeId> + '_ {
        self.nodes.keys().copied()
    }

    pub fn edges(&self, a: NodeId) -> impl DoubleEndedIterator<Item = (NodeId, NodeId)> + '_ {
        self.neighbors(a).map(move |b| (a, b))
    }

    pub fn to_index(&self, node: NodeId) -> Option<usize> {
        self.nodes.get_index_of(&node)
    }

    pub fn edges_directed(
        &self,
        a: NodeId,
        direction: Direction,
    ) -> impl DoubleEndedIterator<Item = (NodeId, NodeId)> + '_ {
        self.neighbors_directed(a, direction).map(move |b| (a, b))
    }

    pub fn neighbors_directed(
        &self,
        a: NodeId,
        direction: Direction,
    ) -> impl DoubleEndedIterator<Item = NodeId> + '_ {
        use Direction::{Both, In, Out};
        let iter = match self.nodes.get(&a) {
            Some(list) => list.iter(),
            None => [].iter(),
        };

        iter.filter(move |(_, d)| matches!((direction, d), (_, Both) | (Out, Out) | (In, In)))
            .map(|(node, _)| node)
            .copied()
    }

    pub fn neighbors(&self, a: NodeId) -> impl DoubleEndedIterator<Item = NodeId> + '_ {
        self.neighbors_directed(a, Direction::Out)
    }

    pub fn all_edges(&self) -> impl DoubleEndedIterator<Item = (NodeId, NodeId)> + '_ {
        self.edges.iter().copied()
    }

    pub fn add_node(&mut self, node: NodeId) {
        self.nodes.entry(node).or_default();
    }

    pub fn remove_node(&mut self, a: NodeId) -> bool {
        let links = match self.nodes.swap_remove(&a) {
            None => return false,
            Some(sus) => sus,
        };
        for (b, dir) in links {
            if let Some(list) = self.nodes.get_mut(&b) {
                list.retain(|&(c, _)| c != a);
            }

            match dir {
                Direction::Out => {
                    self.edges.swap_remove(&(a, b));
                }
                Direction::In => {
                    self.edges.swap_remove(&(b, a));
                }
                Direction::Both => {
                    self.edges.swap_remove(&(a, b));
                    self.edges.swap_remove(&(b, a));
                }
            }
        }
        true
    }

    pub fn add_edge(&mut self, a: NodeId, b: NodeId) -> bool {
        let (a, b) = Self::proper_edge(a, b);

        if self.edges.insert((a, b)) {
            self.nodes.entry(a).or_default().push((
                b,
                if DIRECTED {
                    Direction::Out
                } else {
                    Direction::Both
                },
            ));

            if a != b {
                self.nodes.entry(b).or_default().push((
                    a,
                    if DIRECTED {
                        Direction::In
                    } else {
                        Direction::Both
                    },
                ));
            }

            true
        } else {
            false
        }
    }

    pub fn remove_edge(&mut self, a: NodeId, b: NodeId) -> bool {
        let (a, b) = Self::proper_edge(a, b);

        if self.edges.swap_remove(&(a, b)) {
            if let Some(list) = self.nodes.get_mut(&a) {
                list.retain_mut(|(c, direction)| {
                    if *c == b {
                        match direction {
                            Direction::Out | Direction::In => false,
                            Direction::Both => {
                                *direction = Direction::In;
                                true
                            }
                        }
                    } else {
                        true
                    }
                })
            }

            if let Some(list) = self.nodes.get_mut(&b) {
                list.retain_mut(|(c, direction)| {
                    if *c == a {
                        match direction {
                            Direction::Out | Direction::In => false,
                            Direction::Both => {
                                *direction = Direction::Out;
                                true
                            }
                        }
                    } else {
                        true
                    }
                });
            };

            true
        } else {
            false
        }
    }
}

impl DiGraph {
    pub fn for_each_scc(&self, f: impl FnMut(&[NodeId])) {
        let mut tarjan_scc = super::tarjan_scc::TarjanScc::new();
        tarjan_scc.run(self, f);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Out,
    In,
    Both,
}

impl core::fmt::Debug for Direction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Out => write!(f, "-->"),
            Self::In => write!(f, "<--"),
            Self::Both => write!(f, "<->"),
        }
    }
}

impl core::fmt::Display for Direction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <Self as core::fmt::Debug>::fmt(self, f)
    }
}
