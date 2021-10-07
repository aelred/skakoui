use crate::move_generation::piece_type::{PieceType, PieceTypeT};
use crate::{
    bitboards,
    move_generation::piece_type::{MovesIter, PieceT},
    move_generation::{AllMoves, CapturingMoves},
    Bitboard, Board, BoardFlags, File, PieceTypeV, Player, Square, SquareMap,
};
use lazy_static::lazy_static;

#[derive(Default, Copy, Clone)]
pub struct King;
impl PieceType for King {
    fn value(self) -> PieceTypeV {
        PieceTypeV::King
    }

    fn attacks(&self, source: Square, _: Bitboard, _: impl Player, _: BoardFlags) -> Bitboard {
        KING_MOVES[source]
    }

    fn other_moves(
        &self,
        _: Square,
        occupancy: Bitboard,
        player: impl Player,
        flags: BoardFlags,
    ) -> Bitboard {
        let mut castles = bitboards::EMPTY;

        if flags.is_set(player.castle_kingside_flag())
            && (player.castle_kingside_clear() & occupancy) == bitboards::EMPTY
        {
            castles.set(Square::new(File::KINGSIDE, player.back_rank()));
        }

        if flags.is_set(player.castle_queenside_flag())
            && (player.castle_queenside_clear() & occupancy) == bitboards::EMPTY
        {
            castles.set(Square::new(File::QUEENSIDE, player.back_rank()));
        }

        castles
    }
}

impl PieceTypeT for King {}

pub type Moves<P> = MovesIter<P, King, AllMoves<P>>;
pub type Attacks<P> = MovesIter<P, King, CapturingMoves<P>>;

pub fn moves<P: Player>(player: P, board: &Board, mask: Bitboard) -> Moves<P> {
    MovesIter::new(board, PieceT::new(player, King), AllMoves(player), mask)
}

pub fn attacks<P: Player>(player: P, board: &Board, mask: Bitboard) -> Attacks<P> {
    MovesIter::new(
        board,
        PieceT::new(player, King),
        CapturingMoves(player),
        mask,
    )
}

lazy_static! {
    static ref KING_MOVES: SquareMap<Bitboard> = SquareMap::from(|square| {
        let king = Bitboard::from(square);
        let attacks = king.shift_rank(1) | king.shift_rank(-1);
        let ranks = king | attacks;
        attacks | ranks.shift_file(1) | ranks.shift_file(-1)
    });
}

#[cfg(test)]
mod tests {
    use crate::assert_moves;
    use crate::board::tests::fen;

    #[test]
    fn king_can_move_and_capture_one_square_in_any_direction() {
        let mut board = fen("8/8/8/8/8/1Kp5/2P5/8 w");
        // b3b2 is missing because it puts the king in check
        assert_moves!(board, [b3a2, b3a3, b3c3, b3a4, b3b4, b3c4,]);
    }

    #[test]
    fn king_can_castle() {
        let mut board = fen("8/8/8/8/8/8/r6r/R3K2R w");
        assert_moves!(
            board,
            [e1c1, e1g1, e1d1, e1f1, a1b1, a1c1, a1d1, a1a2, h1g1, h1f1, h1h2]
        );
        let mut board = fen("r3k2r/R6R/8/8/8/8/8/8 b");
        assert_moves!(
            board,
            [e8c8, e8g8, e8d8, e8f8, a8b8, a8c8, a8d8, a8a7, h8g8, h8f8, h8h7]
        );
    }

    #[test]
    fn king_cannot_castle_out_of_check() {
        let mut board = fen("8/8/8/8/8/8/r2q3r/R3K2R w");
        assert_moves!(board, [e1f1]);
        let mut board = fen("r3k2r/R2Q3R/8/8/8/8/8/8 b");
        assert_moves!(board, [e8f8]);
    }

    #[test]
    fn king_cannot_castle_into_check() {
        let mut board = fen("8/8/8/8/8/8/r6p/R3K2R w");
        assert_moves!(
            board,
            [e1c1, e1d1, e1f1, a1b1, a1c1, a1d1, a1a2, h1g1, h1f1, h1h2]
        );
        let mut board = fen("r3k2r/RP5R/8/8/8/8/8/8 b");
        assert_moves!(
            board,
            [e8g8, e8d8, e8f8, a8b8, a8c8, a8d8, a8a7, h8g8, h8f8, h8h7]
        );
    }

    #[test]
    fn king_cannot_castle_through_check() {
        let mut board = fen("8/8/8/8/8/8/r5pr/R3K2R w");
        assert_moves!(
            board,
            [e1c1, e1d1, a1b1, a1c1, a1d1, a1a2, h1g1, h1f1, h1h2]
        );
        let mut board = fen("r3k2r/R1P4R/8/8/8/8/8/8 b");
        assert_moves!(
            board,
            [e8g8, e8f8, a8b8, a8c8, a8d8, a8a7, h8g8, h8f8, h8h7]
        );
    }

    #[test]
    fn rook_can_castle_through_an_attacked_square() {
        let mut board = fen("8/8/8/8/8/8/p6r/R3K2R w");
        assert_moves!(
            board,
            [e1c1, e1g1, e1d1, e1f1, a1b1, a1c1, a1d1, a1a2, h1g1, h1f1, h1h2]
        );
        let mut board = fen("r3k2r/P6R/8/8/8/8/8/8 b");
        assert_moves!(
            board,
            [e8c8, e8g8, e8d8, e8f8, a8b8, a8c8, a8d8, a8a7, h8g8, h8f8, h8h7]
        );
    }
}
