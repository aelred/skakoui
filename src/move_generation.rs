use crate::bitboards;
use crate::Bitboard;
use crate::BlackPlayer;
use crate::Board;
use crate::Move;
use crate::Piece;
use crate::PieceType;
use crate::Player;
use crate::PlayerType;
use crate::Square;
use crate::WhitePlayer;

impl Board {
    pub fn moves(&self) -> impl Iterator<Item = Move> {
        match self.player() {
            Player::White => self.moves_for_player::<WhitePlayer>(),
            Player::Black => self.moves_for_player::<BlackPlayer>(),
        }
    }

    fn moves_for_player<P: PlayerType + 'static>(&self) -> Box<dyn Iterator<Item = Move>> {
        let iter = self
            .pawn_moves::<P>()
            .chain(self.king_moves::<P>())
            .chain(self.knight_moves::<P>())
            .chain(self.rook_moves::<P>());

        Box::new(iter)
    }

    fn pawn_moves<P: PlayerType>(&self) -> impl Iterator<Item = Move> {
        let piece = Piece::new(P::PLAYER, PieceType::Pawn);

        let pawns = self.bitboard_piece(piece);
        let free_spaces = !self.occupancy();

        let pawns_forward = P::advance_bitboard(pawns);

        let pushes = pawns_forward & free_spaces;

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

        let captures_east = pawns_forward.shift_file_neg(1) & opponent_pieces;

        let captures_east_iter = captures_east.squares().map(move |target| {
            let source = target.shift_rank(-P::DIRECTION).shift_file(1);
            Move::new(piece, source, target)
        });

        let captures_west = pawns_forward.shift_file(1) & opponent_pieces;

        let captures_west_iter = captures_west.squares().map(move |target| {
            let source = target.shift_rank(-P::DIRECTION).shift_file(-1);
            Move::new(piece, source, target)
        });

        pushes_iter
            .chain(double_pushes_iter)
            .chain(captures_east_iter)
            .chain(captures_west_iter)
    }

    fn king_moves<P: PlayerType>(&self) -> impl Iterator<Item = Move> {
        let piece = Piece::new(P::PLAYER, PieceType::King);
        let kings = *self.bitboard_piece(piece);
        let occupancy_player = self.occupancy_player(P::PLAYER);

        // Iterate over all kings on the board, although there is only one in a valid game.
        // This means we can handle games with zero or many kings - good for test cases.
        kings.squares().flat_map(move |source| {
            let mut attacks = kings.shift_rank(1) | kings.shift_rank_neg(1);
            let ranks = kings | attacks;

            attacks |= ranks.shift_file(1) | ranks.shift_file_neg(1);

            attacks &= !occupancy_player;

            attacks
                .squares()
                .map(move |target| Move::new(piece, source, target))
        })
    }

    fn knight_moves<P: PlayerType>(&self) -> impl Iterator<Item = Move> {
        let piece = Piece::new(P::PLAYER, PieceType::Knight);
        let knights = *self.bitboard_piece(piece);
        let occupancy_player = self.occupancy_player(P::PLAYER);

        knights.squares().flat_map(move |source| {
            let attacks = Bitboard::from(source).knight_moves() & !occupancy_player;

            attacks
                .squares()
                .map(move |target| Move::new(piece, source, target))
        })
    }

    fn rook_moves<P: PlayerType>(&self) -> impl Iterator<Item = Move> {
        let piece = Piece::new(P::PLAYER, PieceType::Rook);
        let rooks = *self.bitboard_piece(piece);

        let occupancy = self.occupancy();
        let occupancy_player = self.occupancy_player(P::PLAYER);

        rooks.squares().flat_map(move |source| {
            let north_movement = source.north_bitboard();
            let mut north_blockers = north_movement & occupancy;
            // Set MSB so there is always a blocking square (no need to branch)
            north_blockers.set(Square::H8);
            let blocking_square = north_blockers.squares().next().unwrap();
            let north = north_movement ^ blocking_square.north_bitboard();

            let south_movement = source.south_bitboard();
            let mut south_blockers = south_movement & occupancy;
            // Set LSB so there is always a blocking square (no need to branch)
            south_blockers.set(Square::A1);
            let blocking_square = south_blockers.squares().rev().next().unwrap();
            let south = south_movement ^ blocking_square.south_bitboard();

            let east_movement = source.east_bitboard();
            let mut east_blockers = east_movement & occupancy;
            // Set MSB so there is always a blocking square (no need to branch)
            east_blockers.set(Square::H8);
            let blocking_square = east_blockers.squares().next().unwrap();
            let east = east_movement ^ blocking_square.east_bitboard();

            let west_movement = source.west_bitboard();
            let mut west_blockers = west_movement & occupancy;
            // Set MSB so there is always a blocking square (no need to branch)
            west_blockers.set(Square::A1);
            let blocking_square = west_blockers.squares().rev().next().unwrap();
            let west = west_movement ^ blocking_square.west_bitboard();

            let attacks = (north | south | east | west) & !occupancy_player;

            attacks
                .squares()
                .map(move |target| Move::new(piece, source, target))
        })
    }
}

