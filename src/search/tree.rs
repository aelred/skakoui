use crate::search::ttable::{Node, TranspositionTable};
use crate::{Board, Move};
use serde::Serialize;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct SearchTree {
    node: Option<Node>,
    children: HashMap<String, SearchTree, RandomState>,
}

impl SearchTree {
    pub fn from_table(board: &mut Board, table: &TranspositionTable) -> Self {
        let node = table.get(&board.key());
        let mut children = HashMap::new();

        if node.is_some() {
            let moves: Vec<Move> = board.moves().collect();

            for mov in moves {
                let pmov = board.make_move(mov);
                let child = SearchTree::from_table(board, table);
                board.unmake_move(pmov);

                children.insert(mov.to_string(), child);
            }
        }

        Self { node, children }
    }
}
