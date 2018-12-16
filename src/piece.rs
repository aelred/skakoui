use crate::Player;
use enum_map::Enum;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Enum, Hash, Ord, PartialOrd)]
pub enum Piece {
    WK,
    WQ,
    WR,
    WB,
    WN,
    WP,
    BK,
    BQ,
    BR,
    BB,
    BN,
    BP,
}

#[derive(Debug, Eq, PartialEq, Enum, Copy, Clone)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl Piece {
    pub fn new(player: Player, piece_type: PieceType) -> Self {
        match player {
            Player::White => match piece_type {
                PieceType::King => Piece::WK,
                PieceType::Queen => Piece::WQ,
                PieceType::Rook => Piece::WR,
                PieceType::Bishop => Piece::WB,
                PieceType::Knight => Piece::WN,
                PieceType::Pawn => Piece::WP,
            },
            Player::Black => match piece_type {
                PieceType::King => Piece::BK,
                PieceType::Queen => Piece::BQ,
                PieceType::Rook => Piece::BR,
                PieceType::Bishop => Piece::BB,
                PieceType::Knight => Piece::BN,
                PieceType::Pawn => Piece::BP,
            },
        }
    }

    pub fn player(self) -> Player {
        match self {
            Piece::WK => Player::White,
            Piece::WQ => Player::White,
            Piece::WR => Player::White,
            Piece::WB => Player::White,
            Piece::WN => Player::White,
            Piece::WP => Player::White,
            Piece::BK => Player::Black,
            Piece::BQ => Player::Black,
            Piece::BR => Player::Black,
            Piece::BB => Player::Black,
            Piece::BN => Player::Black,
            Piece::BP => Player::Black,
        }
    }

    pub fn piece_type(self) -> PieceType {
        match self {
            Piece::WK => PieceType::King,
            Piece::WQ => PieceType::Queen,
            Piece::WR => PieceType::Rook,
            Piece::WB => PieceType::Bishop,
            Piece::WN => PieceType::Knight,
            Piece::WP => PieceType::Pawn,
            Piece::BK => PieceType::King,
            Piece::BQ => PieceType::Queen,
            Piece::BR => PieceType::Rook,
            Piece::BB => PieceType::Bishop,
            Piece::BN => PieceType::Knight,
            Piece::BP => PieceType::Pawn,
        }
    }
}

impl FromStr for Piece {
    type Err = ();

    fn from_str(str: &str) -> Result<Piece, Self::Err> {
        let piece = match str {
            "♔" => Piece::WK,
            "♕" => Piece::WQ,
            "♖" => Piece::WR,
            "♗" => Piece::WB,
            "♘" => Piece::WN,
            "♙" => Piece::WP,
            "♚" => Piece::BK,
            "♛" => Piece::BQ,
            "♜" => Piece::BR,
            "♝" => Piece::BB,
            "♞" => Piece::BN,
            "♟" => Piece::BP,
            _ => return Err(()),
        };

        Ok(piece)
    }
}

impl fmt::Display for Piece {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        let symbol = match self {
            Piece::WK => "♔",
            Piece::WQ => "♕",
            Piece::WR => "♖",
            Piece::WB => "♗",
            Piece::WN => "♘",
            Piece::WP => "♙",
            Piece::BK => "♚",
            Piece::BQ => "♛",
            Piece::BR => "♜",
            Piece::BB => "♝",
            Piece::BN => "♞",
            Piece::BP => "♟",
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
