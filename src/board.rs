use crate::moves::PlayedMove;
use crate::piece::PieceType::Pawn;
use crate::Move;
use crate::Piece;
use crate::PieceMap;
use crate::PieceType;
use crate::Player;
use crate::Rank;
use crate::Square;
use crate::SquareColor;
use crate::SquareMap;
use crate::{bitboards, PlayerType};
use crate::{Bitboard, WhitePlayer};
use crate::{BlackPlayer, File};
use anyhow::{anyhow, Context, Error};
use arrayvec::ArrayVec;
use enum_map::EnumMap;
use serde::export::Formatter;
use std::convert::TryFrom;
use std::fmt;
use std::ops::BitOr;
use std::str::FromStr;

/// Represents a game in-progress
#[derive(Eq, PartialEq, Clone, Hash)]
pub struct Board {
    /// Bitboards for every piece
    bitboards: PieceMap<Bitboard>,
    /// The player whose turn it is
    player: Player,
    /// Square-wise representation: lookup what piece is on a particular square
    pieces: SquareMap<Option<Piece>>,
    /// Count for each piece
    piece_count: PieceMap<u8>,
    /// Occupancy for white and black
    occupancy_player: EnumMap<Player, Bitboard>,
    /// Occupancy for all pieces
    occupancy: Bitboard,
    /// Castling rights
    flags: BoardFlags,
    /// Number of moves into the game
    plies: u32,
}

impl Board {
    /// Create a new board from piece positions and player turn
    pub fn new(pieces: [[Option<Piece>; 8]; 8], player: Player, mut flags: BoardFlags) -> Self {
        let pieces = SquareMap::from(|square: Square| {
            pieces[square.rank().to_index() as usize][square.file().to_index() as usize]
        });

        let mut unset_flags = 0;
        // Remove any impossible castling options
        if pieces[Square::E1] != Some(Piece::WK) || pieces[Square::H1] != Some(Piece::WR) {
            unset_flags |= WhitePlayer::CASTLE_KINGSIDE_FLAG;
        }
        if pieces[Square::E1] != Some(Piece::WK) || pieces[Square::A1] != Some(Piece::WR) {
            unset_flags |= WhitePlayer::CASTLE_QUEENSIDE_FLAG;
        }
        if pieces[Square::E8] != Some(Piece::BK) || pieces[Square::H8] != Some(Piece::BR) {
            unset_flags |= BlackPlayer::CASTLE_KINGSIDE_FLAG;
        }
        if pieces[Square::E8] != Some(Piece::BK) || pieces[Square::A8] != Some(Piece::BR) {
            unset_flags |= BlackPlayer::CASTLE_QUEENSIDE_FLAG;
        }
        flags.unset(unset_flags);

        Self::with_states(pieces, player, flags)
    }

    /// Parse a board from
    /// [Forsyth-Edwards notation](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation).
    pub fn from_fen(fen: impl Into<String>) -> Result<Board, Error> {
        let fen_str = fen.into();
        let mut fields = fen_str.split_whitespace();
        let pieces_str = fields.next().context("Expected pieces")?;
        let pieces_by_rank = pieces_str.split('/');

        let mut pieces_vec = ArrayVec::<[[Option<Piece>; 8]; 8]>::new();
        for rank in pieces_by_rank {
            let mut rank_vec = ArrayVec::<[Option<Piece>; 8]>::new();
            for c in rank.chars() {
                let s = c.to_string();
                if let Ok(empties) = s.parse::<usize>() {
                    for _ in 0..empties {
                        rank_vec
                            .try_push(None)
                            .with_context(|| anyhow!("More than 8 squares in rank: {}", rank))?;
                    }
                } else if let Ok(piece) = s.parse::<Piece>() {
                    rank_vec
                        .try_push(Some(piece))
                        .with_context(|| anyhow!("More than 8 squares in rank: {}", rank))?;
                }
            }
            pieces_vec
                .try_push(
                    rank_vec
                        .into_inner()
                        .map_err(|_| anyhow!("Less than 8 squares in rank: {}", rank))?,
                )
                .with_context(|| anyhow!("More than 8 ranks: {}", pieces_str))?;
        }
        pieces_vec.reverse();
        let pieces_array = pieces_vec
            .into_inner()
            .map_err(|_| anyhow!("Less than 8 ranks: {}", pieces_str))?;

        let player = fields
            .next()
            .context("Expected player after pieces")?
            .parse::<Player>()?;

        let flags = fields.next().map(|castling| {
            let mut set_flags = 0u8;
            if castling.contains('K') {
                set_flags |= WhitePlayer::CASTLE_KINGSIDE_FLAG;
            }
            if castling.contains('Q') {
                set_flags |= WhitePlayer::CASTLE_QUEENSIDE_FLAG;
            }
            if castling.contains('k') {
                set_flags |= BlackPlayer::CASTLE_KINGSIDE_FLAG;
            }
            if castling.contains('q') {
                set_flags |= BlackPlayer::CASTLE_QUEENSIDE_FLAG;
            }
            BoardFlags(set_flags)
        });

        // TODO: also parse en passant and number of moves

        Ok(Self::new(pieces_array, player, flags.unwrap_or_default()))
    }

