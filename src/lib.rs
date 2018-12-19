#![cfg_attr(feature = "strict", deny(warnings))]

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
    const WORST_SCORE: i32;

    fn advance_bitboard(bitboard: &Bitboard) -> Bitboard;

    fn set_alpha_beta(alpha: &mut i32, beta: &mut i32, score: i32);

    fn better_score(new_score: i32, old_score: i32) -> bool;
}

pub struct WhitePlayer;
pub struct BlackPlayer;

impl PlayerType for WhitePlayer {
    type Opp = BlackPlayer;

    const PLAYER: Player = Player::White;
    const DIRECTION: isize = 1;
    const PAWN_RANK: Rank = Rank::_2;
    const WORST_SCORE: i32 = std::i32::MIN;

    fn advance_bitboard(bitboard: &Bitboard) -> Bitboard {
        bitboard.shift_rank(1)
    }

    fn set_alpha_beta(alpha: &mut i32, _: &mut i32, score: i32) {
        *alpha = i32::max(*alpha, score);
    }

    fn better_score(new_score: i32, old_score: i32) -> bool {
        new_score > old_score
    }
}

impl PlayerType for BlackPlayer {
    type Opp = WhitePlayer;

    const PLAYER: Player = Player::Black;
    const DIRECTION: isize = -1;
    const PAWN_RANK: Rank = Rank::_7;
    const WORST_SCORE: i32 = std::i32::MAX;

    fn advance_bitboard(bitboard: &Bitboard) -> Bitboard {
        bitboard.shift_rank_neg(1)
    }

    fn set_alpha_beta(_: &mut i32, beta: &mut i32, score: i32) {
        *beta = i32::min(*beta, score);
    }

    fn better_score(new_score: i32, old_score: i32) -> bool {
        new_score < old_score
    }
}
