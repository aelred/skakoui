use crate::move_generation::piece_type::{
    slide, EastWest, Movable, MovesIter, NorthSouth, PieceT, PieceTypeT,
};
use crate::{Bitboard, Board, PieceType, PlayerType, Square};

#[derive(Copy, Clone)]
pub struct RookType;
impl PieceTypeT for RookType {
    const PIECE_TYPE: PieceType = PieceType::Rook;

    fn attacks(self, source: Square, occupancy: Bitboard) -> Bitboard {
        slide(NorthSouth, source, occupancy) | slide(EastWest, source, occupancy)
    }
}

impl<P: PlayerType> Movable for PieceT<P, RookType> {
    type Moves = MovesIter<P, RookType>;
    fn moves(self, board: &Board, mask: Bitboard) -> Self::Moves {
        MovesIter::new(board, self, mask)
    }
}
