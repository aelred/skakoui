#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod bitboard;
mod board;
mod moves;
mod piece;
mod square;

use enum_map::Enum;

pub use crate::{
    bitboard::{bitboards, Bitboard},
    board::Board,
    moves::Move,
    piece::{Piece, PieceType},
    square::{File, Rank, Square, SquareColor},
};

#[derive(Debug, Eq, PartialEq, Copy, Clone, Enum)]
pub enum Player {
    White,
    Black,
}

impl Player {
    fn opponent(self) -> Player {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}

pub trait PlayerType {
    const PLAYER: Player;
    const DIRECTION: isize;
    const PAWN_RANK: Rank;

    fn advance_bitboard(bitboard: &Bitboard) -> Bitboard;
}

pub struct WhitePlayer;
pub struct BlackPlayer;

impl PlayerType for WhitePlayer {
    const PLAYER: Player = Player::White;
    const DIRECTION: isize = 1;
    const PAWN_RANK: Rank = Rank::_2;

    fn advance_bitboard(bitboard: &Bitboard) -> Bitboard {
        bitboard.shift_rank(1)
    }
}

impl PlayerType for BlackPlayer {
    const PLAYER: Player = Player::Black;
    const DIRECTION: isize = -1;
    const PAWN_RANK: Rank = Rank::_7;

    fn advance_bitboard(bitboard: &Bitboard) -> Bitboard {
        bitboard.shift_rank_neg(1)
    }
}
