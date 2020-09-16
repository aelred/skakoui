use crate::search::ttable::NodeType::PV;
use crate::{Bitboard, Board, PieceMap, Player};
use chashmap::{CHashMap, ReadGuard};
use std::ops::Deref;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

/// Table of moves, the key represents the game-state
pub struct TranspositionTable {
    map: CHashMap<Key, (Node, usize)>,
    cache_lines: plru::Cache<Box<[AtomicU64]>>,
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        Self {
            map: CHashMap::with_capacity(size),
            cache_lines: plru::create(size),
        }
    }

    pub fn get(&self, key: &Key) -> Option<Node> {
        self.map.get(key).map(|guard| {
            let (node, line) = guard.deref();
            self.cache_lines.touch(*line);
            *node
        })
    }

    pub fn insert(&self, key: Key, node: Node) {
        self.map.upsert(
            key,
            || {
                let line = self.cache_lines.replace();
                self.cache_lines.touch(line);
                (node, line)
            },
            |(current, line)| {
                if current.depth > node.depth || (current.node_type == PV && node.node_type != PV) {
                    return;
                }

                self.cache_lines.touch(*line);
                *current = node;
            },
        );
    }
}

pub type Key = (PieceMap<Bitboard>, Player);

#[derive(Debug, Copy, Clone)]
pub struct Node {
    pub depth: u32,
    pub value: i32,
    pub node_type: NodeType,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum NodeType {
    /// Principal variation node, fully explored and value is exact
    PV,
    /// Cut node, or fail-high node, was beta-cutoff, value is a lower bound
    Cut,
    /// All-node, or fail-low node, no moves exceeded alpha, value is an upper bound
    All,
}
