#![cfg_attr(feature = "strict", deny(warnings))]

mod bitboard;
mod board;
mod fen;
mod file;
mod move_generation;
mod moves;
mod perft;
pub mod pgn;
mod piece;
mod player;
mod rank;
mod search;
mod square;

#[cfg(test)]
pub mod strategies;

pub use crate::{
    bitboard::{bitboards, Bitboard},
    board::{Board, BoardFlags},
    file::File,
    move_generation::{magic, Bishop, King, Knight, Pawn, Queen, Rook},
    moves::{Move, PlayedMove},
    piece::{Piece, PieceType},
    player::{Black, Player, PlayerV, White},
    rank::{Rank, RankMap},
    search::Searcher,
    square::{Square, SquareColor, SquareMap},
};

#[derive(Debug, Clone, Default)]
pub struct GameState {
    pub board: Board,
    moves: Vec<PlayedMove>,
}

impl GameState {
    pub fn new(board: Board) -> Self {
        Self {
            board,
            moves: vec![],
        }
    }

    pub fn push_move(&mut self, mov: Move) {
        let pmov = self.board.make_move(mov);
        self.moves.push(pmov);
    }

    pub fn pop(&mut self) -> Option<Move> {
        self.moves.pop().map(|pmov| {
            self.board.unmake_move(pmov);
            pmov.mov
        })
    }

    pub fn moves(&self) -> impl Iterator<Item = &Move> {
        self.moves.iter().map(|pm| &pm.mov)
    }
}
