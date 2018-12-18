use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::hash::Hash;

pub trait State {
    type Move;

    fn moves(&self) -> Box<dyn Iterator<Item = Self::Move>>;

    fn make_move(&mut self, mov: Self::Move);

    fn eval(&self) -> i32;

    fn quiet(&self) -> bool;
}

struct CacheValue<M> {
    depth: u32,
    result: (Option<M>, i32),
}

pub struct Searcher<S: State> {
    cache: HashMap<S, CacheValue<S::Move>>,
}

impl<S: State + Hash + Eq> Default for Searcher<S> {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

trait Player {
    type Opp: Player;
    const WORST_SCORE: i32;
    fn set_alpha_beta(alpha: &mut i32, beta: &mut i32, score: i32);
    fn better_score(new_score: i32, old_score: i32) -> bool;
}

struct Maximising;
impl Player for Maximising {
    type Opp = Minimising;
    const WORST_SCORE: i32 = std::i32::MIN;

    fn set_alpha_beta(alpha: &mut i32, _: &mut i32, score: i32) {
        *alpha = i32::max(*alpha, score);
    }

    fn better_score(new_score: i32, old_score: i32) -> bool {
        new_score > old_score
    }
}

struct Minimising;
impl Player for Minimising {
    type Opp = Maximising;
    const WORST_SCORE: i32 = std::i32::MAX;

    fn set_alpha_beta(_: &mut i32, beta: &mut i32, score: i32) {
        *beta = i32::min(*beta, score);
    }

    fn better_score(new_score: i32, old_score: i32) -> bool {
        new_score < old_score
    }
}

impl<S: State + Hash + Eq + Clone> Searcher<S>
where
    S::Move: Copy,
{
    pub fn run(
        &mut self,
        state: &S,
        depth: u32,
        maximising_player: bool,
    ) -> (Option<S::Move>, i32) {
        if maximising_player {
            self.search::<Maximising>(state, depth, std::i32::MIN, std::i32::MAX)
        } else {
            self.search::<Minimising>(state, depth, std::i32::MIN, std::i32::MAX)
        }
    }

    fn search<P: Player>(
        &mut self,
        state: &S,
        depth: u32,
        alpha: i32,
        beta: i32,
    ) -> (Option<S::Move>, i32) {
        if self.cache.contains_key(state) && self.cache[state].depth >= depth {
            self.cache[state].result
        } else {
            let result = self.search_uncached::<P>(state, depth, alpha, beta);
            let cache_entry = CacheValue { depth, result };
            self.cache.insert(state.clone(), cache_entry);
            result
        }
    }

    fn search_uncached<P: Player>(
        &mut self,
        state: &S,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
    ) -> (Option<S::Move>, i32) {
        let mut moves = state.moves().peekable();

        if moves.peek().is_none() {
            return (None, P::WORST_SCORE);
        }

        if depth == 0 {
            return self.quiescence_search::<P>(state, 1, std::i32::MIN, std::i32::MAX);
        }

        let mut best_moves = vec![];
        let mut best_value = P::WORST_SCORE;

        for mov in moves {
            let mut child = state.clone();
            child.make_move(mov);
            let value = self.search::<P::Opp>(&child, depth - 1, alpha, beta).1;
            if P::better_score(value, best_value) {
                best_moves = vec![mov];
                best_value = value;

                P::set_alpha_beta(&mut alpha, &mut beta, best_value);
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

    fn quiescence_search<P: Player>(
        &self,
        state: &S,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
    ) -> (Option<S::Move>, i32) {
        let mut moves = state.moves().peekable();

        if moves.peek().is_none() {
            return (None, P::WORST_SCORE);
        }

        if depth == 0 || state.quiet() {
            return (None, state.eval());
        }

        let mut best_move = None;
        let mut best_value = P::WORST_SCORE;

        for mov in moves {
            let mut child = state.clone();
            child.make_move(mov);
            let value = self
                .quiescence_search::<P::Opp>(&child, depth - 1, alpha, beta)
                .1;
            if P::better_score(value, best_value) {
                best_move = Some(mov);
                best_value = value;

                P::set_alpha_beta(&mut alpha, &mut beta, best_value);
                if alpha >= beta {
                    break;
                }
            }
        }

        (best_move, best_value)
    }
}

impl State for crate::Board {
    type Move = crate::Move;

    fn moves(&self) -> Box<dyn Iterator<Item = crate::Move>> {
        Box::new(crate::Board::moves(self))
    }

    fn make_move(&mut self, mov: crate::Move) {
        crate::Board::make_move(self, mov);
    }

    fn eval(&self) -> i32 {
        crate::Board::eval(self)
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

impl crate::Board {
    fn states(&self) -> impl Iterator<Item = crate::Board> {
        let this = self.clone();
        self.moves().map(move |mov| {
            let mut child = this.clone();
            child.make_move(mov);
            child
        })
    }
}
