use crate::Board;
use crate::Move;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::hash::Hash;

pub trait State<M> {
    fn moves(&self) -> Box<dyn Iterator<Item = M>>;

    fn make_move(&mut self, mov: M);

    fn eval(&self) -> i32;

    fn quiet(&self) -> bool;
}

struct CacheValue<M> {
    depth: u32,
    result: (Option<M>, i32),
}

pub struct Searcher<M, S> {
    cache: HashMap<S, CacheValue<M>>,
}

impl<M, S: Hash + Eq> Default for Searcher<M, S> {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

impl<M: Copy, S: State<M> + Hash + Eq + Clone> Searcher<M, S> {
    pub fn run(&mut self, state: &S, depth: u32, maximising_player: bool) -> (Option<M>, i32) {
        self.search(
            state,
            depth,
            std::i32::MIN,
            std::i32::MAX,
            maximising_player,
        )
    }

    fn search(
        &mut self,
        state: &S,
        depth: u32,
        alpha: i32,
        beta: i32,
        maximising_player: bool,
    ) -> (Option<M>, i32) {
        if self.cache.contains_key(state) && self.cache[state].depth >= depth {
            self.cache[state].result
        } else {
            let result = self.search_uncached(state, depth, alpha, beta, maximising_player);
            let cache_entry = CacheValue { depth, result };
            self.cache.insert(state.clone(), cache_entry);
            result
        }
    }

    fn search_uncached(
        &mut self,
        state: &S,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
        maximising_player: bool,
    ) -> (Option<M>, i32) {
        let mut moves = state.moves().peekable();

        if moves.peek().is_none() {
            return (
                None,
                if maximising_player {
                    std::i32::MIN
                } else {
                    std::i32::MAX
                },
            );
        }

        if depth == 0 {
            return self.quiescence_search(
                state,
                1,
                std::i32::MIN,
                std::i32::MAX,
                maximising_player,
            );
        }

        let mut best_moves = vec![];
        let mut best_value;

        if maximising_player {
            best_value = std::i32::MIN;

            for mov in moves {
                let mut child = state.clone();
                child.make_move(mov);
                let value = self.search(&child, depth - 1, alpha, beta, false).1;
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
        } else {
            best_value = std::i32::MAX;

            for mov in moves {
                let mut child = state.clone();
                child.make_move(mov);
                let value = self.search(&child, depth - 1, alpha, beta, true).1;
                if value < best_value {
                    best_moves = vec![mov];
                    best_value = value;

                    beta = i32::min(beta, best_value);
                    if alpha >= beta {
                        break;
                    }
                } else if value == best_value {
                    best_moves.push(mov);
                }
            }
        }

        let best_move = best_moves.choose(&mut rand::thread_rng()).cloned();

        (best_move, best_value)
    }

    fn quiescence_search(
        &self,
        state: &S,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
        maximising_player: bool,
    ) -> (Option<M>, i32) {
        let mut moves = state.moves().peekable();

        if moves.peek().is_none() {
            return (
                None,
                if maximising_player {
                    std::i32::MIN
                } else {
                    std::i32::MAX
                },
            );
        }

        if depth == 0 || state.quiet() {
            return (None, state.eval());
        }

        let mut best_move = None;
        let mut best_value;

        if maximising_player {
            best_value = std::i32::MIN;

            for mov in moves {
                let mut child = state.clone();
                child.make_move(mov);
                let value = self
                    .quiescence_search(&child, depth - 1, alpha, beta, false)
                    .1;
                if value > best_value {
                    best_move = Some(mov);
                    best_value = value;

                    alpha = i32::max(alpha, best_value);
                    if alpha >= beta {
                        break;
                    }
                }
            }
        } else {
            best_value = std::i32::MAX;

            for mov in moves {
                let mut child = state.clone();
                child.make_move(mov);
                let value = self
                    .quiescence_search(&child, depth - 1, alpha, beta, true)
                    .1;
                if value < best_value {
                    best_move = Some(mov);
                    best_value = value;

                    beta = i32::min(beta, best_value);
                    if alpha >= beta {
                        break;
                    }
                }
            }
        }

        (best_move, best_value)
    }
}

impl State<Move> for Board {
    fn moves(&self) -> Box<dyn Iterator<Item = Move>> {
        Box::new(Board::moves(self))
    }

    fn make_move(&mut self, mov: Move) {
        Board::make_move(self, mov);
    }

    fn eval(&self) -> i32 {
        Board::eval(self)
    }

    fn quiet(&self) -> bool {
        let mut min = std::i32::MAX;
        let mut max = std::i32::MIN;

        for state in self.states() {
            let value = state.eval();
            min = i32::min(min, value);
            max = i32::max(max, value);
        }

        ((max - min) as u32) < 5
    }
}

impl Board {
    fn states(&self) -> impl Iterator<Item = Board> {
        let this = self.clone();
        self.moves().map(move |mov| {
            let mut child = this.clone();
            child.make_move(mov);
            child
        })
    }
}
