use crate::search::ttable::NodeType::PV;
use crate::{Bitboard, Board, PieceMap, Player};
use chashmap::{CHashMap, ReadGuard};
use std::sync::Arc;

/// Table of moves, the key represents the game-state
#[derive(Default, Clone)]
pub struct TranspositionTable(Arc<CHashMap<Key, Node>>);

impl TranspositionTable {
    pub fn get(&self, key: &Key) -> Option<ReadGuard<Key, Node>> {
        self.0.get(key)
    }

    pub fn insert(&self, key: Key, node: Node) {
        self.0.upsert(
            key,
            || node,
            |current| {
                if current.depth > node.depth || (current.node_type == PV && node.node_type != PV) {
                    return;
                }

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
