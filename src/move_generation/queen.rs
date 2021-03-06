use crate::{
    move_generation::piece_type::{MovesIter, PieceT, PieceTypeT},
    move_generation::{AllMoves, CapturingMoves},
    Bishop, Bitboard, Board, BoardFlags, PieceType, Player, Rook, Square,
};

#[derive(Default)]
pub struct Queen;
impl PieceTypeT for Queen {
    const PIECE_TYPE: PieceType = PieceType::Queen;

    fn attacks(
        &self,
        source: Square,
        occ: Bitboard,
        player: impl Player,
        flags: BoardFlags,
    ) -> Bitboard {
        Rook.attacks(source, occ, player, flags) | Bishop.attacks(source, occ, player, flags)
    }
}

pub type Moves<P> = MovesIter<P, Queen, AllMoves<P>>;
pub type Attacks<P> = MovesIter<P, Queen, CapturingMoves<P>>;

pub fn moves<P: Player>(player: P, board: &Board, mask: Bitboard) -> Moves<P> {
    MovesIter::new(board, PieceT::new(player, Queen), AllMoves(player), mask)
}

pub fn attacks<P: Player>(player: P, board: &Board, mask: Bitboard) -> Attacks<P> {
    MovesIter::new(
        board,
        PieceT::new(player, Queen),
        CapturingMoves(player),
        mask,
    )
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
