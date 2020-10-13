use crate::move_generation::piece_type::{
    slide, AntiDiagonal, Diagonal, Movable, MovesIter, PieceT, PieceTypeT,
};
use crate::{Bitboard, Board, PieceType, PlayerT, Square};

pub struct BishopType;
impl PieceTypeT for BishopType {
    const PIECE_TYPE: PieceType = PieceType::Bishop;

    fn attacks(&self, source: Square, occupancy: Bitboard) -> Bitboard {
        slide(Diagonal, source, occupancy) | slide(AntiDiagonal, source, occupancy)
    }
}

impl<P: PlayerT> Movable for PieceT<P, BishopType> {
    type Moves = MovesIter<P, BishopType>;
    fn moves(self, board: &Board, mask: Bitboard) -> Self::Moves {
        MovesIter::new(board, self, mask)
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_moves;
    use crate::board::tests::fen;

    #[test]
    fn bishop_can_move_and_capture_diagonally() {
        let mut board = fen("8/8/8/2p5/2P5/1B6/8/3b4 w");
        assert_moves!(board, [b3d1, b3a2, b3c2, b3a4,]);
    }
}
