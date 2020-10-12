use crate::move_generation::piece_type::{
    slide, AntiDiagonal, Diagonal, Movable, MovesIter, PieceT, PieceTypeT,
};
use crate::{Bitboard, Board, PieceType, PlayerType, Square};

pub struct BishopType;
impl PieceTypeT for BishopType {
    const PIECE_TYPE: PieceType = PieceType::Bishop;

    fn attacks(&self, source: Square, occupancy: Bitboard) -> Bitboard {
        slide(Diagonal, source, occupancy) | slide(AntiDiagonal, source, occupancy)
    }
}

impl<P: PlayerType> Movable for PieceT<P, BishopType> {
    type Moves = MovesIter<P, BishopType>;
    fn moves(self, board: &Board, mask: Bitboard) -> Self::Moves {
        MovesIter::new(board, self, mask)
    }
}
