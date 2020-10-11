use crate::move_generation::piece_type::{Movable, MovesIter, PieceT, PieceTypeT};
use crate::{bitboards, Bitboard, Board, PieceType, PlayerType, Square};

#[derive(Copy, Clone)]
pub struct KnightType;
impl PieceTypeT for KnightType {
    const PIECE_TYPE: PieceType = PieceType::Knight;

    fn attacks(self, source: Square, _: Bitboard) -> Bitboard {
        bitboards::KNIGHT_MOVES[source]
    }
}

impl<P: PlayerType> Movable for PieceT<P, KnightType> {
    type Moves = MovesIter<P, KnightType>;
    fn moves(self, board: &Board, mask: Bitboard) -> Self::Moves {
        MovesIter::new(board, self, mask)
    }
}
