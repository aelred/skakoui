#![cfg(feature = "expensive_tests")]

pub mod strategies;

use proptest::prelude::*;
use strategies::*;

proptest! {
    #[test]
    fn legal_moves_can_be_unmade((board_before, mov) in board_and_move(arb_board())) {
        let mut board_after = board_before.clone();
        board_after.make_move(mov);
        board_after.unmake_move(mov);
        assert_eq!(board_before, board_after)
    }
}
