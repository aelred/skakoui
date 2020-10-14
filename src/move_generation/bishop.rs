use crate::move_generation::piece_type::{
    slide, AntiDiagonal, Diagonal, MovesIter, PieceT, PieceTypeT,
};
use crate::{Bitboard, Board, PieceType, PlayerT, Square};

#[derive(Default)]
pub struct BishopType;
impl PieceTypeT for BishopType {
    const PIECE_TYPE: PieceType = PieceType::Bishop;

    fn attacks(&self, source: Square, occupancy: Bitboard, _: impl PlayerT) -> Bitboard {
        slide(Diagonal, source, occupancy) | slide(AntiDiagonal, source, occupancy)
    }
}

pub fn moves<P: PlayerT>(_: P, board: &Board, mask: Bitboard) -> MovesIter<P, BishopType> {
    MovesIter::new(board, PieceT::default(), mask)
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
