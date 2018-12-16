use crate::Piece;
use crate::Square;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Move {
    piece: Piece,
    from: Square,
    to: Square,
    promoting: Option<Piece>,
}

impl Move {
    pub fn new(piece: Piece, from: Square, to: Square) -> Self {
        Move {
            piece,
            from,
            to,
            promoting: None,
        }
    }

    pub fn new_promoting(piece: Piece, from: Square, to: Square, promoting: Piece) -> Self {
        Move {
            piece,
            from,
            to,
            promoting: Some(promoting),
        }
    }

    pub fn piece(self) -> Piece {
        self.piece
    }

    pub fn from(self) -> Square {
        self.from
    }

    pub fn to(self) -> Square {
        self.to
    }

    pub fn promoting(self) -> Option<Piece> {
        self.promoting
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_a_move_from_piece_and_two_squares() {
        let mov = Move::new(Piece::WP, Square::A2, Square::A3);
        assert_eq!(mov.piece(), Piece::WP);
        assert_eq!(mov.from(), Square::A2);
        assert_eq!(mov.to(), Square::A3);
        assert_eq!(mov.promoting(), None);
    }

    #[test]
    fn can_create_a_promoting_move() {
        let mov = Move::new_promoting(Piece::WP, Square::A2, Square::A3, Piece::WN);
        assert_eq!(mov.piece(), Piece::WP);
        assert_eq!(mov.from(), Square::A2);
        assert_eq!(mov.to(), Square::A3);
        assert_eq!(mov.promoting(), Some(Piece::WN));
    }
}
