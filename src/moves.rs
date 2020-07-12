use crate::PieceType;
use crate::PlayerType;
use crate::Square;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Move {
    piece_type: PieceType,
    from: Square,
    to: Square,
    promoting: Option<PieceType>,
}

impl Move {
    pub fn new(piece_type: PieceType, from: Square, to: Square) -> Self {
        Self {
            piece_type,
            from,
            to,
            promoting: None,
        }
    }

    pub fn new_promoting(
        piece_type: PieceType,
        from: Square,
        to: Square,
        promoting: PieceType,
    ) -> Self {
        Self {
            piece_type,
            from,
            to,
            promoting: Some(promoting),
        }
    }

    pub fn piece_type(self) -> PieceType {
        self.piece_type
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

    pub fn with_valid_promotions<P: PlayerType>(self) -> impl IntoIterator<Item = Move> {
        if self.to.rank() == P::LAST_RANK {
            vec![
                Move {
                    promoting: Some(PieceType::Queen),
                    ..self
                },
                Move {
                    promoting: Some(PieceType::Rook),
                    ..self
                },
                Move {
                    promoting: Some(PieceType::Bishop),
                    ..self
                },
                Move {
                    promoting: Some(PieceType::Knight),
                    ..self
                },
            ]
        } else {
            vec![self]
        }
    }
}

impl fmt::Display for Move {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        if let Some(promoting) = self.promoting {
            f.write_fmt(format_args!(
                "{}{}{}{}",
                self.piece_type, self.from, self.to, promoting
            ))
        } else {
            f.write_fmt(format_args!("{}{}{}", self.piece_type, self.from, self.to))
        }
    }
}

impl FromStr for Move {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Box<dyn Error>> {
        let piece_type = s
            .get(..1)
            .ok_or("couldn't index string")?
            .parse::<PieceType>()?;
        let from = s
            .get(1..3)
            .ok_or("couldn't index string")?
            .parse::<Square>()?;
        let to = s
            .get(3..5)
            .ok_or("couldn't index string")?
            .parse::<Square>()?;

        let promoting = if let Some(promoting_str) = s.get(5..) {
            if promoting_str.is_empty() {
                None
            } else {
                Some(promoting_str.parse::<PieceType>()?)
            }
        } else {
            None
        };

        Ok(Move {
            piece_type,
            from,
            to,
            promoting,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn can_create_a_move_from_piece_and_two_squares() {
        let mov = Move::new(PieceType::Pawn, Square::A2, Square::A3);
        assert_eq!(mov.piece_type(), PieceType::Pawn);
        assert_eq!(mov.from(), Square::A2);
        assert_eq!(mov.to(), Square::A3);
        assert_eq!(mov.promoting(), None);
    }

    #[test]
    fn can_create_a_promoting_move() {
        let mov = Move::new_promoting(PieceType::Pawn, Square::A2, Square::A3, PieceType::Knight);
        assert_eq!(mov.piece_type(), PieceType::Pawn);
        assert_eq!(mov.from(), Square::A2);
        assert_eq!(mov.to(), Square::A3);
        assert_eq!(mov.promoting(), Some(PieceType::Knight));
    }

    #[test]
    fn displaying_a_move_returns_move_notation() {
        let mov = Move::new(PieceType::Pawn, Square::A2, Square::A3);
        assert_eq!("Pa2a3", mov.to_string());
    }

    #[test]
    fn move_to_string_is_inverse_of_parse() {
        let mov = Move::new(PieceType::Knight, Square::B1, Square::C3);

        assert_eq!(mov.to_string().parse::<Move>().unwrap(), mov);
        assert_eq!("Pa7a8Q".parse::<Move>().unwrap().to_string(), "Pa7a8Q");
    }

    #[test]
    fn move_parse_on_non_move_string_is_none() {
        assert!("PA8".parse::<Move>().is_err())
    }
}
