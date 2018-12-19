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
    pub fn moves(&mut self) -> Box<dyn Iterator<Item = Move>> {
        match self.player() {
            Player::White => self.moves_for_player::<WhitePlayer>(),
            Player::Black => self.moves_for_player::<BlackPlayer>(),
        }
    }

    fn moves_for_player<P: PlayerType + 'static>(&mut self) -> Box<dyn Iterator<Item = Move>> {
        let pseudo_legal_moves = self.pseudo_legal_moves::<P>();

        let mut this = self.clone();

        // TODO: this is a very inefficient way to confirm if in check
        let legal_moves = pseudo_legal_moves.filter(move |mov| {
            this.make_move(*mov);
            let in_check = this.can_take_king::<P::Opp>();
            this.unmake_move(*mov);
            !in_check
        });

        Box::new(legal_moves)
    }

    fn can_take_king<P: PlayerType + 'static>(&self) -> bool {
        let king = Piece::new(P::Opp::PLAYER, PieceType::King);

        if let Some(king_pos) = self.bitboard_piece(king).squares().next() {
            for mov in self.pseudo_legal_moves::<P>() {
                if mov.to() != king_pos {
                    continue;
                }

                let mut after_move = self.clone();
                after_move.make_move(mov);
                if after_move.count(king) == 0 {
                    return true;
                }
            }

            false
        } else {
            false
        }
    }

    fn pseudo_legal_moves<P: PlayerType + 'static>(&self) -> impl Iterator<Item = Move> {
        self.king_moves()
            .chain(self.queen_moves())
            .chain(self.rook_moves())
            .chain(self.bishop_moves())
            .chain(self.knight_moves())
            .chain(self.pawn_moves::<P>())
    }

    fn pawn_moves<P: PlayerType>(&self) -> impl Iterator<Item = Move> {
        let piece = Piece::new(P::PLAYER, PieceType::Pawn);

        let pawns = self.bitboard_piece(piece);
        let free_spaces = !self.occupancy();

        let pawns_forward = P::advance_bitboard(pawns);

        let pushes = pawns_forward & free_spaces;

        let pushes_iter = pushes.squares().map(move |target| {
            let source = target.shift_rank(-P::DIRECTION);
            Move::new(PieceType::Pawn, source, target)
        });

        let double_mask = bitboards::RANKS[P::PAWN_RANK + P::DIRECTION];
        let double_pushes = P::advance_bitboard(&(pushes & double_mask)) & free_spaces;

        let double_pushes_iter = double_pushes.squares().map(move |target| {
            let source = target.shift_rank(-P::DIRECTION * 2);
            Move::new(PieceType::Pawn, source, target)
        });

        let opponent_pieces = self.occupancy_player(P::PLAYER.opponent());

        let captures_east = pawns_forward.shift_file_neg(1) & opponent_pieces;

        let captures_east_iter = captures_east.squares().map(move |target| {
            let source = target.shift_rank(-P::DIRECTION).shift_file(1);
            Move::new(PieceType::Pawn, source, target)
        });

        let captures_west = pawns_forward.shift_file(1) & opponent_pieces;

        let captures_west_iter = captures_west.squares().map(move |target| {
            let source = target.shift_rank(-P::DIRECTION).shift_file(-1);
            Move::new(PieceType::Pawn, source, target)
        });

        captures_west_iter
            .chain(captures_east_iter)
            .chain(double_pushes_iter)
            .chain(pushes_iter)
    }

    fn king_moves(&self) -> impl Iterator<Item = Move> {
        self.moves_for_piece(PieceType::King, move |source| {
            let king = Bitboard::from(source);
            let attacks = king.shift_rank(1) | king.shift_rank_neg(1);
            let ranks = king | attacks;
            attacks | ranks.shift_file(1) | ranks.shift_file_neg(1)
        })
    }

    fn knight_moves(&self) -> impl Iterator<Item = Move> {
        self.moves_for_piece(PieceType::Knight, move |source| {
            Bitboard::from(source).knight_moves()
        })
    }

    fn rook_moves(&self) -> impl Iterator<Item = Move> {
        let occupancy = self.occupancy();

        self.moves_for_piece(PieceType::Rook, move |source| {
            slide::<NorthSouth>(source, occupancy) | slide::<EastWest>(source, occupancy)
        })
    }

    fn bishop_moves(&self) -> impl Iterator<Item = Move> {
        let occupancy = self.occupancy();

        self.moves_for_piece(PieceType::Bishop, move |source| {
            slide::<Diagonal>(source, occupancy) | slide::<AntiDiagonal>(source, occupancy)
        })
    }

    fn queen_moves(&self) -> impl Iterator<Item = Move> {
        let occupancy = self.occupancy();

        self.moves_for_piece(PieceType::Queen, move |source| {
            slide::<NorthSouth>(source, occupancy)
                | slide::<EastWest>(source, occupancy)
                | slide::<Diagonal>(source, occupancy)
                | slide::<AntiDiagonal>(source, occupancy)
        })
    }

    fn moves_for_piece<F>(&self, piece_type: PieceType, attacks: F) -> impl Iterator<Item = Move>
    where
        F: Fn(Square) -> Bitboard,
    {
        let piece = Piece::new(self.player(), piece_type);
        let positions = *self.bitboard_piece(piece);

        let occupancy_player = self.occupancy_player(self.player());

        positions.squares().flat_map(move |source| {
            let attacks = attacks(source) & !occupancy_player;

            attacks
                .squares()
                .map(move |target| Move::new(piece_type, source, target))
        })
    }
}

