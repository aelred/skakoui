#![cfg_attr(feature = "strict", deny(warnings))]

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod bitboard;
mod board;
mod file;
mod move_generation;
mod moves;
mod piece;
mod rank;
mod square;

use enum_map::Enum;

pub mod search;

pub use crate::{
    bitboard::{bitboards, Bitboard},
    board::Board,
    file::File,
    moves::Move,
    piece::{Piece, PieceType},
    rank::Rank,
    square::{Square, SquareColor},
};

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
}

pub trait PlayerType {
    type Opp: PlayerType;

    const PLAYER: Player;
    const DIRECTION: isize;
    const PAWN_RANK: Rank;

    fn advance_bitboard(bitboard: &Bitboard) -> Bitboard;
}

pub struct WhitePlayer;
pub struct BlackPlayer;

impl PlayerType for WhitePlayer {
    type Opp = BlackPlayer;

    const PLAYER: Player = Player::White;
    const DIRECTION: isize = 1;
    const PAWN_RANK: Rank = Rank::_2;

    fn advance_bitboard(bitboard: &Bitboard) -> Bitboard {
        bitboard.shift_rank(1)
    }
}

impl PlayerType for BlackPlayer {
    type Opp = WhitePlayer;

    const PLAYER: Player = Player::Black;
    const DIRECTION: isize = -1;
    const PAWN_RANK: Rank = Rank::_7;

    fn advance_bitboard(bitboard: &Bitboard) -> Bitboard {
        bitboard.shift_rank_neg(1)
    }
}
