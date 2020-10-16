use crate::{Bitboard, Rank};
use anyhow::Error;
use enum_map::Enum;
use std::fmt;
use std::str::FromStr;

pub trait Player: Sized + Copy {
    type Opp: Player;

    fn value(self) -> PlayerV;
    fn opponent(self) -> Self::Opp;
    fn back_rank(self) -> Rank;
    fn pawn_rank(self) -> Rank;
    fn castle_kingside_clear(self) -> Bitboard;
    fn castle_queenside_clear(self) -> Bitboard;
    fn castle_kingside_flag(self) -> u8;
    fn castle_queenside_flag(self) -> u8;
    fn castle_flags(self) -> u8 {
        self.castle_kingside_flag() | self.castle_queenside_flag()
    }
    fn multiplier(self) -> i8;
    fn advance_bitboard(self, bitboard: Bitboard) -> Bitboard;
    fn char(self) -> char;
}

#[macro_export]
macro_rules! typed_player {
    ($p:expr, $f:expr) => {
        match $p {
            $crate::PlayerV::White => $f($crate::White),
            $crate::PlayerV::Black => $f($crate::Black),
        }
    };
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Enum, Ord, PartialOrd, Hash)]
pub enum PlayerV {
    White,
    Black,
}

impl Player for PlayerV {
    type Opp = PlayerV;

    fn value(self) -> PlayerV {
        self
    }

    fn opponent(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    fn back_rank(self) -> Rank {
        match self {
            Self::White => Rank::_1,
            Self::Black => Rank::_8,
        }
    }

    fn pawn_rank(self) -> Rank {
        match self {
            Self::White => Rank::_2,
            Self::Black => Rank::_7,
        }
    }

    fn castle_kingside_clear(self) -> Bitboard {
        const CLEAR: Bitboard = Bitboard::new(0b_01100000);
        match self {
            Self::White => CLEAR,
            Self::Black => CLEAR.reverse(),
        }
    }

    fn castle_queenside_clear(self) -> Bitboard {
        const CLEAR: Bitboard = Bitboard::new(0b_00001110);
        match self {
            Self::White => CLEAR,
            Self::Black => CLEAR.reverse(),
        }
    }

    fn castle_kingside_flag(self) -> u8 {
        match self {
            Self::White => 0b1000_0000,
            Self::Black => 0b0010_0000,
        }
    }

    fn castle_queenside_flag(self) -> u8 {
        match self {
            Self::White => 0b0100_0000,
            Self::Black => 0b0001_0000,
        }
    }

    fn multiplier(self) -> i8 {
        match self {
            Self::White => 1,
            Self::Black => -1,
        }
    }

    fn advance_bitboard(self, bitboard: Bitboard) -> Bitboard {
        match self {
            Self::White => bitboard.shift_rank(1),
            Self::Black => bitboard.shift_rank_neg(1),
        }
    }

    fn char(self) -> char {
        match self {
            Self::White => 'W',
            Self::Black => 'B',
        }
    }
}

impl FromStr for PlayerV {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_fen(s)
    }
}

impl fmt::Display for PlayerV {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_fen())
    }
}

pub trait PlayerT: Copy + Default {
    type Opp: PlayerT;
    const PLAYER: PlayerV;
}

impl<T: PlayerT> Player for T {
    type Opp = <Self as PlayerT>::Opp;

    fn value(self) -> PlayerV {
        Self::PLAYER
    }

    fn opponent(self) -> Self::Opp {
        <Self as PlayerT>::Opp::default()
    }

    fn back_rank(self) -> Rank {
        Self::PLAYER.back_rank()
    }

    fn pawn_rank(self) -> Rank {
        Self::PLAYER.pawn_rank()
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

    fn char(self) -> char {
        Self::PLAYER.char()
    }
}

#[derive(Copy, Clone, Default)]
pub struct White;

#[derive(Copy, Clone, Default)]
pub struct Black;

impl PlayerT for White {
    type Opp = Black;
    const PLAYER: PlayerV = PlayerV::White;
}

impl PlayerT for Black {
    type Opp = White;
    const PLAYER: PlayerV = PlayerV::Black;
}
