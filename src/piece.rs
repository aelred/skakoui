use crate::Player;
use core::fmt::Write;
use enum_map::Enum;
use std::fmt;
use std::str::FromStr;

#[derive(PartialEq, Eq, Copy, Clone, Hash, Ord, PartialOrd)]
pub struct Piece {
    player: Player,
    piece_type: PieceType,
}

#[derive(Debug, Eq, PartialEq, Enum, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl Piece {
    pub const WK: Self = Self {
        player: Player::White,
        piece_type: PieceType::King,
    };
    pub const WQ: Self = Self {
        player: Player::White,
        piece_type: PieceType::Queen,
    };
    pub const WR: Self = Self {
        player: Player::White,
        piece_type: PieceType::Rook,
    };
    pub const WB: Self = Self {
        player: Player::White,
        piece_type: PieceType::Bishop,
    };
    pub const WN: Self = Self {
        player: Player::White,
        piece_type: PieceType::Knight,
    };
    pub const WP: Self = Self {
        player: Player::White,
        piece_type: PieceType::Pawn,
    };
    pub const BK: Self = Self {
        player: Player::Black,
        piece_type: PieceType::King,
    };
    pub const BQ: Self = Self {
        player: Player::Black,
        piece_type: PieceType::Queen,
    };
    pub const BR: Self = Self {
        player: Player::Black,
        piece_type: PieceType::Rook,
    };
    pub const BB: Self = Self {
        player: Player::Black,
        piece_type: PieceType::Bishop,
    };
    pub const BN: Self = Self {
        player: Player::Black,
        piece_type: PieceType::Knight,
    };
    pub const BP: Self = Self {
        player: Player::Black,
        piece_type: PieceType::Pawn,
    };

    pub fn new(player: Player, piece_type: PieceType) -> Self {
        Self { player, piece_type }
    }

    pub fn player(self) -> Player {
        self.player
    }

    pub fn piece_type(self) -> PieceType {
        self.piece_type
    }

    pub fn to_fen(&self) -> char {
        let c = self.piece_type.as_char();
        if self.player == Player::White {
            c.to_ascii_uppercase()
        } else {
            c.to_ascii_lowercase()
        }
    }
}

impl FromStr for Piece {
    type Err = ();

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let piece = match str {
            "♔" | "K" => Self::WK,
            "♕" | "Q" => Self::WQ,
            "♖" | "R" => Self::WR,
            "♗" | "B" => Self::WB,
            "♘" | "N" => Self::WN,
            "♙" | "P" => Self::WP,
            "♚" | "k" => Self::BK,
            "♛" | "q" => Self::BQ,
            "♜" | "r" => Self::BR,
            "♝" | "b" => Self::BB,
            "♞" | "n" => Self::BN,
            "♟" | "p" => Self::BP,
            _ => return Err(()),
        };

        Ok(piece)
    }
}

impl fmt::Debug for Piece {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        let player_char = self.player.as_char();
        let piece_char = self.piece_type.as_char();
        f.write_fmt(format_args!("Piece::{}{}", player_char, piece_char))
    }
}

impl fmt::Display for Piece {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        let symbol = match self.player {
            Player::White => match self.piece_type {
                PieceType::King => "♔",
                PieceType::Queen => "♕",
                PieceType::Rook => "♖",
                PieceType::Bishop => "♗",
                PieceType::Knight => "♘",
                PieceType::Pawn => "♙",
            },
            Player::Black => match self.piece_type {
                PieceType::King => "♚",
                PieceType::Queen => "♛",
                PieceType::Rook => "♜",
                PieceType::Bishop => "♝",
                PieceType::Knight => "♞",
                PieceType::Pawn => "♟",
            },
        };

        f.write_str(symbol)
    }
}

impl PieceType {
    fn as_char(self) -> char {
        match self {
            PieceType::King => 'K',
            PieceType::Queen => 'Q',
            PieceType::Rook => 'R',
            PieceType::Bishop => 'B',
            PieceType::Knight => 'N',
            PieceType::Pawn => 'P',
        }
    }
}

impl fmt::Display for PieceType {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_char(self.as_char())
    }
}

impl FromStr for PieceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let piece_type = match s {
            "K" => PieceType::King,
            "Q" => PieceType::Queen,
            "R" => PieceType::Rook,
            "B" => PieceType::Bishop,
            "N" => PieceType::Knight,
            "P" => PieceType::Pawn,
            _ => return Err("couldn't recognise piece".to_string()),
        };

        Ok(piece_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn displaying_a_piece_returns_unicode() {
        assert_eq!("♔", Piece::WK.to_string());
    }

    #[test]
    fn piece_to_string_is_inverse_of_parse() {
        assert_eq!(Piece::BQ.to_string().parse(), Ok(Piece::BQ));
        assert_eq!("♖".parse::<Piece>().unwrap().to_string(), "♖");
    }

    #[test]
    fn piece_parse_on_non_piece_string_is_none() {
        assert_eq!("g".parse::<Piece>(), Err(()));
    }

    #[test]
    fn can_get_player_for_a_piece() {
        assert_eq!(Piece::WK.player(), Player::White);
        assert_eq!(Piece::BB.player(), Player::Black);
    }

    #[test]
    fn can_get_type_for_a_piece() {
        assert_eq!(Piece::WK.piece_type(), PieceType::King);
        assert_eq!(Piece::BB.piece_type(), PieceType::Bishop);
    }

    #[test]
    fn can_create_piece_from_player_and_type() {
        assert_eq!(Piece::new(Player::Black, PieceType::Rook), Piece::BR);
    }
}
