use crate::bitboards;
use crate::Bitboard;
use crate::File;
use crate::Move;
use crate::Piece;
use crate::PieceType;
use crate::Player;
use crate::Rank;
use crate::Square;
use crate::SquareColor;
use enum_map::EnumMap;
use std::fmt;
use std::ops::BitOr;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Board {
    bitboards: EnumMap<Player, EnumMap<PieceType, Bitboard>>,
    player: Player,
    pieces: EnumMap<Rank, EnumMap<File, Option<Piece>>>,
    occupancy_player: EnumMap<Player, Bitboard>,
    occupancy: Bitboard,
    board_states: Vec<BoardState>,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
struct BoardState {
    captured_piece_type: Option<PieceType>,
}

impl Board {
    pub fn new(pieces_array: [[Option<Piece>; 8]; 8], player: Player) -> Self {
        Self::with_states(pieces_array, player, vec![])
    }

    fn with_states(
        pieces_array: [[Option<Piece>; 8]; 8],
        player: Player,
        board_states: Vec<BoardState>,
    ) -> Self {
        let mut bitboards = EnumMap::from(|_| EnumMap::from(|_| bitboards::EMPTY));

        let pieces = EnumMap::from(|rank: Rank| {
            EnumMap::from(|file: File| pieces_array[rank.to_index()][file.to_index()])
        });

        for (rank, pieces_rank) in pieces {
            for (file, optional_piece) in pieces_rank {
                if let Some(piece) = optional_piece {
                    let square = Square::new(file, rank);
                    bitboards[piece.player()][piece.piece_type()].set(square);
                }
            }
        }

        let occupancy_player = EnumMap::from(|player| {
            bitboards[player]
                .values()
                .fold(bitboards::EMPTY, BitOr::bitor)
        });

        let occupancy = occupancy_player
            .values()
            .fold(bitboards::EMPTY, BitOr::bitor);

        Board {
            bitboards,
            player,
            pieces,
            occupancy_player,
            occupancy,
            board_states,
        }
    }

    fn get(&self, square: Square) -> Option<Piece> {
        self.pieces[square.rank()][square.file()]
    }

    pub fn player(&self) -> Player {
        self.player
    }

    pub fn make_move(&mut self, mov: Move) {
        let player = self.player();
        let from = mov.from();
        let to = mov.to();
        let piece = Piece::new(player, mov.piece_type());

        let captured_piece_type;

        if let Some(captured_piece) = self.get(to) {
            self.bitboard_piece_mut(captured_piece).reset(to);
            self.occupancy_player[player.opponent()].reset(to);
            captured_piece_type = Some(captured_piece.piece_type());
        } else {
            captured_piece_type = None;
        }

        self.pieces[from.rank()][from.file()] = None;

        let bitboard = self.bitboard_piece_mut(piece);

        if let Some(promotion_type) = mov.promoting() {
            let promotion = Piece::new(player, promotion_type);
            bitboard.reset(from);
            self.bitboard_piece_mut(promotion).set(to);

            self.pieces[to.rank()][to.file()] = Some(promotion);
        } else {
            bitboard.move_bit(from, to);

            self.pieces[to.rank()][to.file()] = Some(piece);
        }

        self.occupancy.reset(from);
        self.occupancy.set(to);
        self.occupancy_player[player].move_bit(from, to);

        self.player = self.player.opponent();

        let new_board_state = BoardState {
            captured_piece_type,
        };

        self.board_states.push(new_board_state);
    }

    pub fn unmake_move(&mut self, mov: Move) {
        let player = self.player().opponent();
        let from = mov.from();
        let to = mov.to();
        let piece = Piece::new(player, mov.piece_type());

        let BoardState {
            captured_piece_type,
        } = self.board_states.pop().expect("No move to undo");

        if let Some(captured_piece_type) = captured_piece_type {
            let captured_piece = Piece::new(player.opponent(), captured_piece_type);
            self.bitboard_piece_mut(captured_piece).set(to);
            self.occupancy_player[player.opponent()].set(to);
            self.pieces[to.rank()][to.file()] = Some(captured_piece);
        } else {
            self.occupancy.reset(to);
            self.pieces[to.rank()][to.file()] = None;
        }

        let bitboard = self.bitboard_piece_mut(piece);

        if let Some(promotion_type) = mov.promoting() {
            let promotion = Piece::new(player, promotion_type);
            bitboard.set(from);
            self.bitboard_piece_mut(promotion).reset(to);
        } else {
            bitboard.move_bit(to, from);
        }

        self.pieces[from.rank()][from.file()] = Some(piece);

        self.occupancy.set(from);
        self.occupancy_player[player].move_bit(to, from);

        self.player = player;
    }

    pub fn bitboards(&self) -> &EnumMap<Player, EnumMap<PieceType, Bitboard>> {
        &self.bitboards
    }

    pub fn bitboard_piece(&self, piece: Piece) -> &Bitboard {
        &self.bitboards[piece.player()][piece.piece_type()]
    }

    fn bitboard_piece_mut(&mut self, piece: Piece) -> &mut Bitboard {
        &mut self.bitboards[piece.player()][piece.piece_type()]
    }

    pub fn occupancy(&self) -> Bitboard {
        self.occupancy
    }

    pub fn occupancy_player(&self, player: Player) -> Bitboard {
        self.occupancy_player[player]
    }

    pub fn eval(&self) -> i32 {
        let white_centric_score = 200 * (self.count(Piece::WK) - self.count(Piece::BK))
            + 9 * (self.count(Piece::WQ) - self.count(Piece::BQ))
            + 5 * (self.count(Piece::WR) - self.count(Piece::BR))
            + 3 * (self.count(Piece::WB) - self.count(Piece::BB))
            + 3 * (self.count(Piece::WN) - self.count(Piece::BN))
            + (self.count(Piece::WP) - self.count(Piece::BP));

        white_centric_score * self.player.score_multiplier()
        // TODO: mobility, isolated pawns, blah blah blah
    }

    pub fn count(&self, piece: Piece) -> i32 {
        self.bitboard_piece(piece).count() as i32
    }
}

impl Default for Board {
    fn default() -> Self {
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

        Board::new(
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
        )
    }
}

impl fmt::Display for Board {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        let files_str: String = File::VALUES.iter().map(File::to_string).collect();
        f.write_fmt(format_args!("  {}\n", files_str))?;

        for (rank, pieces) in self.pieces.iter().rev() {
            f.write_fmt(format_args!("{} ", rank))?;
            for (file, optional_piece) in pieces {
                let square = Square::new(file, rank);

                let s = if let Some(piece) = optional_piece {
                    piece.to_string()
                } else {
                    let col = match square.color() {
                        SquareColor::White => " ",
                        SquareColor::Black => "█",
                    };
                    col.to_string()
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
    use pretty_assertions::assert_eq;

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

        let mov = Move::new(PieceType::Knight, Square::B1, Square::C3);

        board.make_move(mov);

        let expected_board = Board::with_states(
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
            vec![BoardState {
                captured_piece_type: None,
            }],
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

        let mov = Move::new(PieceType::Bishop, Square::B5, Square::D7);

        board.make_move(mov);

        let expected_board = Board::with_states(
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
            vec![BoardState {
                captured_piece_type: Some(PieceType::Pawn),
            }],
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

        let mov = Move::new_promoting(PieceType::Pawn, Square::G7, Square::G8, PieceType::Queen);

        board.make_move(mov);

        let expected_board = Board::with_states(
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
            vec![BoardState {
                captured_piece_type: None,
            }],
        );

        assert_eq!(board, expected_board);
    }

    #[test]
    fn can_unmake_a_move_on_board() {
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

        let expected_board = board.clone();

        let mov1 = Move::new_promoting(PieceType::Pawn, Square::G7, Square::H8, PieceType::Queen);
        let mov2 = Move::new(PieceType::Pawn, Square::H7, Square::H5);
        let mov3 = Move::new(PieceType::Queen, Square::H8, Square::H5);

        board.make_move(mov1);
        board.make_move(mov2);
        board.make_move(mov3);
        board.unmake_move(mov3);
        board.unmake_move(mov2);
        board.unmake_move(mov1);

        assert_eq!(board, expected_board);
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

        board.make_move(Move::new(PieceType::Pawn, Square::A2, Square::A4));

        let en_passant = Move::new(PieceType::Pawn, Square::B4, Square::A3);
        board.make_move(en_passant);

        let expected_board = Board::with_states(
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
            vec![BoardState {
                captured_piece_type: Some(PieceType::Pawn),
            }],
        );

        assert_eq!(board, expected_board);
    }
}
