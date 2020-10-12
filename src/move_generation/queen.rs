use crate::move_generation::piece_type::{
    slide, AntiDiagonal, Diagonal, EastWest, Movable, MovesIter, NorthSouth, PieceT, PieceTypeT,
};
use crate::{Bitboard, Board, PieceType, PlayerType, Square};

pub struct QueenType;
impl PieceTypeT for QueenType {
    const PIECE_TYPE: PieceType = PieceType::Queen;

    fn attacks(&self, source: Square, occupancy: Bitboard) -> Bitboard {
        slide(NorthSouth, source, occupancy)
            | slide(EastWest, source, occupancy)
            | slide(Diagonal, source, occupancy)
            | slide(AntiDiagonal, source, occupancy)
    }
}

impl<P: PlayerType> Movable for PieceT<P, QueenType> {
    type Moves = MovesIter<P, QueenType>;
    fn moves(self, board: &Board, mask: Bitboard) -> Self::Moves {
        MovesIter::new(board, self, mask)
    }
}
