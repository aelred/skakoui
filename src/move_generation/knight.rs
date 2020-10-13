use crate::move_generation::piece_type::{Movable, MovesIter, PieceT, PieceTypeT};
use crate::{Bitboard, Board, PieceType, PlayerT, Square, SquareMap};
use lazy_static::lazy_static;

pub struct KnightType;
impl PieceTypeT for KnightType {
    const PIECE_TYPE: PieceType = PieceType::Knight;

    fn attacks(&self, source: Square, _: Bitboard, _: impl PlayerT) -> Bitboard {
        KNIGHT_MOVES[source]
    }
}

impl<P: PlayerT> Movable for PieceT<P, KnightType> {
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

#[cfg(test)]
mod tests {
    use crate::assert_moves;
    use crate::board::tests::fen;

    #[test]
    fn knight_can_move_and_capture_in_its_weird_way() {
        let mut board = fen("8/8/2p5/2P5/3p4/1N6/8/8 w");
        assert_moves!(board, [b3a1, b3c1, b3d2, b3d4, b3a5,]);
    }
}
