mod strategies;

use chess::Searcher;
use proptest::prelude::*;
use std::time::Duration;
use strategies::mate_in_1_board;

proptest! {
    #[test]
    fn searcher_can_find_mate_in_1_in_a_second((mut board, mating_move) in mate_in_1_board()) {
        let mut searcher = Searcher::default();
        searcher.go(&board);
        std::thread::sleep(Duration::from_secs(10));
        searcher.stop();
        let pv = searcher.principal_variation();
        let mov = *pv.first().unwrap();

        board.make_move(mov);
        let checkmate = board.checkmate();
        board.unmake_move(mov);

        assert!(checkmate, "{}\nExpect: {}\nActual: {}\nPV: {:?}", board, mating_move, mov, pv);
    }
}
