use crate::magic::Magic;
use crate::move_generation::piece_type::PieceType;
use crate::piece::Piece;
use crate::{
    move_generation::piece_type::{MovesIter, PieceTypeT},
    move_generation::{AllMoves, CapturingMoves},
    Bitboard, Board, BoardFlags, PieceTypeV, Player, Square,
};

#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Bishop;

impl PieceType for Bishop {
    fn value(self) -> PieceTypeV {
        PieceTypeV::Bishop
    }

    fn attacks(
        &self,
        source: Square,
        occupancy: Bitboard,
        _: impl Player,
        _: BoardFlags,
    ) -> Bitboard {
        Bishop.magic_moves(source, occupancy)
    }
}

impl PieceTypeT for Bishop {}

pub type Moves<P> = MovesIter<P, Bishop, AllMoves<P>>;
pub type Attacks<P> = MovesIter<P, Bishop, CapturingMoves<P>>;

pub fn moves<P: Player>(player: P, board: &Board, mask: Bitboard) -> Moves<P> {
    MovesIter::new(board, Piece::new(player, Bishop), AllMoves(player), mask)
}

pub fn attacks<P: Player>(player: P, board: &Board, mask: Bitboard) -> Attacks<P> {
    MovesIter::new(
        board,
        Piece::new(player, Bishop),
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
