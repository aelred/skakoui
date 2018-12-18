use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::hash::Hash;

pub trait State: Hash + Eq + Clone {
    type Move: Copy;

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
    quiescence_searcher: QuiescenceSearcher<S>,
}

impl<S: State> Default for Searcher<S> {
    fn default() -> Self {
        Self {
            cache: HashMap::default(),
            quiescence_searcher: QuiescenceSearcher::default(),
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

trait AlphaBetaSearcher<S: State> {
    fn evaluate_leaf<P: Player>(&mut self, state: &S) -> (Option<S::Move>, i32);

    fn cache(&mut self) -> &mut HashMap<S, CacheValue<S::Move>>;

    fn should_terminate(state: &S) -> bool;

    fn run<P: Player>(&mut self, state: &S, depth: u32) -> (Option<S::Move>, i32) {
        self.search::<P>(state, depth, std::i32::MIN, std::i32::MAX)
    }

    fn search<P: Player>(
        &mut self,
        state: &S,
        depth: u32,
        alpha: i32,
        beta: i32,
    ) -> (Option<S::Move>, i32) {
        if self.cache().contains_key(state) && self.cache()[state].depth >= depth {
            self.cache()[state].result
        } else {
            let result = self.search_uncached::<P>(state, depth, alpha, beta);
            let cache_entry = CacheValue { depth, result };
            self.cache().insert(state.clone(), cache_entry);
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
            return self.evaluate_leaf::<P>(state);
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
}

impl<S: State> Searcher<S> {
    pub fn run(
        &mut self,
        state: &S,
        depth: u32,
        maximising_player: bool,
    ) -> (Option<S::Move>, i32) {
        if maximising_player {
            AlphaBetaSearcher::run::<Maximising>(self, state, depth)
        } else {
            AlphaBetaSearcher::run::<Minimising>(self, state, depth)
        }
    }
}

impl<S: State> AlphaBetaSearcher<S> for Searcher<S> {
    fn evaluate_leaf<P: Player>(&mut self, state: &S) -> (Option<S::Move>, i32) {
        self.quiescence_searcher.run::<P>(state, 1)
    }

    fn cache(&mut self) -> &mut HashMap<S, CacheValue<S::Move>> {
        &mut self.cache
    }

    fn should_terminate(_: &S) -> bool {
        false
    }
}

struct QuiescenceSearcher<S: State> {
    cache: HashMap<S, CacheValue<S::Move>>,
}

impl<S: State> Default for QuiescenceSearcher<S> {
    fn default() -> Self {
        Self {
            cache: HashMap::default(),
        }
    }
}

impl<S: State> AlphaBetaSearcher<S> for QuiescenceSearcher<S> {
    fn evaluate_leaf<P: Player>(&mut self, state: &S) -> (Option<<S as State>::Move>, i32) {
        (None, state.eval())
    }

    fn cache(&mut self) -> &mut HashMap<S, CacheValue<S::Move>> {
        &mut self.cache
    }

    fn should_terminate(state: &S) -> bool {
        state.quiet()
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
