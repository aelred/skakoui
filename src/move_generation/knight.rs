use crate::move_generation::piece_type::{MovesIter, PieceT, PieceTypeT};
use crate::move_generation::{AllMoves, CapturingMoves};
use crate::{Bitboard, Board, PieceType, PlayerT, Square, SquareMap};
use lazy_static::lazy_static;

#[derive(Default)]
pub struct Knight;
impl PieceTypeT for Knight {
    const PIECE_TYPE: PieceType = PieceType::Knight;

    fn attacks(&self, source: Square, _: Bitboard, _: impl PlayerT) -> Bitboard {
        KNIGHT_MOVES[source]
    }
}

pub type Moves<P> = MovesIter<P, Knight, AllMoves<P>>;
pub type Attacks<P> = MovesIter<P, Knight, CapturingMoves<P>>;

pub fn moves<P: PlayerT>(player: P, board: &Board, mask: Bitboard) -> Moves<P> {
    MovesIter::new(board, PieceT::default(), AllMoves(player), mask)
}

pub fn attacks<P: PlayerT>(player: P, board: &Board, mask: Bitboard) -> Attacks<P> {
    MovesIter::new(board, PieceT::default(), CapturingMoves(player), mask)
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
