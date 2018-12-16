use crate::Board;
use crate::Move;
use crate::Player;
use crate::WhitePlayer;
use crate::BlackPlayer;
use crate::PlayerType;
use crate::Piece;
use crate::PieceType;
use crate::bitboards;

impl Board {
    pub fn moves(&self) -> impl Iterator<Item = Move> {
        match self.player() {
            Player::White => self.moves_for_player::<WhitePlayer>(),
            Player::Black => self.moves_for_player::<BlackPlayer>(),
        }
    }

    fn moves_for_player<P: PlayerType + 'static>(&self) -> Box<dyn Iterator<Item = Move>> {
        Box::new(self.pawn_moves::<P>())
    }

    fn pawn_moves<P: PlayerType>(&self) -> impl Iterator<Item = Move> {
        let piece = Piece::new(P::PLAYER, PieceType::Pawn);

        let initial_board = self.bitboard_piece(piece);
        let free_spaces = !self.occupancy();

        let bitboard = P::advance_bitboard(initial_board);

        let pushes = bitboard & free_spaces;

        let pushes_iter = pushes.squares().map(move |target| {
            let source = target.shift_rank(-P::DIRECTION);
            Move::new(piece, source, target)
        });

        let double_mask = bitboards::RANKS[P::PAWN_RANK + P::DIRECTION];
        let double_pushes = P::advance_bitboard(&(pushes & double_mask)) & free_spaces;

        let double_pushes_iter = double_pushes.squares().map(move |target| {
            let source = target.shift_rank(-P::DIRECTION * 2);
            Move::new(piece, source, target)
        });

        let opponent_pieces = self.occupancy_player(P::PLAYER.opponent());

        let captures_east = bitboard.shift_file_neg(1) & opponent_pieces;

        let captures_east_iter = captures_east.squares().map(move |target| {
            let source = target.shift_rank(-P::DIRECTION).shift_file(1);
            Move::new(piece, source, target)
        });

        let captures_west = bitboard.shift_file(1) & opponent_pieces;

        let captures_west_iter = captures_west.squares().map(move |target| {
            let source = target.shift_rank(-P::DIRECTION).shift_file(-1);
            Move::new(piece, source, target)
        });

        pushes_iter
            .chain(double_pushes_iter)
            .chain(captures_east_iter)
            .chain(captures_west_iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Square;

    const __: Option<Piece> = None;
    const WK: Option<Piece> = Some(Piece::WK);
    const WQ: Option<Piece> = Some(Piece::WQ);
    const WR: Option<Piece> = Some(Piece::WR);
    const WB: Option<Piece> = Some(Piece::WB);
    const WN: Option<Piece> = Some(Piece::WN);
    const WP: Option<Piece> = Some(Piece::WP);
    const BK: Option<Piece> = Some(Piece::BK);
    const BQ: Option<Piece> = Some(Piece::BQ);
    const BR: Option<Piece> = Some(Piece::BR);
    const BB: Option<Piece> = Some(Piece::BB);
    const BN: Option<Piece> = Some(Piece::BN);
    const BP: Option<Piece> = Some(Piece::BP);

    macro_rules! assert_moves {
        ($board:expr, [$($moves:expr),* $(,)*]) => {
            let mut moves: Vec<Move> = $board.moves().collect();
            moves.sort();

            let mut expected_moves: Vec<Move> = [
                $($moves),*
            ].iter().cloned().collect();
            expected_moves.sort();

            assert_eq!(moves, expected_moves);
        };
    }

    #[test]
    fn can_generate_all_possible_starting_moves_for_white() {
        let board = Board::default();

        // TODO: fill with all moves
        assert_moves!(
            board,
            [
                Move::new(Piece::WP, Square::A2, Square::A3),
                Move::new(Piece::WP, Square::B2, Square::B3),
                Move::new(Piece::WP, Square::C2, Square::C3),
                Move::new(Piece::WP, Square::D2, Square::D3),
                Move::new(Piece::WP, Square::E2, Square::E3),
                Move::new(Piece::WP, Square::F2, Square::F3),
                Move::new(Piece::WP, Square::G2, Square::G3),
                Move::new(Piece::WP, Square::H2, Square::H3),
                Move::new(Piece::WP, Square::A2, Square::A4),
                Move::new(Piece::WP, Square::B2, Square::B4),
                Move::new(Piece::WP, Square::C2, Square::C4),
                Move::new(Piece::WP, Square::D2, Square::D4),
                Move::new(Piece::WP, Square::E2, Square::E4),
                Move::new(Piece::WP, Square::F2, Square::F4),
                Move::new(Piece::WP, Square::G2, Square::G4),
                Move::new(Piece::WP, Square::H2, Square::H4)
            ]
        );
    }

    #[test]
    fn can_generate_all_possible_starting_moves_for_black() {
        let mut board = Board::default();
        board.make_move(Move::new(Piece::WP, Square::A2, Square::A3));

        // TODO: fill with all moves
        assert_moves!(
            board,
            [
                Move::new(Piece::BP, Square::A7, Square::A6),
                Move::new(Piece::BP, Square::B7, Square::B6),
                Move::new(Piece::BP, Square::C7, Square::C6),
                Move::new(Piece::BP, Square::D7, Square::D6),
                Move::new(Piece::BP, Square::E7, Square::E6),
                Move::new(Piece::BP, Square::F7, Square::F6),
                Move::new(Piece::BP, Square::G7, Square::G6),
                Move::new(Piece::BP, Square::H7, Square::H6),
                Move::new(Piece::BP, Square::A7, Square::A5),
                Move::new(Piece::BP, Square::B7, Square::B5),
                Move::new(Piece::BP, Square::C7, Square::C5),
                Move::new(Piece::BP, Square::D7, Square::D5),
                Move::new(Piece::BP, Square::E7, Square::E5),
                Move::new(Piece::BP, Square::F7, Square::F5),
                Move::new(Piece::BP, Square::G7, Square::G5),
                Move::new(Piece::BP, Square::H7, Square::H5)
            ]
        );
    }

    #[test]
    fn pawn_cannot_move_at_end_of_board() {
        // Such a situation is impossible in normal chess, but it's an edge case that could cause
        // something to go out of bounds.

        let board = Board::new(
            [
                [BP, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::Black,
        );

        assert_moves!(board, []);
    }

    #[test]
    fn pawn_cannot_capture_piece_directly_in_front_of_it() {
        let board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, WQ, __, __, __, __],
                [__, __, __, BP, __, WN, __, __],
                [__, __, __, __, __, WN, __, __],
                [__, __, __, __, __, BP, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::Black,
        );

        assert_moves!(board, []);
    }

    #[test]
    fn pawn_can_capture_pieces_on_diagonal() {
        let board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, WN, WP, WN, __, __, __],
                [__, __, __, BP, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::Black,
        );

        assert_moves!(
            board,
            [
                Move::new(Piece::BP, Square::D5, Square::C4),
                Move::new(Piece::BP, Square::D5, Square::E4)
            ]
        );
    }

    #[test]
    fn pawn_cannot_capture_same_player_pieces() {
        let board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, WP, BN, __, __, __],
                [__, __, __, BP, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::Black,
        );

        assert_moves!(board, []);
    }

    #[test]
    fn pawn_cannot_double_push_if_blocked() {
        let board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [WP, __, __, __, __, __, __, __],
                [BP, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_moves!(board, []);
    }

    #[test]
    fn pawn_cannot_double_push_when_not_at_initial_position() {
        let board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [WP, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_moves!(board, [Move::new(Piece::WP, Square::A3, Square::A4)]);
    }

    #[ignore]
    #[test]
    fn pawn_can_take_another_pawn_en_passant_immediately_after_double_push() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [WP, __, __, __, __, __, __, __],
                [__, WN, __, __, __, __, __, __],
                [__, BP, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        board.make_move(Move::new(Piece::WP, Square::A2, Square::A4));

        assert_moves!(board, [Move::new(Piece::BP, Square::B4, Square::A3)]);
    }

    #[ignore]
    #[test]
    fn pawn_cannot_take_another_pawn_en_passant_in_other_situations() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [WP, WN, __, __, __, __, __, __],
                [__, BP, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        board.make_move(Move::new(Piece::WP, Square::A3, Square::A4));

        assert_moves!(board, []);
    }
}