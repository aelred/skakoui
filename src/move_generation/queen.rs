use crate::move_generation::piece_type::{
    slide, AntiDiagonal, Diagonal, EastWest, MovesIter, NorthSouth, PieceT, PieceTypeT,
};
use crate::{Bitboard, Board, PieceType, PlayerT, Square};

#[derive(Default)]
pub struct QueenType;
impl PieceTypeT for QueenType {
    const PIECE_TYPE: PieceType = PieceType::Queen;

    fn attacks(&self, source: Square, occupancy: Bitboard, _: impl PlayerT) -> Bitboard {
        slide(NorthSouth, source, occupancy)
            | slide(EastWest, source, occupancy)
            | slide(Diagonal, source, occupancy)
            | slide(AntiDiagonal, source, occupancy)
    }
}

pub fn moves<P: PlayerT>(_: P, board: &Board, mask: Bitboard) -> MovesIter<P, QueenType> {
    MovesIter::new(board, PieceT::default(), mask)
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