    pub fn to_fen(&self) -> String {
        let mut array: [[Option<Piece>; 8]; 8] = [[None; 8]; 8];

        for (square, piece) in self.pieces.iter() {
            array[square.rank().to_index() as usize][square.file().to_index() as usize] = *piece;
        }

        let mut fen = String::new();
        let mut empty_count = 0;

        fn push_empty_count(fen: &mut String, empty_count: &mut i32) {
            if *empty_count != 0 {
                fen.push_str(&empty_count.to_string());
                *empty_count = 0;
            }
        }

        for rank in array.iter().rev() {
            if !fen.is_empty() {
                fen.push('/');
            }

            for square in rank {
                match square {
                    Some(piece) => {
                        push_empty_count(&mut fen, &mut empty_count);
                        fen.push(piece.to_fen())
                    }
                    None => empty_count += 1,
                }
            }
            push_empty_count(&mut fen, &mut empty_count);
        }

        fen.push(' ');
        fen.push(self.player.to_fen());

        fen.push(' ');
        let mut can_castle = false;
        if self.flags.is_set(WhitePlayer::CASTLE_KINGSIDE_FLAG) {
            fen.push('K');
            can_castle = true;
        }
        if self.flags.is_set(WhitePlayer::CASTLE_QUEENSIDE_FLAG) {
            fen.push('Q');
            can_castle = true;
        }
        if self.flags.is_set(BlackPlayer::CASTLE_KINGSIDE_FLAG) {
            fen.push('k');
            can_castle = true;
        }
        if self.flags.is_set(BlackPlayer::CASTLE_QUEENSIDE_FLAG) {
            fen.push('q');
            can_castle = true;
        }
        if !can_castle {
            fen.push('-');
        }

        fen
    }

