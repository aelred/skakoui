use crate::{Board, Move};

impl Board {
    /// Count number of moves to a certain depth.
    /// Useful for both testing correctness and performance.
    pub fn perft(&mut self, depth: usize) -> usize {
        if depth == 0 {
            return 1;
        }

        let mut count = 0;

        let moves: Vec<Move> = self.moves().collect();

        // Optimisation - skip making and un-making last moves
        if depth == 1 {
            return moves.len();
        }

        for mov in moves {
            let pmov = self.make_move(mov);
            count += self.perft(depth - 1);
            self.unmake_move(pmov);
        }

        count
    }
}
