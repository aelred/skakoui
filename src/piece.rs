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
    pub const WK: Piece = Piece { player: Player::White, piece_type: PieceType::King };
    pub const WQ: Piece = Piece { player: Player::White, piece_type: PieceType::Queen };
    pub const WR: Piece = Piece { player: Player::White, piece_type: PieceType::Rook };
    pub const WB: Piece = Piece { player: Player::White, piece_type: PieceType::Bishop };
    pub const WN: Piece = Piece { player: Player::White, piece_type: PieceType::Knight };
    pub const WP: Piece = Piece { player: Player::White, piece_type: PieceType::Pawn };
    pub const BK: Piece = Piece { player: Player::Black, piece_type: PieceType::King };
    pub const BQ: Piece = Piece { player: Player::Black, piece_type: PieceType::Queen };
    pub const BR: Piece = Piece { player: Player::Black, piece_type: PieceType::Rook };
    pub const BB: Piece = Piece { player: Player::Black, piece_type: PieceType::Bishop };
    pub const BN: Piece = Piece { player: Player::Black, piece_type: PieceType::Knight };
    pub const BP: Piece = Piece { player: Player::Black, piece_type: PieceType::Pawn };

    pub fn new(player: Player, piece_type: PieceType) -> Self {
        Piece { player, piece_type }
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
        let symbol = match self.player {
            Player::White => {
                match self.piece_type {
                    PieceType::King => "♔",
                    PieceType::Queen => "♕",
                    PieceType::Rook => "♖",
                    PieceType::Bishop => "♗",
                    PieceType::Knight => "♘",
                    PieceType::Pawn => "♙",
                }
            }
            Player::Black => {
                match self.piece_type {
                    PieceType::King => "♚",
                    PieceType::Queen => "♛",
                    PieceType::Rook => "♜",
                    PieceType::Bishop => "♝",
                    PieceType::Knight => "♞",
                    PieceType::Pawn => "♟",
                }
            }
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