fn slide<Dir: SlideDirection>(source: Square, occupancy: Bitboard) -> Bitboard {
    let pos_movement = Dir::positive_bitboard(source);
    let mut blockers = pos_movement & occupancy;
    // Set the last square so there is always a blocking square (no need to branch)
    blockers.set(Square::H8);
    let blocking_square = blockers.squares().next().unwrap();
    let pos_movement = pos_movement ^ Dir::positive_bitboard(blocking_square);

    let neg_movement = Dir::negative_bitboard(source);
    let mut blockers = neg_movement & occupancy;
    // Set the last square so there is always a blocking square (no need to branch)
    blockers.set(Square::A1);
    let blocking_square = blockers.squares().rev().next().unwrap();
    let neg_movement = neg_movement ^ Dir::negative_bitboard(blocking_square);

    pos_movement | neg_movement
}

trait SlideDirection {
    fn positive_bitboard(source: Square) -> Bitboard;
    fn negative_bitboard(source: Square) -> Bitboard;
}

struct NorthSouth;
impl SlideDirection for NorthSouth {
    fn positive_bitboard(source: Square) -> Bitboard {
        bitboards::FILES[source.file()] & !bitboards::RANKS_FILLED[source.rank().to_index() + 1]
    }

    fn negative_bitboard(source: Square) -> Bitboard {
        bitboards::FILES[source.file()] & bitboards::RANKS_FILLED[source.rank().to_index()]
    }
}

struct EastWest;
impl SlideDirection for EastWest {
    fn positive_bitboard(source: Square) -> Bitboard {
        bitboards::RANKS[source.rank()] & !bitboards::FILES_FILLED[source.file().to_index() + 1]
    }

    fn negative_bitboard(source: Square) -> Bitboard {
        bitboards::RANKS[source.rank()] & bitboards::FILES_FILLED[source.file().to_index()]
    }
}

struct Diagonal;
impl SlideDirection for Diagonal {
    fn positive_bitboard(source: Square) -> Bitboard {
        bitboards::DIAGONALS[source.file()][source.rank()]
            & !bitboards::FILES_FILLED[source.file().to_index() + 1]
    }

    fn negative_bitboard(source: Square) -> Bitboard {
        bitboards::DIAGONALS[source.file()][source.rank()]
            & bitboards::FILES_FILLED[source.file().to_index()]
    }
}

struct AntiDiagonal;
impl SlideDirection for AntiDiagonal {
    fn positive_bitboard(source: Square) -> Bitboard {
        bitboards::ANTIDIAGONALS[source.file()][source.rank()]
            & !bitboards::RANKS_FILLED[source.rank().to_index() + 1]
    }

