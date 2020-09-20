use crate::{Bitboard, PieceMap, Player};
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;

/// Table of moves, the key represents the game-state
pub struct TranspositionTable {
    table: Vec<(AtomicU64, AtomicU64)>,
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        Self {
            table: std::iter::repeat_with(Default::default)
                .take(size)
                .collect(),
        }
    }

    pub fn get(&self, key: &Key) -> Option<Node> {
        let hkey: u64 = self.hash_key(&key);
        let index = hkey as usize % self.table.len();
        let hnode = self.table[index].1.load(Relaxed);
        if self.table[index].0.load(Relaxed) ^ hnode == hkey {
            Some(hnode.into())
        } else {
            None
        }
    }

    pub fn insert(&self, key: Key, node: Node) {
        let hkey: u64 = self.hash_key(&key);
        let hnode: u64 = node.into();
        let index = hkey as usize % self.table.len();
        self.table[index].0.store(hkey ^ hnode, Relaxed);
        self.table[index].1.store(hnode, Relaxed);
    }

    fn hash_key(&self, key: &Key) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

pub type Key = (PieceMap<Bitboard>, Player);

#[derive(Debug, Copy, Clone, Serialize)]
pub struct Node {
    pub depth: u16,
    pub value: i32,
    pub node_type: NodeType,
}

impl Into<u64> for Node {
    fn into(self) -> u64 {
        self.value as u32 as u64 | (self.depth as u64) << 32 | (self.node_type as u64) << 48
    }
}

impl From<u64> for Node {
    fn from(n: u64) -> Self {
        Self {
            value: n as u32 as i32,
            depth: (n >> 32) as u16,
            node_type: (n >> 48).into(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize)]
pub enum NodeType {
    /// Principal variation node, fully explored and value is exact
    PV = 0,
    /// Cut node, or fail-high node, was beta-cutoff, value is a lower bound
    Cut = 1,
    /// All-node, or fail-low node, no moves exceeded alpha, value is an upper bound
    All = 2,
}

impl From<u64> for NodeType {
    fn from(n: u64) -> Self {
        match n {
            0 => Self::PV,
            1 => Self::Cut,
            2 => Self::All,
            n => panic!("Weird NodeType: {} - definitely a bug", n),
        }
    }
}
