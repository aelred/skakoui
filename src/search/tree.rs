use crate::search::ttable::{Node, TranspositionTable};
use crate::{Board, Move};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct SearchTree {
    node: Option<Node>,
    children: BTreeMap<String, SearchTree>,
}

impl SearchTree {
    pub fn from_table(board: &mut Board, table: &TranspositionTable, depth: u16) -> Self {
        let node = table.get(&board.key());
        let mut children = BTreeMap::new();

        if node.is_some() && depth > 0 {
            let moves: Vec<Move> = board.moves().collect();

            for mov in moves {
                let pmov = board.make_move(mov);
                let child = Self::from_table(board, table, depth - 1);
                board.unmake_move(pmov);

                if child.node.is_some() || !child.children.is_empty() {
                    children.insert(mov.to_string(), child);
                }
            }
        }

        Self { node, children }
    }
}
