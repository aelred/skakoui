use crate::bitboards;
use crate::Bitboard;
use crate::BlackPlayer;
use crate::File;
use crate::Move;
use crate::Piece;
use crate::PieceType;
use crate::Player;
use crate::PlayerType;
use crate::Rank;
use crate::Square;
use crate::SquareColor;
use crate::WhitePlayer;
use enum_map::EnumMap;
use std::fmt;
use std::ops::BitOr;

#[derive(Debug, Eq, PartialEq)]
pub struct Board {
    bitboards: EnumMap<Player, EnumMap<PieceType, Bitboard>>,
    player: Player,
}

impl Board {
    fn get(&self, square: Square) -> Option<Piece> {
        for (player, bitboards) in self.bitboards.iter() {
            for (piece_type, bitboard) in bitboards {
                if bitboard.get(square) {
                    return Some(Piece::new(player, piece_type));
                }
            }
        }

        None
    }

    pub fn make_move(&mut self, mov: Move) {
        if let Some(captured_piece) = self.get(mov.to()) {
            self.bitboard_piece_mut(captured_piece).reset(mov.to());
        }

        let bitboard = self.bitboard_piece_mut(mov.piece());

        match mov.promoting() {
            Some(promotion) => {
                bitboard.reset(mov.from());
                self.bitboard_piece_mut(promotion).set(mov.to());
            }
            None => {
                bitboard.move_bit(mov.from(), mov.to());
            }
        }

        self.player = self.player.opponent();
    }

    pub fn bitboard_piece(&self, piece: Piece) -> &Bitboard {
        &self.bitboards[piece.player()][piece.piece_type()]
    }

    pub fn bitboard_piece_mut(&mut self, piece: Piece) -> &mut Bitboard {
        &mut self.bitboards[piece.player()][piece.piece_type()]
    }

