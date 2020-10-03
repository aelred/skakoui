#![cfg_attr(feature = "strict", deny(warnings))]

mod bitboard;
mod board;
mod fen;
mod file;
mod move_generation;
mod moves;
pub mod pgn;
mod piece;
mod piece_map;
mod rank;
mod search;
mod square;

use anyhow::Error;
use enum_map::Enum;

pub use crate::{
    bitboard::{bitboards, Bitboard},
    board::{Board, BoardFlags},
    file::File,
    moves::{Move, PlayedMove},
    piece::{Piece, PieceType},
    piece_map::PieceMap,
    rank::{Rank, RankMap},
    search::Searcher,
    square::{Square, SquareColor, SquareMap},
};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Enum, Ord, PartialOrd, Hash)]
pub enum Player {
    White,
    Black,
}

impl Player {
    fn opponent(self) -> Self {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }

    fn as_char(self) -> char {
        match self {
            Player::White => 'W',
            Player::Black => 'B',
        }
    }

    fn score_multiplier(self) -> i32 {
        match self {
            Player::White => 1,
            Player::Black => -1,
        }
    }

    const fn back_rank(self) -> Rank {
        match self {
            Player::White => Rank::_1,
            Player::Black => Rank::_8,
        }
    }

    const fn castle_kingside_flag(self) -> u8 {
        match self {
            Player::White => 0b1000_0000,
            Player::Black => 0b0010_0000,
        }
    }

    const fn castle_queenside_flag(self) -> u8 {
        match self {
            Player::White => 0b0100_0000,
            Player::Black => 0b0001_0000,
        }
    }

    const fn castle_flags(self) -> u8 {
        self.castle_kingside_flag() | self.castle_queenside_flag()
    }
}

impl FromStr for Player {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Player::from_fen(s)
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_fen())
    }
}

pub trait PlayerType {
    type Opp: PlayerType;

    const PLAYER: Player;
    const DIRECTION: i8;
    const PAWN_RANK: Rank;
    const PROMOTING_RANK: Rank;

    // Bitboards indicating which squares must be clear to allow castling
    const CASTLE_KINGSIDE_CLEAR: Bitboard;
    const CASTLE_QUEENSIDE_CLEAR: Bitboard;

    // Masks to look-up flags [BitboardFlags]
    const CASTLE_KINGSIDE_FLAG: u8 = Self::PLAYER.castle_kingside_flag();
    const CASTLE_QUEENSIDE_FLAG: u8 = Self::PLAYER.castle_queenside_flag();

    fn advance_bitboard(bitboard: Bitboard) -> Bitboard;
}

pub struct WhitePlayer;
pub struct BlackPlayer;

impl PlayerType for WhitePlayer {
    type Opp = BlackPlayer;

    const PLAYER: Player = Player::White;
    const DIRECTION: i8 = 1;
    const PAWN_RANK: Rank = Rank::_2;
    const PROMOTING_RANK: Rank = Rank::_8;
    const CASTLE_KINGSIDE_CLEAR: Bitboard = bitboards::CASTLE_KINGSIDE_CLEAR;
    const CASTLE_QUEENSIDE_CLEAR: Bitboard = bitboards::CASTLE_QUEENSIDE_CLEAR;

    fn advance_bitboard(bitboard: Bitboard) -> Bitboard {
        bitboard.shift_rank(1)
    }
}

impl PlayerType for BlackPlayer {
    type Opp = WhitePlayer;

    const PLAYER: Player = Player::Black;
    const DIRECTION: i8 = -1;
    const PAWN_RANK: Rank = Rank::_7;
    const PROMOTING_RANK: Rank = Rank::_1;
    const CASTLE_KINGSIDE_CLEAR: Bitboard = bitboards::CASTLE_KINGSIDE_CLEAR.reverse();
    const CASTLE_QUEENSIDE_CLEAR: Bitboard = bitboards::CASTLE_QUEENSIDE_CLEAR.reverse();

    fn advance_bitboard(bitboard: Bitboard) -> Bitboard {
        bitboard.shift_rank_neg(1)
    }
}

#[derive(Debug, Clone, Default)]
pub struct GameState {
    pub board: Board,
    moves: Vec<PlayedMove>,
}

impl GameState {
    pub fn new(board: Board) -> Self {
        Self {
            board,
            moves: vec![],
        }
    }

    pub fn push_move(&mut self, mov: Move) {
        let pmov = self.board.make_move(mov);
        self.moves.push(pmov);
    }

    pub fn pop(&mut self) -> Option<Move> {
        self.moves.pop().map(|pmov| {
            self.board.unmake_move(pmov);
            pmov.mov
        })
    }

    pub fn moves(&self) -> impl Iterator<Item = &Move> {
        self.moves.iter().map(|pm| &pm.mov)
    }
}
