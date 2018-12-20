use crate::Board;
use crate::Move;
use crate::Player;
use enum_map::EnumMap;
use rand::seq::SliceRandom;
use std::collections::HashMap;

const HIGH_SCORE: i32 = std::i32::MAX;
const LOW_SCORE: i32 = -HIGH_SCORE;

type Key = (
    EnumMap<crate::Player, EnumMap<crate::PieceType, crate::Bitboard>>,
    Player,
);

struct CacheValue {
    depth: u32,
    result: i32,
}

#[derive(Default)]
pub struct Searcher {
    cache: HashMap<Key, CacheValue>,
}

impl Board {
    fn key(&self) -> Key {
        (*self.bitboards(), self.player())
    }
}

impl Searcher {
    const DEPTH: u32 = 4;

    pub fn run(&mut self, board: &mut Board) -> (Option<Move>, i32) {
        let mut moves = board.moves().peekable();

        if moves.peek().is_none() {
            return (None, LOW_SCORE);
        }

        let mut alpha = LOW_SCORE;
        let mut best_moves = vec![];

        for mov in moves {
            board.make_move(mov);
            let value = -self.search(board, Self::DEPTH, LOW_SCORE, -alpha);
            board.unmake_move(mov);

            if value > alpha {
                alpha = value;
                best_moves = vec![mov];
            } else if value == alpha {
                best_moves.push(mov);
            }
        }

        let best_move = best_moves.choose(&mut rand::thread_rng()).cloned();

        (best_move, alpha)
    }

    fn search(&mut self, board: &mut Board, depth: u32, alpha: i32, beta: i32) -> i32 {
        let key = board.key();
        if self.cache.contains_key(&key) && self.cache[&key].depth >= depth {
            self.cache[&key].result
        } else {
            let result = self.search_uncached(board, depth, alpha, beta);
            let cache_entry = CacheValue { depth, result };
            self.cache.insert(key, cache_entry);
            result
        }
    }

    fn search_uncached(&mut self, board: &mut Board, depth: u32, mut alpha: i32, beta: i32) -> i32 {
        if depth == 0 {
            return Self::quiesce(board, alpha, beta);
        }

        let mut moves = board.moves().peekable();

        if moves.peek().is_none() {
            return LOW_SCORE;
        }

        for mov in moves {
            board.make_move(mov);
            let value = -self.search(board, depth - 1, -beta, -alpha);
            board.unmake_move(mov);

            if value >= beta {
                return beta;
            }
            if value > alpha {
                alpha = value;
            }
        }

        alpha
    }

    fn quiesce(board: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        let stand_pat = board.eval();

        if stand_pat >= beta {
            return beta;
        }

        if alpha < stand_pat {
            alpha = stand_pat;
        }

        for mov in board.moves() {
            board.make_move(mov);
            // Only evaluate captures
            if board.eval() == stand_pat {
                board.unmake_move(mov);
                continue;
            }
            let value = -Self::quiesce(board, -beta, -alpha);
            board.unmake_move(mov);

            if value >= beta {
                return beta;
            }
            if value > alpha {
                alpha = value;
            }
        }

        alpha
    }
}
