use crate::search::ttable::NodeType::PV;
use crate::{Bitboard, Board, PieceMap, Player};
use chashmap::{CHashMap, ReadGuard};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};

/// Table of moves, the key represents the game-state
pub struct TranspositionTable {
    key_to_line: CHashMap<Key, usize>,
    values: Vec<RwLock<Option<(Key, Node)>>>,
    cache_lines: plru::Cache<Box<[AtomicU64]>>,
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        Self {
            key_to_line: CHashMap::with_capacity(size),
            values: std::iter::repeat_with(|| RwLock::new(None))
                .take(size)
                .collect(),
            cache_lines: plru::create(size),
        }
    }

    pub fn get(&self, key: &Key) -> Option<Node> {
        self.key_to_line
            .get(key)
            .and_then(|guard| {
                let line = guard.deref();
                self.cache_lines.touch(*line);
                *self.values[*line].read().unwrap()
            })
            .map(|(_, node)| node)
    }

    pub fn insert(&self, key: Key, node: Node) {
        let line = self.cache_lines.replace();
        self.cache_lines.touch(line);

        let old_entry = self.values[line].write().unwrap().replace((key, node));
        if let Some((prev_key, _)) = old_entry {
            self.key_to_line.remove(&prev_key);
        }

        self.key_to_line.insert(key, line);
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
