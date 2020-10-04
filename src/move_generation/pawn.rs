use crate::bitboard::SquareIterator;
use crate::{bitboards, Board, Move, Piece, PieceType, PlayerType};
use std::marker::PhantomData;

pub struct PawnMovesIter<P> {
    pushes: SquareIterator,
    double_pushes: SquareIterator,
    captures: PawnCapturesIter<P>,
    _phantom: PhantomData<P>,
}

impl<P: PlayerType> PawnMovesIter<P> {
    pub fn new(board: &Board) -> Self {
        let piece = Piece::new(P::PLAYER, PieceType::Pawn);

        let pawns = board.bitboard_piece(piece);
        let free_spaces = !board.occupancy();

        let pawns_forward = P::advance_bitboard(*pawns);

        let pushes = pawns_forward & free_spaces;

        let double_mask = bitboards::RANKS[P::PAWN_RANK + P::DIRECTION];
        let double_pushes = P::advance_bitboard(pushes & double_mask) & free_spaces;

        PawnMovesIter {
            pushes: pushes.squares(),
            double_pushes: double_pushes.squares(),
            captures: PawnCapturesIter::new(board),
            _phantom: PhantomData,
        }
    }
}

impl<P: PlayerType> Iterator for PawnMovesIter<P> {
    type Item = Move;

    fn next(&mut self) -> Option<Move> {
        if let Some(target) = self.pushes.next() {
            let source = target.shift_rank(-P::DIRECTION);
            return Some(Move::new(source, target));
        }

        if let Some(target) = self.double_pushes.next() {
            let source = target.shift_rank(-P::DIRECTION * 2);
            return Some(Move::new(source, target));
        }

        self.captures.next()
    }
}

pub struct PawnCapturesIter<P> {
    captures_east: SquareIterator,
    captures_west: SquareIterator,
    _phantom: PhantomData<P>,
}

impl<P: PlayerType> PawnCapturesIter<P> {
    pub fn new(board: &Board) -> Self {
        let piece = Piece::new(P::PLAYER, PieceType::Pawn);
        let pawns = board.bitboard_piece(piece);
        let pawns_forward = P::advance_bitboard(*pawns);

        let opponent_pieces = board.occupancy_player(P::Opp::PLAYER);

        let captures_east = pawns_forward.shift_file_neg(1) & opponent_pieces;
        let captures_west = pawns_forward.shift_file(1) & opponent_pieces;

        PawnCapturesIter {
            captures_east: captures_east.squares(),
            captures_west: captures_west.squares(),
            _phantom: PhantomData,
        }
    }
}

impl<P: PlayerType> Iterator for PawnCapturesIter<P> {
    type Item = Move;

    fn next(&mut self) -> Option<Move> {
        if let Some(target) = self.captures_east.next() {
            let source = target.shift_rank(-P::DIRECTION).shift_file(1);
            return Some(Move::new(source, target));
        }

        if let Some(target) = self.captures_west.next() {
            let source = target.shift_rank(-P::DIRECTION).shift_file(-1);
            return Some(Move::new(source, target));
        }

        None
    }
}