    pub fn moves(&self) -> impl Iterator<Item = Move> {
        match self.player {
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

    fn occupancy(&self) -> Bitboard {
        self.bitboards
            .values()
            .flat_map(EnumMap::values)
            .fold(bitboards::EMPTY, BitOr::bitor)
    }

    fn occupancy_player(&self, player: Player) -> Bitboard {
        self.bitboards[player]
            .values()
            .fold(bitboards::EMPTY, BitOr::bitor)
    }
}

impl Default for Board {
    fn default() -> Self {
        Board {
            bitboards: *bitboards::START_POSITIONS,
            player: Player::White,
        }
    }
}

impl fmt::Display for Board {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        let files_str: String = File::VALUES.iter().map(File::to_string).collect();
        f.write_fmt(format_args!("  {}\n", files_str))?;

        for rank in Rank::VALUES.iter().rev() {
            f.write_fmt(format_args!("{} ", rank))?;
            for file in File::VALUES.iter() {
                let square = Square::new(*file, *rank);
                let s = match self.get(square) {
                    Some(piece) => piece.to_string(),
                    None => {
                        let col = match square.color() {
                            SquareColor::White => " ",
                            SquareColor::Black => "█",
                        };
                        col.to_string()
                    }
                };
                f.write_str(&s)?;
            }
            f.write_fmt(format_args!(" {}\n", rank))?;
        }

        f.write_fmt(format_args!("  {}", files_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    impl Board {
        fn new(pieces: [[Option<Piece>; 8]; 8], player: Player) -> Self {
            let mut bitboards = EnumMap::from(|_| EnumMap::from(|_| bitboards::EMPTY));

            for (i, pieces_rank) in pieces.iter().enumerate() {
                for (j, optional_piece) in pieces_rank.iter().enumerate() {
                    if let Some(piece) = *optional_piece {
                        let rank = Rank::from_index(i);
                        let file = File::from_index(j);
                        let square = Square::new(file, rank);
                        bitboards[piece.player()][piece.piece_type()].set(square);
                    }
                }
            }

            Board { bitboards, player }
        }
    }

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
    fn can_create_default_chess_board() {
        Board::default();
    }

    #[test]
    fn default_chess_board_has_pieces_in_position() {
        let board = Board::default();

        let expected_board = Board::new(
            [
                [WR, WN, WB, WQ, WK, WB, WN, WR],
                [WP, WP, WP, WP, WP, WP, WP, WP],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [BP, BP, BP, BP, BP, BP, BP, BP],
                [BR, BN, BB, BQ, BK, BB, BN, BR],
            ],
            Player::White,
        );

        assert_eq!(board, expected_board);

        // The above grid is fiddly to get right, so check some known piece positions are right
        assert_eq!(board.get(Square::A1), WR);
        assert_eq!(board.get(Square::A2), WP);
        assert_eq!(board.get(Square::E8), BK);
    }

    #[test]
    fn displaying_a_board_returns_a_unicode_grid() {
        let expected = r#"
  abcdefgh
8 ♜♞♝♛♚♝♞♜ 8
7 ♟♟♟♟♟♟♟♟ 7
6  █ █ █ █ 6
5 █ █ █ █  5
4  █ █ █ █ 4
3 █ █ █ █  3
2 ♙♙♙♙♙♙♙♙ 2
1 ♖♘♗♕♔♗♘♖ 1
  abcdefgh"#
            .trim_matches('\n');

        assert_eq!(Board::default().to_string(), expected);
    }

    #[test]
    fn can_make_a_move_on_board() {
        let mut board = Board::default();

        let mov = Move::new(Piece::WN, Square::B1, Square::C3);

        board.make_move(mov);

        let expected_board = Board::new(
            [
                [WR, __, WB, WQ, WK, WB, WN, WR],
                [WP, WP, WP, WP, WP, WP, WP, WP],
                [__, __, WN, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [BP, BP, BP, BP, BP, BP, BP, BP],
                [BR, BN, BB, BQ, BK, BB, BN, BR],
            ],
            Player::Black,
        );

        assert_eq!(board, expected_board);
    }

    #[test]
    fn can_make_a_capturing_move_on_board() {
        let mut board = Board::new(
            [
                [WR, WN, WB, WQ, WK, __, WN, WR],
                [WP, WP, WP, WP, __, WP, WP, WP],
                [__, __, __, __, WP, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, WB, __, __, __, BP, __, __],
                [__, __, __, __, __, __, __, __],
                [BP, BP, BP, BP, BP, __, BP, BP],
                [BR, BN, BB, BQ, BK, BB, BN, BR],
            ],
            Player::White,
        );

        let mov = Move::new(Piece::WB, Square::B5, Square::D7);

        board.make_move(mov);

        let expected_board = Board::new(
            [
                [WR, WN, WB, WQ, WK, __, WN, WR],
                [WP, WP, WP, WP, __, WP, WP, WP],
                [__, __, __, __, WP, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, BP, __, __],
                [__, __, __, __, __, __, __, __],
                [BP, BP, BP, WB, BP, __, BP, BP],
                [BR, BN, BB, BQ, BK, BB, BN, BR],
            ],
            Player::Black,
        );

        assert_eq!(board, expected_board);
    }

    #[test]
    fn can_make_a_promoting_move_on_board() {
        let mut board = Board::new(
            [
                [WR, WN, WB, WQ, WK, WB, WN, WR],
                [WP, WP, WP, WP, WP, __, WP, WP],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, BN, __],
                [__, __, __, __, __, BP, __, __],
                [__, __, __, __, __, __, __, __],
                [BP, BP, BP, BP, BP, __, WP, BP],
                [BR, BN, BB, BQ, BK, BB, __, BR],
            ],
            Player::White,
        );

        let mov = Move::new_promoting(Piece::WP, Square::G7, Square::G8, PieceType::Queen);

        board.make_move(mov);

        let expected_board = Board::new(
            [
                [WR, WN, WB, WQ, WK, WB, WN, WR],
                [WP, WP, WP, WP, WP, __, WP, WP],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, BN, __],
                [__, __, __, __, __, BP, __, __],
                [__, __, __, __, __, __, __, __],
                [BP, BP, BP, BP, BP, __, __, BP],
                [BR, BN, BB, BQ, BK, BB, WQ, BR],
            ],
            Player::Black,
        );

        assert_eq!(board, expected_board);
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

    #[ignore]
    #[test]
    fn when_making_an_en_passant_move_the_pawn_is_taken() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [WP, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, BP, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        board.make_move(Move::new(Piece::WP, Square::A2, Square::A4));

        let en_passant = Move::new(Piece::BP, Square::B4, Square::A3);
        board.make_move(en_passant);

        let expected_board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [BP, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_eq!(board, expected_board);
    }
}