    fn negative_bitboard(source: Square) -> Bitboard {
        bitboards::ANTIDIAGONALS[source.file()][source.rank()]
            & bitboards::RANKS_FILLED[source.rank().to_index()]
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
    const BQ: Option<Piece> = Some(Piece::BQ);
    const BR: Option<Piece> = Some(Piece::BR);
    const BB: Option<Piece> = Some(Piece::BB);
    const BP: Option<Piece> = Some(Piece::BP);

    macro_rules! mov {
        ($mov:expr) => {
            stringify!($mov).parse::<Move>().unwrap()
        };
    }

    macro_rules! assert_moves {
        ($board:expr, [$($moves:expr),* $(,)*]) => {
            let mut moves: Vec<Move> = $board.moves().collect();
            moves.sort();

            let mut expected_moves: Vec<Move> = [
                $(mov!($moves)),*
            ].iter().cloned().collect();
            expected_moves.sort();

            assert_eq!(moves, expected_moves, "\n{}", $board);
        };
    }

    #[test]
    fn can_generate_all_possible_starting_moves_for_white() {
        let mut board = Board::default();

        assert_moves!(
            board,
            [
                Pa2a3, Pb2b3, Pc2c3, Pd2d3, Pe2e3, Pf2f3, Pg2g3, Ph2h3, Pa2a4, Pb2b4, Pc2c4, Pd2d4,
                Pe2e4, Pf2f4, Pg2g4, Ph2h4, Nb1a3, Nb1c3, Ng1h3, Ng1f3
            ]
        );
    }

    #[test]
    fn can_generate_all_possible_starting_moves_for_black() {
        let mut board = Board::default();
        board.make_move(mov!(Pa2a3));

        assert_moves!(
            board,
            [
                Pa7a6, Pb7b6, Pc7c6, Pd7d6, Pe7e6, Pf7f6, Pg7g6, Ph7h6, Pa7a5, Pb7b5, Pc7c5, Pd7d5,
                Pe7e5, Pf7f5, Pg7g5, Ph7h5, Nb8a6, Nb8c6, Ng8h6, Ng8f6
            ]
        );
    }

    #[test]
    fn pawn_cannot_move_at_end_of_board() {
        // Such a situation is impossible in normal chess, but it's an edge case that could cause
        // something to go out of bounds.

        let mut board = Board::new(
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
        let mut board = Board::new(
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
        let mut board = Board::new(
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

        assert_moves!(board, [Pd5c4, Pd5e4]);
    }

    #[test]
    fn pawn_cannot_capture_same_player_pieces() {
        let mut board = Board::new(
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
        let mut board = Board::new(
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
        let mut board = Board::new(
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

        assert_moves!(board, [Pa3a4]);
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

        board.make_move(mov!(Pa2a4));

        assert_moves!(board, [Pb4a3]);
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

        board.make_move(mov!(Pa3a4));

        assert_moves!(board, []);
    }

    #[test]
    fn king_can_move_and_capture_one_square_in_any_direction() {
        let mut board = Board::new(
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

        // Kb3b2 is missing because it puts the king in check
        assert_moves!(board, [Kb3a2, Kb3a3, Kb3c3, Kb3a4, Kb3b4, Kb3c4,]);
    }

    #[test]
    fn knight_can_move_and_capture_in_its_weird_way() {
        let mut board = Board::new(
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

        assert_moves!(board, [Nb3a1, Nb3c1, Nb3d2, Nb3d4, Nb3a5,]);
    }

    #[test]
    fn rook_can_move_and_capture_along_rank_and_file() {
        let mut board = Board::new(
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

        assert_moves!(board, [Rb3b1, Rb3b2, Rb3a3, Rb3c3, Rb3b4,]);
    }

    #[test]
    fn bishop_can_move_and_capture_diagonally() {
        let mut board = Board::new(
            [
                [__, __, __, BB, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, WB, __, __, __, __, __, __],
                [__, __, WP, __, __, __, __, __],
                [__, __, BP, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_moves!(board, [Bb3d1, Bb3a2, Bb3c2, Bb3a4,]);
    }

    #[test]
    fn queen_can_move_and_capture_in_all_directions() {
        let mut board = Board::new(
            [
                [__, __, __, BB, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, WQ, WP, __, __, __, __, __],
                [__, __, WP, __, __, __, __, __],
                [__, __, BP, __, __, __, __, __],
                [__, BP, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_moves!(
            board,
            [Qb3d1, Qb3a2, Qb3c2, Qb3a4, Qb3a3, Qb3b1, Qb3b2, Qb3b4, Qb3b5, Qb3b6,]
        );
    }

    #[test]
    fn cannot_make_a_move_that_leaves_king_in_check() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [WK, WP, __, __, __, __, __, BR],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        // Note that the pawn is not allowed to move
        assert_moves!(board, [Ka2a1, Ka2b1, Ka2a3, Ka2b3,]);
    }
}
