use crate::search::LOW_SCORE;
use crate::{Bitboard, Board, BoardFlags, Move, PieceType, PlayerV};
use enum_map::EnumMap;
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
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
        let hkey: u64 = hash_key(key);
        let index = hkey as usize % self.table.len();
        let hnode = self.table[index].1.load(Relaxed);
        if self.table[index].0.load(Relaxed) ^ hnode == hkey {
            Some(hnode.into())
        } else {
            None
        }
    }

    pub fn insert(&self, key: Key, node: Node) {
        let hkey: u64 = hash_key(&key);
        let hnode: u64 = node.into();
        let index = hkey as usize % self.table.len();
        self.table[index].0.store(hkey ^ hnode, Relaxed);
        self.table[index].1.store(hnode, Relaxed);
    }

    pub fn principal_variation(&self, board: &mut Board) -> Vec<Move> {
        let mut pv = vec![];
        let mut key_set = HashSet::new();
        let mut adjust = 1;

        'find_pv: loop {
            // Negate score based on who is playing
            adjust *= -1;

            let moves: Vec<Move> = board.moves().collect();

            let mut best_move = None;
            let mut best_score = LOW_SCORE;

            for mov in moves {
                let pmov = board.make_move(mov);
                let key = board.key();
                board.unmake_move(pmov);

                // Check for loop
                if !key_set.insert(key) {
                    break 'find_pv;
                }

                if let Some(entry) = self.get(&key) {
                    let score = entry.value * adjust;
                    if entry.node_type == NodeType::PV && score > best_score {
                        best_move = Some(mov);
                        best_score = score;
                    }
                }
            }

            if let Some(best) = best_move {
                let pmov = board.make_move(best);
                pv.push(pmov);
            } else {
                break;
            }
        }

        for pmov in pv.iter().rev() {
            board.unmake_move(*pmov);
        }

        // Return a totally random move if we couldn't find anything.
        // This can happen if search is stopped very quickly.
        if pv.is_empty() {
            if let Some(mov) = board.moves().next() {
                return vec![mov];
            }
        }

        pv.into_iter().map(|pmov| pmov.mov).collect()
    }
}

pub type Key = (
    EnumMap<PieceType, Bitboard>,
    EnumMap<PlayerV, Bitboard>,
    PlayerV,
    BoardFlags,
);

fn hash_key(key: &Key) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Copy, Clone, Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Node {
    pub depth: u16,
    pub node_type: NodeType,
    pub value: i32,
}

impl From<Node> for u64 {
    fn from(n: Node) -> u64 {
        n.value as u32 as u64 | (n.depth as u64) << 32 | (n.node_type as u64) << 48
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

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Serialize)]
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
