use crate::move_generation::piece_type::{
    slide, EastWest, MovesIter, NorthSouth, PieceT, PieceTypeT,
};
use crate::{Bitboard, Board, PieceType, PlayerT, Square};

#[derive(Default)]
pub struct RookType;
impl PieceTypeT for RookType {
    const PIECE_TYPE: PieceType = PieceType::Rook;

    fn attacks(&self, source: Square, occupancy: Bitboard, _: impl PlayerT) -> Bitboard {
        slide(NorthSouth, source, occupancy) | slide(EastWest, source, occupancy)
    }
}

pub fn moves<P: PlayerT>(_: P, board: &Board, mask: Bitboard) -> MovesIter<P, RookType> {
    MovesIter::new(board, PieceT::default(), mask)
}

#[cfg(test)]
mod tests {
    use crate::assert_moves;
    use crate::board::tests::fen;

    #[test]
    fn rook_can_move_and_capture_along_rank_and_file() {
        let mut board = fen("8/8/1p6/1P6/8/1Rq5/8/8 w");
        assert_moves!(board, [b3b1, b3b2, b3a3, b3c3, b3b4,]);
    }
}
