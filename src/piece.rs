use crate::move_generation::PieceT;
use crate::{PieceTypeV, Player, PlayerV};
use anyhow::Error;
use std::fmt;
use std::str::FromStr;

#[derive(PartialEq, Eq, Copy, Clone, Hash, Ord, PartialOrd)]
pub struct Piece {
    player: PlayerV,
    piece_type: PieceTypeV,
}

impl Piece {
    pub const WK: Self = Self {
        player: PlayerV::White,
        piece_type: PieceTypeV::King,
    };
    pub const WQ: Self = Self {
        player: PlayerV::White,
        piece_type: PieceTypeV::Queen,
    };
    pub const WR: Self = Self {
        player: PlayerV::White,
        piece_type: PieceTypeV::Rook,
    };
    pub const WB: Self = Self {
        player: PlayerV::White,
        piece_type: PieceTypeV::Bishop,
    };
    pub const WN: Self = Self {
        player: PlayerV::White,
        piece_type: PieceTypeV::Knight,
    };
    pub const WP: Self = Self {
        player: PlayerV::White,
        piece_type: PieceTypeV::Pawn,
    };
    pub const BK: Self = Self {
        player: PlayerV::Black,
        piece_type: PieceTypeV::King,
    };
    pub const BQ: Self = Self {
        player: PlayerV::Black,
        piece_type: PieceTypeV::Queen,
    };
    pub const BR: Self = Self {
        player: PlayerV::Black,
        piece_type: PieceTypeV::Rook,
    };
    pub const BB: Self = Self {
        player: PlayerV::Black,
        piece_type: PieceTypeV::Bishop,
    };
    pub const BN: Self = Self {
        player: PlayerV::Black,
        piece_type: PieceTypeV::Knight,
    };
    pub const BP: Self = Self {
        player: PlayerV::Black,
        piece_type: PieceTypeV::Pawn,
    };

    pub fn new(player: impl Player, piece_type: PieceTypeV) -> Self {
        Self {
            player: player.value(),
            piece_type,
        }
    }

    pub fn player(self) -> PlayerV {
        self.player
    }

    pub fn piece_type(self) -> PieceTypeV {
        self.piece_type
    }

    pub fn typed(self) -> PieceT<PlayerV, PieceTypeV> {
        PieceT::new(self.player, self.piece_type)
    }
}

impl FromStr for Piece {
    type Err = Error;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        Self::from_fen(str)
    }
}

impl fmt::Debug for Piece {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        let player_char = self.player.char();
        let piece_char = self.piece_type.to_char();
        f.write_fmt(format_args!("Piece::{}{}", player_char, piece_char))
    }
}

impl fmt::Display for Piece {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        let symbol = match self.player {
            PlayerV::White => match self.piece_type {
                PieceTypeV::King => "♔",
                PieceTypeV::Queen => "♕",
                PieceTypeV::Rook => "♖",
                PieceTypeV::Bishop => "♗",
                PieceTypeV::Knight => "♘",
                PieceTypeV::Pawn => "♙",
            },
            PlayerV::Black => match self.piece_type {
                PieceTypeV::King => "♚",
                PieceTypeV::Queen => "♛",
                PieceTypeV::Rook => "♜",
                PieceTypeV::Bishop => "♝",
                PieceTypeV::Knight => "♞",
                PieceTypeV::Pawn => "♟",
            },
        };

        f.write_str(symbol)
    }
}

impl PieceTypeV {
    pub fn to_char(self) -> char {
        match self {
            PieceTypeV::King => 'K',
            PieceTypeV::Queen => 'Q',
            PieceTypeV::Rook => 'R',
            PieceTypeV::Bishop => 'B',
            PieceTypeV::Knight => 'N',
            PieceTypeV::Pawn => 'P',
        }
    }
}

impl fmt::Display for PieceTypeV {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_str(&self.to_fen())
    }
}

impl FromStr for PieceTypeV {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_fen(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Black;
    use pretty_assertions::assert_eq;

    #[test]
    fn displaying_a_piece_returns_unicode() {
        assert_eq!("♔", Piece::WK.to_string());
    }

    #[test]
    fn piece_to_string_is_inverse_of_parse() {
        assert_eq!(Piece::BQ.to_string().parse::<Piece>().unwrap(), Piece::BQ);
        assert_eq!("♖".parse::<Piece>().unwrap().to_string(), "♖");
    }

    #[test]
    fn piece_parse_on_non_piece_string_is_none() {
        assert!("g".parse::<Piece>().is_err());
    }

    #[test]
    fn can_get_player_for_a_piece() {
        assert_eq!(Piece::WK.player(), PlayerV::White);
        assert_eq!(Piece::BB.player(), PlayerV::Black);
    }

    #[test]
    fn can_get_type_for_a_piece() {
        assert_eq!(Piece::WK.piece_type(), PieceTypeV::King);
        assert_eq!(Piece::BB.piece_type(), PieceTypeV::Bishop);
    }

    #[test]
    fn can_create_piece_from_player_and_type() {
        assert_eq!(Piece::new(Black, PieceTypeV::Rook), Piece::BR);
    }
}
