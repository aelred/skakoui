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

    #[test]
    fn legal_moves_never_leave_king_in_check((mut board, mov) in board_and_move(arb_board())) {
        let me = board.player();
        board.make_move(mov);
        assert!(!board.check(me));
    }
}
