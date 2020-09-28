#![cfg(feature = "expensive_tests")]

pub mod strategies;

use proptest::prelude::*;
use strategies::*;

proptest! {
    #[test]
    fn legal_moves_can_be_unmade((board_before, mov) in board_and_move(arb_board())) {
        let mut board_after = board_before.clone();
        let pmov = board_after.make_move(mov);
        board_after.unmake_move(pmov);
        assert_eq!(board_before, board_after)
    }
}
