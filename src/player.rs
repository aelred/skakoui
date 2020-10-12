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
    pub fn opponent(self) -> Self {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }

    pub fn as_char(self) -> char {
        match self {
            Player::White => 'W',
            Player::Black => 'B',
        }
    }

    pub fn score_multiplier(self) -> i32 {
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
    const DIRECTION: i8;
    const BACK_RANK: Rank;
    const PAWN_RANK: Rank;
    const PROMOTING_RANK: Rank = Self::Opp::BACK_RANK;

    // Bitboards indicating which squares must be clear to allow castling
    const CASTLE_KINGSIDE_CLEAR: Bitboard = Self::PLAYER.castle_kingside_clear();
    const CASTLE_QUEENSIDE_CLEAR: Bitboard = Self::PLAYER.castle_queenside_clear();

    // Masks to look-up flags [BitboardFlags]
    const CASTLE_KINGSIDE_FLAG: u8 = Self::PLAYER.castle_kingside_flag();
    const CASTLE_QUEENSIDE_FLAG: u8 = Self::PLAYER.castle_queenside_flag();

    fn value(self) -> Player {
        Self::PLAYER
    }

    fn opponent(self) -> Self::Opp {
        Self::Opp::default()
    }

    fn back_rank(self) -> Rank {
        Self::BACK_RANK
    }

    fn pawn_rank(self) -> Rank {
        Self::PAWN_RANK
    }

    fn promoting_rank(self) -> Rank {
        Self::PROMOTING_RANK
    }

    fn castle_kingside_clear(self) -> Bitboard {
        Self::CASTLE_KINGSIDE_CLEAR
    }

    fn castle_queenside_clear(self) -> Bitboard {
        Self::CASTLE_QUEENSIDE_CLEAR
    }

    fn castle_kingside_flag(self) -> u8 {
        Self::CASTLE_KINGSIDE_FLAG
    }

    fn castle_queenside_flag(self) -> u8 {
        Self::CASTLE_QUEENSIDE_FLAG
    }

    fn direction(self) -> i8 {
        Self::DIRECTION
    }

    fn advance_bitboard(self, bitboard: Bitboard) -> Bitboard;
}

#[derive(Copy, Clone, Default)]
pub struct WhitePlayer;

#[derive(Copy, Clone, Default)]
pub struct BlackPlayer;

impl PlayerType for WhitePlayer {
    type Opp = BlackPlayer;

    const PLAYER: Player = Player::White;
    const DIRECTION: i8 = 1;
    const BACK_RANK: Rank = Rank::_1;
    const PAWN_RANK: Rank = Rank::_2;

    fn advance_bitboard(self, bitboard: Bitboard) -> Bitboard {
        bitboard.shift_rank(1)
    }
}

impl PlayerType for BlackPlayer {
    type Opp = WhitePlayer;

    const PLAYER: Player = Player::Black;
    const DIRECTION: i8 = -1;
    const BACK_RANK: Rank = Rank::_8;
    const PAWN_RANK: Rank = Rank::_7;

    fn advance_bitboard(self, bitboard: Bitboard) -> Bitboard {
        bitboard.shift_rank_neg(1)
    }
}
