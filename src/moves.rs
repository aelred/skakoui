use crate::{BoardFlags, File, PieceType, Player, Square};
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Move {
    from: Square,
    to: Square,
    promoting: Option<PieceType>,
}

impl Move {
    pub fn new(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            promoting: None,
        }
    }

    pub fn new_promoting(from: Square, to: Square, promoting: PieceType) -> Self {
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

    pub fn promoting(self) -> Option<PieceType> {
        self.promoting
    }

    pub fn mut_promoting(&mut self) -> &mut Option<PieceType> {
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
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Box<dyn Error>> {
        let from = s
            .get(0..2)
            .ok_or("couldn't index string")?
            .parse::<Square>()?;
        let to = s
            .get(2..4)
            .ok_or("couldn't index string")?
            .parse::<Square>()?;

        let promoting = if let Some(promoting_str) = s.get(4..) {
            if promoting_str.is_empty() {
                None
            } else {
                Some(promoting_str.parse::<PieceType>()?)
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
    pub capture: Option<PieceType>,
    pub flags: BoardFlags,
}

impl PlayedMove {
    pub fn new(mov: Move, capture: Option<PieceType>, flags: BoardFlags) -> Self {
        Self {
            mov,
            capture,
            flags,
        }
    }

    pub fn mov(&self) -> Move {
        self.mov
    }

    pub fn capture(&self) -> Option<PieceType> {
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
        let mov = Move::new_promoting(Square::A2, Square::A3, PieceType::Knight);
        assert_eq!(mov.from(), Square::A2);
        assert_eq!(mov.to(), Square::A3);
        assert_eq!(mov.promoting(), Some(PieceType::Knight));
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
