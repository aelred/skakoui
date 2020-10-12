use crate::move_generation::piece_type::{Movable, MovesIter, PieceT, PieceTypeT};
use crate::{Bitboard, Board, BoardFlags, File, PieceType, PlayerType, Square, SquareMap};
use lazy_static::lazy_static;

pub struct KingType;
impl PieceTypeT for KingType {
    const PIECE_TYPE: PieceType = PieceType::King;

    fn movement(
        &self,
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

    fn attacks(&self, source: Square, _: Bitboard) -> Bitboard {
        KING_MOVES[source]
    }
}

impl<P: PlayerType> Movable for PieceT<P, KingType> {
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
