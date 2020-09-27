#![cfg(feature = "expensive_tests")]

pub mod strategies;

use proptest::prelude::ProptestConfig;
use proptest::prelude::*;
use skakoui::Searcher;

use strategies::mate_in_1_board;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]
    #[test]
    fn searcher_can_find_mate_in_1((mut board, mating_move) in mate_in_1_board()) {
        println!("Testing board\n{}\nExpect: {}\n{:?}", board, mating_move, board);
        let mut searcher = Searcher::default();
        searcher.go(&board, Some(2));
        searcher.wait();
        let pv = searcher.principal_variation();
        let mov = *pv.first().unwrap();

        board.make_move(mov);
        let checkmate = board.checkmate();
        board.unmake_move(mov);

        assert!(checkmate, "{}\nExpect: {}\nActual: {}\nPV: {:?}", board, mating_move, mov, pv);
    }
}
