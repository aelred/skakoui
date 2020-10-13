use crate::move_generation::piece_type::{
    slide, AntiDiagonal, Diagonal, EastWest, Movable, MovesIter, NorthSouth, PieceT, PieceTypeT,
};
use crate::{Bitboard, Board, PieceType, PlayerT, Square};

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

impl<P: PlayerT> Movable for PieceT<P, QueenType> {
    type Moves = MovesIter<P, QueenType>;
    fn moves(self, board: &Board, mask: Bitboard) -> Self::Moves {
        MovesIter::new(board, self, mask)
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_moves;
    use crate::board::tests::fen;

    #[test]
    fn queen_can_move_and_capture_in_all_directions() {
        let mut board = fen("8/8/1p6/2p5/2P5/1QP5/8/3b4 w");
        assert_moves!(
            board,
            [b3d1, b3a2, b3c2, b3a4, b3a3, b3b1, b3b2, b3b4, b3b5, b3b6,]
        );
    }
}