    fn with_states(pieces: SquareMap<Option<Piece>>, player: Player, flags: BoardFlags) -> Self {
        let mut bitboards = PieceMap::from(|_| bitboards::EMPTY);

        for (square, optional_piece) in pieces.iter() {
            if let Some(piece) = optional_piece {
                bitboards[*piece].set(square);
            }
        }

        let piece_count = PieceMap::from(|piece| bitboards[piece].count());

        let occupancy_player = EnumMap::from(|player| {
            bitboards
                .for_player(player)
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
            piece_count,
            occupancy_player,
            occupancy,
            flags,
            plies: 0,
        }
    }

    /// Get the piece at a particular square
    #[inline]
    pub fn get(&self, square: Square) -> Option<Piece> {
        self.pieces[square]
    }

    /// Get whose turn it is
    #[inline]
    pub fn player(&self) -> Player {
        self.player
    }

    /// Number of half-moves played in the game
    pub fn plies(&self) -> u32 {
        self.plies
    }

    /// Perform a move on the board, mutating the board
    pub fn make_move(&mut self, mov: Move) -> PlayedMove {
        let prev_flags = self.flags;

        let player = self.player();
        let from = mov.from();
        let to = mov.to();

        self.assert_can_move(player, from, to);

        let piece = self.get(from).unwrap();

        let captured_piece_type = if let Some(captured_piece) = self.get(to) {
            self.bitboard_piece_mut(captured_piece).reset(to);
            self.occupancy_player[player.opponent()].reset(to);
            self.piece_count[captured_piece] -= 1;
            Some(captured_piece.piece_type())
        } else {
            None
        };

        self.pieces[from] = None;

        let bitboard = self.bitboard_piece_mut(piece);

        if let Some(promotion_type) = mov.promoting() {
            let promotion = Piece::new(player, promotion_type);
            bitboard.reset(from);
            self.bitboard_piece_mut(promotion).set(to);

            self.pieces[to] = Some(promotion);
            self.piece_count[piece] -= 1;
            self.piece_count[promotion] += 1;
        } else {
            bitboard.move_bit(from, to);

            self.pieces[to] = Some(piece);
        }

        self.occupancy.reset(from);
        self.occupancy.set(to);
        self.occupancy_player[player].move_bit(from, to);

        fn castle_flags(player: Player, square: Square) -> u8 {
            if player.back_rank() == square.rank() {
                match square.file() {
                    File::E => player.castle_flags(),
                    File::H => player.castle_kingside_flag(),
                    File::A => player.castle_queenside_flag(),
                    _ => 0,
                }
            } else {
                0
            }
        }

        let unset_flags = castle_flags(player, from) | castle_flags(player.opponent(), to);
        self.flags.unset(unset_flags);

        if piece.piece_type() == PieceType::King && from.file() == File::E {
            let kingside_castling = to.file() == File::G;
            let queenside_castling = to.file() == File::C;

            if kingside_castling || queenside_castling {
                let (rook_from_file, rook_to_file, flag) = if kingside_castling {
                    (File::H, File::F, player.castle_kingside_flag())
                } else {
                    (File::A, File::D, player.castle_queenside_flag())
                };
                self.flags.unset(flag);

                let rook_from = Square::new(rook_from_file, from.rank());
                let rook_to = Square::new(rook_to_file, to.rank());

                let rook = Piece::new(player, PieceType::Rook);
                debug_assert_eq!(
                    self.pieces[rook_from],
                    Some(rook),
                    "Expected {} at {}",
                    rook,
                    rook_from
                );
                debug_assert_eq!(
                    self.pieces[rook_to], None,
                    "Expected {} to be empty",
                    rook_to
                );

                self.pieces[rook_from] = None;
                self.pieces[rook_to] = Some(rook);
                self.bitboard_piece_mut(rook).move_bit(rook_from, rook_to);
                self.occupancy.move_bit(rook_from, rook_to);
                self.occupancy_player[player].move_bit(rook_from, rook_to);
            }
        }

        self.player = self.player.opponent();
        self.plies += 1;

        self.assert_invariants();

        PlayedMove::new(mov, captured_piece_type, prev_flags)
    }

    /// Undo a move on the board - opposite of [make_move]
    pub fn unmake_move(&mut self, pmov: PlayedMove) {
        let PlayedMove {
            mov,
            capture,
            flags,
        } = pmov;

        let player = self.player().opponent();
        let from = mov.from();
        let to = mov.to();

        self.assert_can_move(player, to, from);

        let piece = if mov.promoting().is_some() {
            Piece::new(player, Pawn)
        } else {
            self.get(to).unwrap()
        };

        self.flags = flags;

        if let Some(captured_piece_type) = capture {
            let captured_piece = Piece::new(player.opponent(), captured_piece_type);
            self.bitboard_piece_mut(captured_piece).set(to);
            self.occupancy_player[player.opponent()].set(to);
            self.pieces[to] = Some(captured_piece);
            self.piece_count[captured_piece] += 1;
        } else {
            self.occupancy.reset(to);
            self.pieces[to] = None;
        }

        let bitboard = self.bitboard_piece_mut(piece);

        if let Some(promotion_type) = mov.promoting() {
            let promotion = Piece::new(player, promotion_type);
            bitboard.set(from);
            self.bitboard_piece_mut(promotion).reset(to);
            self.piece_count[piece] += 1;
            self.piece_count[promotion] -= 1;
        } else {
            bitboard.move_bit(to, from);
        }

        self.pieces[from] = Some(piece);

        self.occupancy.set(from);
        self.occupancy_player[player].move_bit(to, from);

        self.player = player;

        let maybe_castling = piece.piece_type() == PieceType::King && from.file() == File::E;

        if maybe_castling {
            let kingside_castling = to.file() == File::G;
            let queenside_castling = to.file() == File::C;

            if kingside_castling || queenside_castling {
                let (rook_from_file, rook_to_file) = if kingside_castling {
                    (File::H, File::F)
                } else {
                    (File::A, File::D)
                };

                let rook_from = Square::new(rook_from_file, from.rank());
                let rook_to = Square::new(rook_to_file, to.rank());

                let rook = Piece::new(player, PieceType::Rook);
                debug_assert_eq!(
                    self.pieces[rook_to],
                    Some(rook),
                    "Expected {} at {}",
                    rook,
                    rook_to
                );
                debug_assert_eq!(
                    self.pieces[rook_from], None,
                    "Expected {} to be empty",
                    rook_from
                );

                self.pieces[rook_from] = Some(rook);
                self.pieces[rook_to] = None;
                self.bitboard_piece_mut(rook).move_bit(rook_to, rook_from);
                self.occupancy.move_bit(rook_to, rook_from);
                self.occupancy_player[player].move_bit(rook_to, rook_from);
            }
        }

        self.plies -= 1;

        self.assert_invariants();
    }

    #[inline]
    pub fn pieces(&self) -> &SquareMap<Option<Piece>> {
        &self.pieces
    }

    #[inline]
    pub fn bitboards(&self) -> &PieceMap<Bitboard> {
        &self.bitboards
    }

    #[inline]
    pub fn bitboard_piece(&self, piece: Piece) -> &Bitboard {
        &self.bitboards[piece]
    }

    #[inline]
    fn bitboard_piece_mut(&mut self, piece: Piece) -> &mut Bitboard {
        &mut self.bitboards[piece]
    }

    #[inline]
    pub fn occupancy(&self) -> Bitboard {
        self.occupancy
    }

    #[inline]
    pub fn occupancy_player(&self, player: Player) -> Bitboard {
        self.occupancy_player[player]
    }

    #[inline]
    pub fn flags(&self) -> BoardFlags {
        self.flags
    }

    pub fn eval_win(&self) -> i32 {
        let delay_penalty = self.plies() as i32;
        1_000_000 - delay_penalty
    }

    pub fn eval(&self) -> i32 {
        let white_centric_score = 200 * (self.count(Piece::WK) - self.count(Piece::BK))
            + 9 * (self.count(Piece::WQ) - self.count(Piece::BQ))
            + 5 * (self.count(Piece::WR) - self.count(Piece::BR))
            + 3 * (self.count(Piece::WB) - self.count(Piece::BB))
            + 3 * (self.count(Piece::WN) - self.count(Piece::BN))
            + (self.count(Piece::WP) - self.count(Piece::BP));

        white_centric_score * self.player.score_multiplier() * 100
        // TODO: mobility, isolated pawns, blah blah blah
    }

    #[inline]
    pub fn count(&self, piece: Piece) -> i32 {
        i32::from(self.piece_count[piece])
    }

    /// Check this move is possible - not legal - just that it moves a piece of the right colour
    /// to a space without a friendly piece
    fn assert_can_move(&self, player: Player, from: Square, to: Square) {
        if cfg!(debug_assertions) {
            assert_eq!(
                self.pieces[from].map(Piece::player),
                Some(player),
                "{} should have a {} piece, but was {:?}",
                from,
                player,
                self.pieces[from]
            );
            assert_ne!(
                self.pieces[to].map(Piece::player),
                Some(player),
                "{} should not have a {} piece, but was {:?}",
                to,
                player,
                self.pieces[to]
            );
        }
    }

    /// Check invariants for internal redundant board state (only enabled in debug).
    fn assert_invariants(&self) {
        if cfg!(debug_assertions) {
            let mut expected_piece_count = PieceMap::from(|_| 0u8);

            // Use `pieces` as the source-of-truth
            for (square, piece) in self.pieces.iter() {
                for (bb_piece, bitboard) in self.bitboards.iter() {
                    assert_eq!(bitboard.get(square), *piece == Some(bb_piece));
                }

                if let Some(piece) = piece {
                    expected_piece_count[*piece] += 1;
                }

                assert_eq!(self.occupancy.get(square), piece.is_some());

                let player_at_square = piece.map(Piece::player);
                for (player, occupancy_player) in self.occupancy_player.iter() {
                    assert_eq!(
                        occupancy_player.get(square),
                        player_at_square == Some(player)
                    );
                }
            }

            assert_eq!(self.piece_count, expected_piece_count);
        }
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
            BoardFlags::default(),
        )
    }
}

