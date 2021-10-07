use std::fmt;
use std::str::FromStr;

use anyhow::Error;

use crate::move_generation::PieceType;
use crate::{Bishop, Black, King, Knight, Pawn, PieceTypeV, Player, PlayerV, Queen, Rook, White};

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Piece<P, PT> {
    pub player: P,
    pub piece_type: PT,
}

pub type PieceV = Piece<PlayerV, PieceTypeV>;

impl<P: Player, PT: PieceType> Piece<P, PT> {
    pub fn new(player: P, piece_type: PT) -> Self {
        Self { player, piece_type }
    }

    #[inline]
    pub fn value(&self) -> PieceV {
        Piece::newv(self.player, self.piece_type)
    }
}

impl PieceV {
    pub fn newv(player: impl Player, piece_type: impl PieceType) -> Self {
        Self {
            player: player.value(),
            piece_type: piece_type.value(),
        }
    }
}

impl FromStr for PieceV {
    type Err = Error;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        Self::from_fen(str)
    }
}

impl<P: Player, PT: PieceType> fmt::Debug for Piece<P, PT> {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        let player_char = self.player.char();
        let piece_char = self.piece_type.value().to_char();
        f.write_fmt(format_args!("piece::{}{}", player_char, piece_char))
    }
}

impl<P: Player, PT: PieceType> fmt::Display for Piece<P, PT> {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        let symbol = match self.player.value() {
            PlayerV::White => match self.piece_type.value() {
                PieceTypeV::King => "♔",
                PieceTypeV::Queen => "♕",
                PieceTypeV::Rook => "♖",
                PieceTypeV::Bishop => "♗",
                PieceTypeV::Knight => "♘",
                PieceTypeV::Pawn => "♙",
            },
            PlayerV::Black => match self.piece_type.value() {
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

pub const WK: Piece<White, King> = Piece {
    player: White,
    piece_type: King,
};
pub const WQ: Piece<White, Queen> = Piece {
    player: White,
    piece_type: Queen,
};
pub const WR: Piece<White, Rook> = Piece {
    player: White,
    piece_type: Rook,
};
pub const WB: Piece<White, Bishop> = Piece {
    player: White,
    piece_type: Bishop,
};
pub const WN: Piece<White, Knight> = Piece {
    player: White,
    piece_type: Knight,
};
pub const WP: Piece<White, Pawn> = Piece {
    player: White,
    piece_type: Pawn,
};
pub const BK: Piece<Black, King> = Piece {
    player: Black,
    piece_type: King,
};
pub const BQ: Piece<Black, Queen> = Piece {
    player: Black,
    piece_type: Queen,
};
pub const BR: Piece<Black, Rook> = Piece {
    player: Black,
    piece_type: Rook,
};
pub const BB: Piece<Black, Bishop> = Piece {
    player: Black,
    piece_type: Bishop,
};
pub const BN: Piece<Black, Knight> = Piece {
    player: Black,
    piece_type: Knight,
};
pub const BP: Piece<Black, Pawn> = Piece {
    player: Black,
    piece_type: Pawn,
};

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::Black;

    use super::*;

    #[test]
    fn displaying_a_piece_returns_unicode() {
        assert_eq!("♔", WK.to_string());
    }

    #[test]
    fn piece_to_string_is_inverse_of_parse() {
        assert_eq!(BQ.to_string().parse::<PieceV>().unwrap(), BQ.value());
        assert_eq!("♖".parse::<PieceV>().unwrap().to_string(), "♖");
    }

    #[test]
    fn piece_parse_on_non_piece_string_is_none() {
        assert!("g".parse::<PieceV>().is_err());
    }

    #[test]
    fn can_get_player_for_a_piece() {
        assert_eq!(WK.player, White);
        assert_eq!(BB.player, Black);
    }

    #[test]
    fn can_get_type_for_a_piece() {
        assert_eq!(WK.piece_type, King);
        assert_eq!(BB.piece_type, Bishop);
    }

    #[test]
    fn can_create_piece_from_player_and_type() {
        assert_eq!(Piece::new(Black, Rook), BR);
    }
}
