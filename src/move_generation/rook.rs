use crate::move_generation::piece_type::SlideDirection;
use crate::{
    move_generation::piece_type::{EastWest, MovesIter, NorthSouth, PieceT, PieceTypeT},
    move_generation::{AllMoves, CapturingMoves},
    Bitboard, Board, PieceType, Player, Square,
};

#[derive(Default)]
pub struct Rook;
impl PieceTypeT for Rook {
    const PIECE_TYPE: PieceType = PieceType::Rook;

    fn attacks(&self, source: Square, occupancy: Bitboard, _: impl Player) -> Bitboard {
        NorthSouth.slide(source, occupancy) | EastWest.slide(source, occupancy)
    }
}

pub type Moves<P> = MovesIter<P, Rook, AllMoves<P>>;
pub type Attacks<P> = MovesIter<P, Rook, CapturingMoves<P>>;

pub fn moves<P: Player>(player: P, board: &Board, mask: Bitboard) -> Moves<P> {
    MovesIter::new(board, PieceT::new(player, Rook), AllMoves(player), mask)
}

pub fn attacks<P: Player>(player: P, board: &Board, mask: Bitboard) -> Attacks<P> {
    MovesIter::new(
        board,
        PieceT::new(player, Rook),
        CapturingMoves(player),
        mask,
    )
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