impl fmt::Debug for Board {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        let fen = self.to_fen();
        let fen_url = fen.replace(" ", "_");
        write!(
            f,
            "Board::from_fen(\"{}\") /* https://lichess.org/analysis/{} */",
            fen, fen_url
        )
    }
}

impl fmt::Display for Board {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        let files_str: String = File::VALUES.iter().map(File::to_string).collect();
        f.write_fmt(format_args!("  {}\n", files_str))?;

        for rank in Rank::VALUES.iter().rev() {
            f.write_fmt(format_args!("{} ", rank))?;
            for file in &File::VALUES {
                let square = Square::new(*file, *rank);

                let s = if let Some(piece) = self.get(square) {
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

impl FromStr for Board {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Board::from_fen(s)
    }
}

impl TryFrom<&str> for Board {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Board::from_fen(value)
    }
}

/// Bits: 0bKQkq_xxxx
///
/// K = White can castle kingside
/// Q = White can castle queenside
/// k = Black can castle kingside
/// q = Black can castle queenside
/// x = unused
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct BoardFlags(u8);

impl Default for BoardFlags {
    fn default() -> Self {
        BoardFlags(0b1111_0000)
    }
}

impl fmt::Debug for BoardFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "BoardFlags({:#010b})", self.0)
    }
}

