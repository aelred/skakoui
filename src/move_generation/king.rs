use crate::move_generation::piece_type::{Movable, MovesIter, PieceT, PieceTypeT};
use crate::{bitboards, Bitboard, Board, BoardFlags, File, PieceType, PlayerType, Square};

#[derive(Copy, Clone)]
pub struct KingType;
impl PieceTypeT for KingType {
    const PIECE_TYPE: PieceType = PieceType::King;

    fn movement(
        self,
        source: Square,
        occupancy: Bitboard,
        player: impl PlayerType,
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

    fn attacks(self, source: Square, _: Bitboard) -> Bitboard {
        bitboards::KING_MOVES[source]
    }
}

impl<P: PlayerType> Movable for PieceT<P, KingType> {
    type Moves = MovesIter<P, KingType>;
    fn moves(self, board: &Board, mask: Bitboard) -> Self::Moves {
        MovesIter::new(board, self, mask)
    }
}
