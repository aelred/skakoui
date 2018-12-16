use crate::Player;
use enum_map::Enum;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, Ord, PartialOrd)]
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
}

impl FromStr for Piece {
    type Err = ();

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let piece = match str {
            "♔" => Self::WK,
            "♕" => Self::WQ,
            "♖" => Self::WR,
            "♗" => Self::WB,
            "♘" => Self::WN,
            "♙" => Self::WP,
            "♚" => Self::BK,
            "♛" => Self::BQ,
            "♜" => Self::BR,
            "♝" => Self::BB,
            "♞" => Self::BN,
            "♟" => Self::BP,
            _ => return Err(()),
        };

        Ok(piece)
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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!("K".parse::<Piece>(), Err(()));
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