impl BoardFlags {
    pub fn new(x: u8) -> Self {
        BoardFlags(x)
    }

    pub fn is_set(self, mask: u8) -> bool {
        self.0 & mask != 0
    }

    pub fn set(&mut self, mask: u8) {
        self.0 |= mask;
    }

    pub fn unset(&mut self, mask: u8) {
        self.0 &= !mask;
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn can_create_default_chess_board() {
        Board::default();
    }

    #[test]
    fn can_create_board_from_fen_notation() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = Board::from_fen(fen).unwrap();
        assert_eq!(board, Board::default());
    }

    #[test]
    fn default_chess_board_has_pieces_in_position() {
        let board = Board::default();

        // Check some known piece positions are right
        assert_eq!(board.get(Square::A1), Some(Piece::WR));
        assert_eq!(board.get(Square::A2), Some(Piece::WP));
        assert_eq!(board.get(Square::E8), Some(Piece::BK));
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

        let mov = Move::new(Square::B1, Square::C3);

        board.make_move(mov);

        let expect = fen("rnbqkbnr/pppppppp/8/8/8/2N5/PPPPPPPP/R1BQKBNR b");
        assert_eq!(board, expect);
    }

    #[test]
    fn can_make_a_capturing_move_on_board() {
        let mut board = fen("rnbqkbnr/ppppp1pp/8/1B3p2/8/4P3/PPPP1PPP/RNBQK1NR w");

        let mov = Move::new(Square::B5, Square::D7);

        board.make_move(mov);

        let expected_board = fen("rnbqkbnr/pppBp1pp/8/5p2/8/4P3/PPPP1PPP/RNBQK1NR b");
        assert_eq!(board, expected_board);
    }

    #[test]
    fn can_make_a_promoting_move_on_board() {
        let mut board = fen("rnbqkb1r/ppppp1Pp/8/5p2/6n1/8/PPPPP1PP/RNBQKBNR w");

        let mov = Move::new_promoting(Square::G7, Square::G8, PieceType::Queen);

        board.make_move(mov);

        let expected_board = fen("rnbqkbQr/ppppp2p/8/5p2/6n1/8/PPPPP1PP/RNBQKBNR b");
        assert_eq!(board, expected_board);
    }

