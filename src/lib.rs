#![cfg_attr(feature = "strict", deny(warnings))]

mod bitboard;
mod board;
mod file;
mod move_generation;
mod moves;
mod piece;
mod piece_map;
mod rank;
mod search;
mod square;

use enum_map::Enum;

pub use crate::{
    bitboard::{bitboards, Bitboard},
    board::Board,
    file::File,
    moves::Move,
    piece::{Piece, PieceType},
    piece_map::PieceMap,
    rank::{Rank, RankMap},
    search::Searcher,
    square::{Square, SquareColor, SquareMap},
};
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

    fn to_fen(self) -> char {
        self.as_char().to_ascii_lowercase()
    }

    fn score_multiplier(self) -> i32 {
        match self {
            Player::White => 1,
            Player::Black => -1,
        }
    }
}

impl FromStr for Player {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let player = match s {
            "W" | "w" => Player::White,
            "B" | "b" => Player::Black,
            _ => return Err(()),
        };
        Ok(player)
    }
}

pub trait PlayerType {
    type Opp: PlayerType;

    const PLAYER: Player;
    const DIRECTION: i8;
    const PAWN_RANK: Rank;
    const LAST_RANK: Rank;

    fn advance_bitboard(bitboard: Bitboard) -> Bitboard;
}

pub struct WhitePlayer;
pub struct BlackPlayer;

impl PlayerType for WhitePlayer {
    type Opp = BlackPlayer;

    const PLAYER: Player = Player::White;
    const DIRECTION: i8 = 1;
    const PAWN_RANK: Rank = Rank::_2;
    const LAST_RANK: Rank = Rank::_8;

    fn advance_bitboard(bitboard: Bitboard) -> Bitboard {
        bitboard.shift_rank(1)
    }
}

impl PlayerType for BlackPlayer {
    type Opp = WhitePlayer;

    const PLAYER: Player = Player::Black;
    const DIRECTION: i8 = -1;
    const PAWN_RANK: Rank = Rank::_7;
    const LAST_RANK: Rank = Rank::_1;

    fn advance_bitboard(bitboard: Bitboard) -> Bitboard {
        bitboard.shift_rank_neg(1)
    }
}
