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
        let pseudo_legal_moves = self.pseudo_legal_moves::<P>();

        let cloned_board = self.clone();

        // TODO: this is a very inefficient way to confirm if in check
        let legal_moves = pseudo_legal_moves.filter(move |mov| {
            let mut after_move = cloned_board.clone();
            after_move.make_move(*mov);
            !after_move.can_take_king::<P::Opp>()
        });

        Box::new(legal_moves)
    }

    fn can_take_king<P: PlayerType + 'static>(&self) -> bool {
        let king = Piece::new(P::Opp::PLAYER, PieceType::King);

        if self.count(king) == 0 {
            return false;
        }

        for mov in self.pseudo_legal_moves::<P>() {
            let mut after_move = self.clone();
            after_move.make_move(mov);
            if after_move.count(king) == 0 {
                return true;
            }
        }

        false
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
            let north = slide::<North>(source, occupancy);
            let south = slide::<South>(source, occupancy);
            let east = slide::<East>(source, occupancy);
            let west = slide::<West>(source, occupancy);
            (north | south | east | west)
        })
    }

    fn bishop_moves(&self) -> impl Iterator<Item = Move> {
        let occupancy = self.occupancy();

        self.moves_for_piece(PieceType::Bishop, move |source| {
            let nw = slide::<NorthWest>(source, occupancy);
            let se = slide::<SouthEast>(source, occupancy);
            let ne = slide::<NorthEast>(source, occupancy);
            let sw = slide::<SouthWest>(source, occupancy);
            (nw | se | ne | sw)
        })
    }

    fn queen_moves(&self) -> impl Iterator<Item = Move> {
        let occupancy = self.occupancy();

        self.moves_for_piece(PieceType::Queen, move |source| {
            let north = slide::<North>(source, occupancy);
            let south = slide::<South>(source, occupancy);
            let east = slide::<East>(source, occupancy);
            let west = slide::<West>(source, occupancy);
            let nw = slide::<NorthWest>(source, occupancy);
            let se = slide::<SouthEast>(source, occupancy);
            let ne = slide::<NorthEast>(source, occupancy);
            let sw = slide::<SouthWest>(source, occupancy);
            (north | south | east | west | nw | se | ne | sw)
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
    let movement = Dir::bitboard(source);
    let mut blockers = movement & occupancy;
    // Set the last square so there is always a blocking square (no need to branch)
    blockers.set(Dir::Type::LAST_SQUARE);
    let blocking_square = Dir::Type::first_square(&mut blockers.squares()).unwrap();
    movement ^ Dir::bitboard(blocking_square)
}

trait SlideDirection {
    type Type: SlideDirectionType;
    fn bitboard(source: Square) -> Bitboard;
}

struct North;
impl SlideDirection for North {
    type Type = Positive;

    fn bitboard(source: Square) -> Bitboard {
        bitboards::FILES[source.file()] & !bitboards::RANKS_FILLED[source.rank().to_index() + 1]
    }
}

struct South;
impl SlideDirection for South {
    type Type = Negative;

    fn bitboard(source: Square) -> Bitboard {
        bitboards::FILES[source.file()] & bitboards::RANKS_FILLED[source.rank().to_index()]
    }
}

struct East;
impl SlideDirection for East {
    type Type = Positive;

    fn bitboard(source: Square) -> Bitboard {
        bitboards::RANKS[source.rank()] & !bitboards::FILES_FILLED[source.file().to_index() + 1]
    }
}

struct West;
impl SlideDirection for West {
    type Type = Negative;

    fn bitboard(source: Square) -> Bitboard {
        bitboards::RANKS[source.rank()] & bitboards::FILES_FILLED[source.file().to_index()]
    }
}

struct NorthWest;
impl SlideDirection for NorthWest {
    type Type = Positive;

    fn bitboard(source: Square) -> Bitboard {
        bitboards::ANTIDIAGONALS[source.file()][source.rank()]
            & !bitboards::RANKS_FILLED[source.rank().to_index() + 1]
    }
}

struct SouthEast;
impl SlideDirection for SouthEast {
    type Type = Negative;

    fn bitboard(source: Square) -> Bitboard {
        bitboards::ANTIDIAGONALS[source.file()][source.rank()]
            & bitboards::RANKS_FILLED[source.rank().to_index()]
    }
}

struct NorthEast;
impl SlideDirection for NorthEast {
    type Type = Positive;

    fn bitboard(source: Square) -> Bitboard {
        bitboards::DIAGONALS[source.file()][source.rank()]
            & !bitboards::FILES_FILLED[source.file().to_index() + 1]
    }
}

struct SouthWest;
impl SlideDirection for SouthWest {
    type Type = Negative;

    fn bitboard(source: Square) -> Bitboard {
        bitboards::DIAGONALS[source.file()][source.rank()]
            & bitboards::FILES_FILLED[source.file().to_index()]
    }
}

trait SlideDirectionType {
    const LAST_SQUARE: Square;
    fn first_square(squares: &mut impl DoubleEndedIterator<Item = Square>) -> Option<Square>;
}

struct Positive;
impl SlideDirectionType for Positive {
    const LAST_SQUARE: Square = Square::H8;

    fn first_square(squares: &mut impl DoubleEndedIterator<Item = Square>) -> Option<Square> {
        squares.next()
    }
}

struct Negative;
impl SlideDirectionType for Negative {
    const LAST_SQUARE: Square = Square::A1;

    fn first_square(squares: &mut impl DoubleEndedIterator<Item = Square>) -> Option<Square> {
        squares.rev().next()
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
                Move::new(PieceType::Pawn, Square::A2, Square::A3),
                Move::new(PieceType::Pawn, Square::B2, Square::B3),
                Move::new(PieceType::Pawn, Square::C2, Square::C3),
                Move::new(PieceType::Pawn, Square::D2, Square::D3),
                Move::new(PieceType::Pawn, Square::E2, Square::E3),
                Move::new(PieceType::Pawn, Square::F2, Square::F3),
                Move::new(PieceType::Pawn, Square::G2, Square::G3),
                Move::new(PieceType::Pawn, Square::H2, Square::H3),
                Move::new(PieceType::Pawn, Square::A2, Square::A4),
                Move::new(PieceType::Pawn, Square::B2, Square::B4),
                Move::new(PieceType::Pawn, Square::C2, Square::C4),
                Move::new(PieceType::Pawn, Square::D2, Square::D4),
                Move::new(PieceType::Pawn, Square::E2, Square::E4),
                Move::new(PieceType::Pawn, Square::F2, Square::F4),
                Move::new(PieceType::Pawn, Square::G2, Square::G4),
                Move::new(PieceType::Pawn, Square::H2, Square::H4),
                Move::new(PieceType::Knight, Square::B1, Square::A3),
                Move::new(PieceType::Knight, Square::B1, Square::C3),
                Move::new(PieceType::Knight, Square::G1, Square::H3),
                Move::new(PieceType::Knight, Square::G1, Square::F3)
            ]
        );
    }

    #[test]
    fn can_generate_all_possible_starting_moves_for_black() {
        let mut board = Board::default();
        board.make_move(Move::new(PieceType::Pawn, Square::A2, Square::A3));

        assert_moves!(
            board,
            [
                Move::new(PieceType::Pawn, Square::A7, Square::A6),
                Move::new(PieceType::Pawn, Square::B7, Square::B6),
                Move::new(PieceType::Pawn, Square::C7, Square::C6),
                Move::new(PieceType::Pawn, Square::D7, Square::D6),
                Move::new(PieceType::Pawn, Square::E7, Square::E6),
                Move::new(PieceType::Pawn, Square::F7, Square::F6),
                Move::new(PieceType::Pawn, Square::G7, Square::G6),
                Move::new(PieceType::Pawn, Square::H7, Square::H6),
                Move::new(PieceType::Pawn, Square::A7, Square::A5),
                Move::new(PieceType::Pawn, Square::B7, Square::B5),
                Move::new(PieceType::Pawn, Square::C7, Square::C5),
                Move::new(PieceType::Pawn, Square::D7, Square::D5),
                Move::new(PieceType::Pawn, Square::E7, Square::E5),
                Move::new(PieceType::Pawn, Square::F7, Square::F5),
                Move::new(PieceType::Pawn, Square::G7, Square::G5),
                Move::new(PieceType::Pawn, Square::H7, Square::H5),
                Move::new(PieceType::Knight, Square::B8, Square::A6),
                Move::new(PieceType::Knight, Square::B8, Square::C6),
                Move::new(PieceType::Knight, Square::G8, Square::H6),
                Move::new(PieceType::Knight, Square::G8, Square::F6)
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
                Move::new(PieceType::Pawn, Square::D5, Square::C4),
                Move::new(PieceType::Pawn, Square::D5, Square::E4)
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

        assert_moves!(board, [Move::new(PieceType::Pawn, Square::A3, Square::A4)]);
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

        board.make_move(Move::new(PieceType::Pawn, Square::A2, Square::A4));

        assert_moves!(board, [Move::new(PieceType::Pawn, Square::B4, Square::A3)]);
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

        board.make_move(Move::new(PieceType::Pawn, Square::A3, Square::A4));

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

        // Kb3b2 is missing because it puts the king in check
        assert_moves!(
            board,
            [
                Move::new(PieceType::King, Square::B3, Square::A2),
                Move::new(PieceType::King, Square::B3, Square::A3),
                Move::new(PieceType::King, Square::B3, Square::C3),
                Move::new(PieceType::King, Square::B3, Square::A4),
                Move::new(PieceType::King, Square::B3, Square::B4),
                Move::new(PieceType::King, Square::B3, Square::C4),
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
                Move::new(PieceType::Knight, Square::B3, Square::A1),
                Move::new(PieceType::Knight, Square::B3, Square::C1),
                Move::new(PieceType::Knight, Square::B3, Square::D2),
                Move::new(PieceType::Knight, Square::B3, Square::D4),
                Move::new(PieceType::Knight, Square::B3, Square::A5),
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
                Move::new(PieceType::Rook, Square::B3, Square::B1),
                Move::new(PieceType::Rook, Square::B3, Square::B2),
                Move::new(PieceType::Rook, Square::B3, Square::A3),
                Move::new(PieceType::Rook, Square::B3, Square::C3),
                Move::new(PieceType::Rook, Square::B3, Square::B4),
            ]
        );
    }

    #[test]
    fn bishop_can_move_and_capture_diagonally() {
        let board = Board::new(
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

        assert_moves!(
            board,
            [
                Move::new(PieceType::Bishop, Square::B3, Square::D1),
                Move::new(PieceType::Bishop, Square::B3, Square::A2),
                Move::new(PieceType::Bishop, Square::B3, Square::C2),
                Move::new(PieceType::Bishop, Square::B3, Square::A4),
            ]
        );
    }

    #[test]
    fn queen_can_move_and_capture_in_all_directions() {
        let board = Board::new(
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
            [
                Move::new(PieceType::Queen, Square::B3, Square::D1),
                Move::new(PieceType::Queen, Square::B3, Square::A2),
                Move::new(PieceType::Queen, Square::B3, Square::C2),
                Move::new(PieceType::Queen, Square::B3, Square::A4),
                Move::new(PieceType::Queen, Square::B3, Square::A3),
                Move::new(PieceType::Queen, Square::B3, Square::B1),
                Move::new(PieceType::Queen, Square::B3, Square::B2),
                Move::new(PieceType::Queen, Square::B3, Square::B4),
                Move::new(PieceType::Queen, Square::B3, Square::B5),
                Move::new(PieceType::Queen, Square::B3, Square::B6),
            ]
        );
    }

    #[test]
    fn cannot_make_a_move_that_leaves_king_in_check() {
        let board = Board::new(
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
        assert_moves!(
            board,
            [
                Move::new(PieceType::King, Square::A2, Square::A1),
                Move::new(PieceType::King, Square::A2, Square::B1),
                Move::new(PieceType::King, Square::A2, Square::A3),
                Move::new(PieceType::King, Square::A2, Square::B3),
            ]
        );
    }
}
