use crate::{bitboards, Bitboard, Rank};
use anyhow::Error;
use enum_map::Enum;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Enum, Ord, PartialOrd, Hash)]
pub enum Player {
    White,
    Black,
}

impl Player {
    pub const fn opponent(self) -> Self {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }

    pub const fn as_char(self) -> char {
        match self {
            Player::White => 'W',
            Player::Black => 'B',
        }
    }

    pub const fn multiplier(self) -> i8 {
        match self {
            Player::White => 1,
            Player::Black => -1,
        }
    }

    pub const fn back_rank(self) -> Rank {
        match self {
            Player::White => Rank::_1,
            Player::Black => Rank::_8,
        }
    }

    pub const fn pawn_rank(self) -> Rank {
        match self {
            Player::White => Rank::_2,
            Player::Black => Rank::_7,
        }
    }

    pub const fn promoting_rank(self) -> Rank {
        self.opponent().back_rank()
    }

    pub const fn castle_kingside_flag(self) -> u8 {
        match self {
            Player::White => 0b1000_0000,
            Player::Black => 0b0010_0000,
        }
    }

    pub const fn castle_queenside_flag(self) -> u8 {
        match self {
            Player::White => 0b0100_0000,
            Player::Black => 0b0001_0000,
        }
    }

    pub const fn castle_kingside_clear(self) -> Bitboard {
        match self {
            Player::White => bitboards::CASTLE_KINGSIDE_CLEAR,
            Player::Black => bitboards::CASTLE_KINGSIDE_CLEAR.reverse(),
        }
    }

    pub const fn castle_queenside_clear(self) -> Bitboard {
        match self {
            Player::White => bitboards::CASTLE_QUEENSIDE_CLEAR,
            Player::Black => bitboards::CASTLE_QUEENSIDE_CLEAR.reverse(),
        }
    }

    pub const fn castle_flags(self) -> u8 {
        self.castle_kingside_flag() | self.castle_queenside_flag()
    }

    pub fn advance_bitboard(self, bitboard: Bitboard) -> Bitboard {
        match self {
            Player::White => bitboard.shift_rank(1),
            Player::Black => bitboard.shift_rank_neg(1),
        }
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

pub trait PlayerType: Sized + Copy + Default {
    type Opp: PlayerType;
    const PLAYER: Player;

    fn value(self) -> Player {
        Self::PLAYER
    }

    fn opponent(self) -> Self::Opp {
        Self::Opp::default()
    }

    fn back_rank(self) -> Rank {
        Self::PLAYER.back_rank()
    }

    fn pawn_rank(self) -> Rank {
        Self::PLAYER.pawn_rank()
    }

    fn promoting_rank(self) -> Rank {
        Self::PLAYER.promoting_rank()
    }

    fn castle_kingside_clear(self) -> Bitboard {
        Self::PLAYER.castle_kingside_clear()
    }

    fn castle_queenside_clear(self) -> Bitboard {
        Self::PLAYER.castle_queenside_clear()
    }

    fn castle_kingside_flag(self) -> u8 {
        Self::PLAYER.castle_kingside_flag()
    }

    fn castle_queenside_flag(self) -> u8 {
        Self::PLAYER.castle_queenside_flag()
    }

    fn multiplier(self) -> i8 {
        Self::PLAYER.multiplier()
    }

    fn advance_bitboard(self, bitboard: Bitboard) -> Bitboard {
        Self::PLAYER.advance_bitboard(bitboard)
    }
}

#[derive(Copy, Clone, Default)]
pub struct WhitePlayer;

#[derive(Copy, Clone, Default)]
pub struct BlackPlayer;

impl PlayerType for WhitePlayer {
    type Opp = BlackPlayer;
    const PLAYER: Player = Player::White;
}

impl PlayerType for BlackPlayer {
    type Opp = WhitePlayer;
    const PLAYER: Player = Player::Black;
}
