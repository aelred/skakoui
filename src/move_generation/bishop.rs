use crate::move_generation::piece_type::SlideDirection;
use crate::{
    move_generation::piece_type::{AntiDiagonal, Diagonal, MovesIter, PieceT, PieceTypeT},
    move_generation::{AllMoves, CapturingMoves},
    Bitboard, Board, BoardFlags, PieceType, Player, Square,
};

#[derive(Default)]
pub struct Bishop;

impl PieceTypeT for Bishop {
    const PIECE_TYPE: PieceType = PieceType::Bishop;

    fn attacks(
        &self,
        source: Square,
        occupancy: Bitboard,
        _: impl Player,
        _: BoardFlags,
    ) -> Bitboard {
        Diagonal.slide(source, occupancy) | AntiDiagonal.slide(source, occupancy)
    }
}

pub type Moves<P> = MovesIter<P, Bishop, AllMoves<P>>;
pub type Attacks<P> = MovesIter<P, Bishop, CapturingMoves<P>>;

pub fn moves<P: Player>(player: P, board: &Board, mask: Bitboard) -> Moves<P> {
    MovesIter::new(board, PieceT::new(player, Bishop), AllMoves(player), mask)
}

pub fn attacks<P: Player>(player: P, board: &Board, mask: Bitboard) -> Attacks<P> {
    MovesIter::new(
        board,
        PieceT::new(player, Bishop),
        CapturingMoves(player),
        mask,
    )
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
