use crate::Board;
use crate::Move;
use crate::Player;
use enum_map::EnumMap;
use rand::seq::SliceRandom;
use std::collections::HashMap;

type Key = (
    EnumMap<crate::Player, EnumMap<crate::PieceType, crate::Bitboard>>,
    Player,
);

struct CacheValue {
    depth: u32,
    result: (Option<Move>, i32),
}

#[derive(Default)]
pub struct Searcher {
    cache: HashMap<Key, CacheValue>,
    quiescence_searcher: QuiescenceSearcher,
}

impl Board {
    fn key(&self) -> Key {
        (*self.bitboards(), self.player())
    }
}

trait AlphaBetaSearcher {
    fn evaluate_leaf(&mut self, board: &mut Board) -> (Option<Move>, i32);

    fn cache(&mut self) -> &mut HashMap<Key, CacheValue>;

    fn should_terminate(board: &mut Board) -> bool;

    fn run(&mut self, board: &mut Board, depth: u32) -> (Option<Move>, i32) {
        self.search(board, depth, std::i32::MIN, std::i32::MAX)
    }

    fn search(
        &mut self,
        board: &mut Board,
        depth: u32,
        alpha: i32,
        beta: i32,
    ) -> (Option<Move>, i32) {
        let key = board.key();
        if self.cache().contains_key(&key) && self.cache()[&key].depth >= depth {
            self.cache()[&key].result
        } else {
            let result = self.search_uncached(board, depth, alpha, beta);
            let cache_entry = CacheValue { depth, result };
            self.cache().insert(key, cache_entry);
            result
        }
    }

    fn search_uncached(
        &mut self,
        board: &mut Board,
        depth: u32,
        mut alpha: i32,
        beta: i32,
    ) -> (Option<Move>, i32) {
        let mut moves = board.moves().peekable();

        if moves.peek().is_none() {
            return (None, std::i32::MIN);
        }

        if depth == 0 {
            return self.evaluate_leaf(board);
        }

        let mut best_moves = vec![];
        let mut best_value = std::i32::MIN;

        for mov in moves {
            board.make_move(mov);
            let value = -self.search(board, depth - 1, -beta, -alpha).1;
            board.unmake_move(mov);

            if value > best_value {
                best_moves = vec![mov];
                best_value = value;

                alpha = i32::max(alpha, best_value);

                if alpha >= beta {
                    break;
                }
            } else if value == best_value {
                best_moves.push(mov);
            }
        }

        let best_move = best_moves.choose(&mut rand::thread_rng()).cloned();

        (best_move, best_value)
    }
}

impl Searcher {
    pub fn run(&mut self, board: &mut Board, depth: u32) -> (Option<Move>, i32) {
        AlphaBetaSearcher::run(self, board, depth)
    }
}

impl AlphaBetaSearcher for Searcher {
    fn evaluate_leaf(&mut self, board: &mut Board) -> (Option<Move>, i32) {
        self.quiescence_searcher.run(board, 1)
    }

    fn cache(&mut self) -> &mut HashMap<Key, CacheValue> {
        &mut self.cache
    }

    fn should_terminate(_: &mut Board) -> bool {
        false
    }
}

#[derive(Default)]
struct QuiescenceSearcher {
    cache: HashMap<Key, CacheValue>,
}

impl AlphaBetaSearcher for QuiescenceSearcher {
    fn evaluate_leaf(&mut self, board: &mut Board) -> (Option<Move>, i32) {
        (None, board.eval())
    }

    fn cache(&mut self) -> &mut HashMap<Key, CacheValue> {
        &mut self.cache
    }

    fn should_terminate(board: &mut Board) -> bool {
        let mut min = std::i32::MAX;
        let mut max = std::i32::MIN;

        for mov in board.moves() {
            board.make_move(mov);
            let value = board.eval();
            board.unmake_move(mov);
            min = i32::min(min, value);
            max = i32::max(max, value);
        }

        ((max - min) as u32) < 5
    }
}
