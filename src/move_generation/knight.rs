use lazy_static::lazy_static;

use crate::move_generation::piece_type::{PieceType, PieceTypeT};
use crate::piece::Piece;
use crate::{
    move_generation::piece_type::MovesIter,
    move_generation::{AllMoves, CapturingMoves},
    Bitboard, Board, BoardFlags, PieceTypeV, Player, Square, SquareMap,
};

#[derive(Default, Copy, Clone)]
pub struct Knight;
impl PieceType for Knight {
    fn value(self) -> PieceTypeV {
        PieceTypeV::Knight
    }

    fn attacks(&self, source: Square, _: Bitboard, _: impl Player, _: BoardFlags) -> Bitboard {
        KNIGHT_MOVES[source]
    }
}

impl PieceTypeT for Knight {}

pub type Moves<P> = MovesIter<P, Knight, AllMoves<P>>;
pub type Attacks<P> = MovesIter<P, Knight, CapturingMoves<P>>;

pub fn moves<P: Player>(player: P, board: &Board, mask: Bitboard) -> Moves<P> {
    MovesIter::new(board, Piece::new(player, Knight), AllMoves(player), mask)
}

pub fn attacks<P: Player>(player: P, board: &Board, mask: Bitboard) -> Attacks<P> {
    MovesIter::new(
        board,
        Piece::new(player, Knight),
        CapturingMoves(player),
        mask,
    )
}

lazy_static! {
    static ref KNIGHT_MOVES: SquareMap<Bitboard> = SquareMap::from(|square| {
        let knight = Bitboard::from(square);
        let ranks = knight.shift_rank(2) | knight.shift_rank(-2);
        let rank_attacks = ranks.shift_file(1) | ranks.shift_file(-1);
        let files = knight.shift_file(2) | knight.shift_file(-2);
        let file_attacks = files.shift_rank(1) | files.shift_rank(-1);
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
