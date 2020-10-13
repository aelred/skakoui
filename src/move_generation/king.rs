use crate::move_generation::piece_type::{Movable, MovesIter, PieceT, PieceTypeT};
use crate::{Bitboard, Board, BoardFlags, File, PieceType, PlayerT, Square, SquareMap};
use lazy_static::lazy_static;

pub struct KingType;
impl PieceTypeT for KingType {
    const PIECE_TYPE: PieceType = PieceType::King;

    fn movement(
        &self,
        source: Square,
        occupancy: Bitboard,
        player: impl PlayerT,
        flags: BoardFlags,
    ) -> Bitboard {
        let mut movement = self.attacks(source, occupancy);

        if flags.is_set(player.castle_kingside_flag())
            && (player.castle_kingside_clear() & occupancy).is_empty()
        {
            movement.set(Square::new(File::G, player.back_rank()));
        }

        if flags.is_set(player.castle_queenside_flag())
            && (player.castle_queenside_clear() & occupancy).is_empty()
        {
            movement.set(Square::new(File::C, player.back_rank()));
        }

        movement
    }

    fn attacks(&self, source: Square, _: Bitboard) -> Bitboard {
        KING_MOVES[source]
    }
}

impl<P: PlayerT> Movable for PieceT<P, KingType> {
    type Moves = MovesIter<P, KingType>;
    fn moves(self, board: &Board, mask: Bitboard) -> Self::Moves {
        MovesIter::new(board, self, mask)
    }
}

lazy_static! {
    static ref KING_MOVES: SquareMap<Bitboard> = SquareMap::from(|square| {
        let king = Bitboard::from(square);
        let attacks = king.shift_rank(1) | king.shift_rank_neg(1);
        let ranks = king | attacks;
        attacks | ranks.shift_file(1) | ranks.shift_file_neg(1)
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
}