    #[test]
    fn can_unmake_a_move_on_board() {
        let mut board = fen("rnbqkb1r/ppppp1Pp/8/5p2/6n1/8/PPPPP1PP/RNBQKBNR w");

        let expected_board = board.clone();

        let mov1 = Move::new_promoting(Square::G7, Square::H8, PieceType::Queen);
        let mov2 = Move::new(Square::H7, Square::H5);
        let mov3 = Move::new(Square::H8, Square::H5);

        let pmov1 = board.make_move(mov1);
        let pmov2 = board.make_move(mov2);
        let pmov3 = board.make_move(mov3);
        board.unmake_move(pmov3);
        board.unmake_move(pmov2);
        board.unmake_move(pmov1);

        assert_eq!(board, expected_board);
    }

    #[test]
    fn white_can_castle_kingside() {
        let mut board = fen("8/8/8/8/8/8/8/4K2R w K");
        board.make_move(Move::castle_kingside::<WhitePlayer>());

        let expect = fen("8/8/8/8/8/8/8/5RK1 b -");
        assert_eq!(board, expect);
    }

    #[test]
    fn black_can_castle_kingside() {
        let mut board = fen("4k2r/8/8/8/8/8/8/8 b k");
        board.make_move(Move::castle_kingside::<BlackPlayer>());

        let expect = fen("5rk1/8/8/8/8/8/8/8 w -");
        assert_eq!(board, expect);
    }

    #[test]
    fn white_can_castle_queenside() {
        let mut board = fen("8/8/8/8/8/8/8/R3K3 w Q");
        board.make_move(Move::castle_queenside::<WhitePlayer>());

        let expect = fen("8/8/8/8/8/8/8/2KR4 b -");
        assert_eq!(board, expect);
    }

    #[test]
    fn black_can_castle_queenside() {
        let mut board = fen("r3k3/8/8/8/8/8/8/8 b q");
        board.make_move(Move::castle_queenside::<BlackPlayer>());

        let expect = fen("2kr4/8/8/8/8/8/8/8 w -");
        assert_eq!(board, expect);
    }

    #[test]
    fn moving_king_removes_castling_right() {
        let mut board = fen("r3k2r/8/8/8/8/8/8/8 b kq");
        let expect = fen("r4k1r/8/8/8/8/8/8/8 w -");
        board.make_move(Move::new(Square::E8, Square::F8));
        assert_eq!(board, expect);
    }

    #[test]
    fn moving_kingside_rook_removes_castling_right_kingside() {
        let mut board = fen("r3k2r/8/8/8/8/8/8/8 b kq");
        let expect = fen("r3k1r1/8/8/8/8/8/8/8 w q");
        board.make_move(Move::new(Square::H8, Square::G8));
        assert_eq!(board, expect);
    }

    #[test]
    fn moving_queenside_rook_removes_castling_right_queenside() {
        let mut board = fen("r3k2r/8/8/8/8/8/8/8 b kq");
        let expect = fen("1r2k2r/8/8/8/8/8/8/8 w k");
        board.make_move(Move::new(Square::A8, Square::B8));
        assert_eq!(board, expect);
    }

    #[test]
    fn capturing_kingside_rook_removes_castling_right_kingside() {
        let mut board = fen("r3k2r/7Q/8/8/8/8/8/8 w kq");
        let expect = fen("r3k2Q/8/8/8/8/8/8/8 b q");
        board.make_move(Move::new(Square::H7, Square::H8));
        assert_eq!(board, expect);
    }

    #[test]
    fn capturing_queenside_rook_removes_castling_right_queenside() {
        let mut board = fen("r3k2r/Q7/8/8/8/8/8/8 w kq");
        let expect = fen("Q3k2r/8/8/8/8/8/8/8 b k");
        board.make_move(Move::new(Square::A7, Square::A8));
        assert_eq!(board, expect);
    }

    #[ignore]
    #[test]
    fn when_making_an_en_passant_move_the_pawn_is_taken() {
        let mut board = fen("8/8/8/8/1p6/8/P7/8 w");

        board.make_move(Move::new(Square::A2, Square::A4));

        let en_passant = Move::new(Square::B4, Square::A3);
        board.make_move(en_passant);

        let expected_board = fen("8/8/8/8/8/p7/8/8 w");
        assert_eq!(board, expected_board);
    }

    pub fn fen(fen: &str) -> Board {
        Board::from_fen(fen).unwrap()
    }
}