impl Square {
    fn north_bitboard(self) -> Bitboard {
        bitboards::FILES[self.file()] & !bitboards::RANKS_FILLED[self.rank().to_index() + 1]
    }

    fn south_bitboard(self) -> Bitboard {
        bitboards::FILES[self.file()] & bitboards::RANKS_FILLED[self.rank().to_index()]
    }

    fn east_bitboard(self) -> Bitboard {
        bitboards::RANKS[self.rank()] & !bitboards::FILES_FILLED[self.file().to_index() + 1]
    }

    fn west_bitboard(self) -> Bitboard {
        bitboards::RANKS[self.rank()] & bitboards::FILES_FILLED[self.file().to_index()]
    }
}

impl Bitboard {
    fn knight_moves(self) -> Bitboard {
        let ranks = self.shift_rank(2) | self.shift_rank_neg(2);
        let rank_attacks = ranks.shift_file(1) | ranks.shift_file_neg(1);

        let files = self.shift_file(2) | self.shift_file_neg(2);
        let file_attacks = files.shift_rank(1) | files.shift_rank_neg(1);

        rank_attacks | file_attacks
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

            assert_eq!(moves, expected_moves, "\n{}", $board);
        };
    }

    #[test]
    fn can_generate_all_possible_starting_moves_for_white() {
        let board = Board::default();

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
                Move::new(Piece::WP, Square::H2, Square::H4),
                Move::new(Piece::WN, Square::B1, Square::A3),
                Move::new(Piece::WN, Square::B1, Square::C3),
                Move::new(Piece::WN, Square::G1, Square::H3),
                Move::new(Piece::WN, Square::G1, Square::F3)
            ]
        );
    }

    #[test]
    fn can_generate_all_possible_starting_moves_for_black() {
        let mut board = Board::default();
        board.make_move(Move::new(Piece::WP, Square::A2, Square::A3));

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
                Move::new(Piece::BP, Square::H7, Square::H5),
                Move::new(Piece::BN, Square::B8, Square::A6),
                Move::new(Piece::BN, Square::B8, Square::C6),
                Move::new(Piece::BN, Square::G8, Square::H6),
                Move::new(Piece::BN, Square::G8, Square::F6)
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
                [__, __, __, __, WP, __, __, __],
                [__, __, __, WP, BP, __, __, __],
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

    #[test]
    fn king_can_move_and_capture_one_square_in_any_direction() {
        let board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, WP, __, __, __, __, __],
                [__, WK, BP, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_moves!(
            board,
            [
                Move::new(Piece::WK, Square::B3, Square::A2),
                Move::new(Piece::WK, Square::B3, Square::B2),
                Move::new(Piece::WK, Square::B3, Square::A3),
                Move::new(Piece::WK, Square::B3, Square::C3),
                Move::new(Piece::WK, Square::B3, Square::A4),
                Move::new(Piece::WK, Square::B3, Square::B4),
                Move::new(Piece::WK, Square::B3, Square::C4),
            ]
        );
    }

    #[test]
    fn knight_can_move_and_capture_in_its_weird_way() {
        let board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, WN, __, __, __, __, __, __],
                [__, __, __, BP, __, __, __, __],
                [__, __, WP, __, __, __, __, __],
                [__, __, BP, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_moves!(
            board,
            [
                Move::new(Piece::WN, Square::B3, Square::A1),
                Move::new(Piece::WN, Square::B3, Square::C1),
                Move::new(Piece::WN, Square::B3, Square::D2),
                Move::new(Piece::WN, Square::B3, Square::D4),
                Move::new(Piece::WN, Square::B3, Square::A5),
            ]
        );
    }

    #[test]
    fn rook_can_move_and_capture_along_rank_and_file() {
        let board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, WR, BQ, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, WP, __, __, __, __, __, __],
                [__, BP, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_moves!(
            board,
            [
                Move::new(Piece::WR, Square::B3, Square::B1),
                Move::new(Piece::WR, Square::B3, Square::B2),
                Move::new(Piece::WR, Square::B3, Square::A3),
                Move::new(Piece::WR, Square::B3, Square::C3),
                Move::new(Piece::WR, Square::B3, Square::B4),
            ]
        );
    }
}
