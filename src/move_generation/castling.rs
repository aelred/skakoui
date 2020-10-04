use crate::{Bitboard, Board, BoardFlags, Move, PlayerType};
use std::marker::PhantomData;

pub struct CastlingIter<P> {
    checked_kingside: bool,
    checked_queenside: bool,
    occupancy: Bitboard,
    flags: BoardFlags,
    _phantom: PhantomData<P>,
}

impl<P> CastlingIter<P> {
    pub fn new(board: &Board) -> Self {
        Self {
            checked_kingside: false,
            checked_queenside: false,
            occupancy: board.occupancy(),
            flags: board.flags(),
            _phantom: PhantomData::default(),
        }
    }
}

impl<P: PlayerType> Iterator for CastlingIter<P> {
    type Item = Move;

    fn next(&mut self) -> Option<Move> {
        if !self.checked_kingside {
            self.checked_kingside = true;
            if self.flags.is_set(P::CASTLE_KINGSIDE_FLAG)
                && (P::CASTLE_KINGSIDE_CLEAR & self.occupancy).is_empty()
            {
                return Some(Move::castle_kingside(P::PLAYER));
            }
        }

        if !self.checked_queenside {
            self.checked_queenside = true;
            if self.flags.is_set(P::CASTLE_QUEENSIDE_FLAG)
                && (P::CASTLE_QUEENSIDE_CLEAR & self.occupancy).is_empty()
            {
                return Some(Move::castle_queenside(P::PLAYER));
            }
        }

        None
    }
}
