use crate::move_generation::piece_type::{Movable, MovesIter, PieceT, PieceTypeT};
use crate::{Bitboard, Board, PieceType, PlayerType, Square, SquareMap};
use lazy_static::lazy_static;

pub struct KnightType;
impl PieceTypeT for KnightType {
    const PIECE_TYPE: PieceType = PieceType::Knight;

    fn attacks(&self, source: Square, _: Bitboard) -> Bitboard {
        KNIGHT_MOVES[source]
    }
}

impl<P: PlayerType> Movable for PieceT<P, KnightType> {
    type Moves = MovesIter<P, KnightType>;
    fn moves(self, board: &Board, mask: Bitboard) -> Self::Moves {
        MovesIter::new(board, self, mask)
    }
}

lazy_static! {
    static ref KNIGHT_MOVES: SquareMap<Bitboard> = SquareMap::from(|square| {
        let knight = Bitboard::from(square);
        let ranks = knight.shift_rank(2) | knight.shift_rank_neg(2);
        let rank_attacks = ranks.shift_file(1) | ranks.shift_file_neg(1);
        let files = knight.shift_file(2) | knight.shift_file_neg(2);
        let file_attacks = files.shift_rank(1) | files.shift_rank_neg(1);
        rank_attacks | file_attacks
    });
}
