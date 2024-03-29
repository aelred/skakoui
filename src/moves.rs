use crate::{BoardFlags, File, PieceTypeV, Player, Square};
use anyhow::{anyhow, Error};
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Move {
    from: Square,
    to: Square,
    promoting: Option<PieceTypeV>,
}

impl Move {
    pub fn new(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            promoting: None,
        }
    }

    pub fn new_promoting(from: Square, to: Square, promoting: PieceTypeV) -> Self {
        Self {
            from,
            to,
            promoting: Some(promoting),
        }
    }

    pub fn castle_kingside(player: impl Player) -> Self {
        Self {
            from: Square::new(File::E, player.back_rank()),
            to: Square::new(File::KINGSIDE, player.back_rank()),
            promoting: None,
        }
    }

    pub fn castle_queenside(player: impl Player) -> Self {
        Self {
            from: Square::new(File::E, player.back_rank()),
            to: Square::new(File::QUEENSIDE, player.back_rank()),
            promoting: None,
        }
    }

    pub fn from(self) -> Square {
        self.from
    }

    pub fn to(self) -> Square {
        self.to
    }

    pub fn promoting(self) -> Option<PieceTypeV> {
        self.promoting
    }

    pub fn mut_promoting(&mut self) -> &mut Option<PieceTypeV> {
        &mut self.promoting
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mov!(")?;
        fmt::Display::fmt(self, f)?;
        write!(f, ")")
    }
}

impl fmt::Display for Move {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        if let Some(promoting) = self.promoting {
            f.write_fmt(format_args!("{}{}{}", self.from, self.to, promoting))
        } else {
            f.write_fmt(format_args!("{}{}", self.from, self.to))
        }
    }
}

impl FromStr for Move {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let from = s
            .get(0..2)
            .ok_or_else(|| anyhow!("couldn't index string"))?
            .parse::<Square>()?;
        let to = s
            .get(2..4)
            .ok_or_else(|| anyhow!("couldn't index string"))?
            .parse::<Square>()?;

        let promoting = if let Some(promoting_str) = s.get(4..) {
            if promoting_str.is_empty() {
                None
            } else {
                Some(promoting_str.parse::<PieceTypeV>()?)
            }
        } else {
            None
        };

        Ok(Move {
            from,
            to,
            promoting,
        })
    }
}

#[macro_export]
macro_rules! mov {
    ($mov:expr) => {
        stringify!($mov).parse::<$crate::Move>().unwrap()
    };
}

/// Move that has been played with extra information so it can be un-done.
#[derive(Debug, Copy, Clone)]
pub struct PlayedMove {
    pub mov: Move,
    pub capture: Option<PieceTypeV>,
    pub en_passant_capture: bool,
    pub flags: BoardFlags,
}

impl PlayedMove {
    pub fn new(
        mov: Move,
        capture: Option<PieceTypeV>,
        en_passant_capture: bool,
        flags: BoardFlags,
    ) -> Self {
        Self {
            mov,
            capture,
            en_passant_capture,
            flags,
        }
    }

    pub fn mov(self) -> Move {
        self.mov
    }

    pub fn capture(self) -> Option<PieceTypeV> {
        self.capture
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn can_create_a_move_from_piece_and_two_squares() {
        let mov = Move::new(Square::A2, Square::A3);
        assert_eq!(mov.from(), Square::A2);
        assert_eq!(mov.to(), Square::A3);
        assert_eq!(mov.promoting(), None);
    }

    #[test]
    fn can_create_a_promoting_move() {
        let mov = Move::new_promoting(Square::A2, Square::A3, PieceTypeV::Knight);
        assert_eq!(mov.from(), Square::A2);
        assert_eq!(mov.to(), Square::A3);
        assert_eq!(mov.promoting(), Some(PieceTypeV::Knight));
    }

    #[test]
    fn displaying_a_move_returns_move_notation() {
        let mov = Move::new(Square::A2, Square::A3);
        assert_eq!("a2a3", mov.to_string());
    }

    #[test]
    fn move_to_string_is_inverse_of_parse() {
        let mov = Move::new(Square::B1, Square::C3);

        assert_eq!(mov.to_string().parse::<Move>().unwrap(), mov);
        assert_eq!("a7a8Q".parse::<Move>().unwrap().to_string(), "a7a8Q");
    }

    #[test]
    fn move_parse_on_non_move_string_is_none() {
        assert!("PA8".parse::<Move>().is_err())
    }
}
