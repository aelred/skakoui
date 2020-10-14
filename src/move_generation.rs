use crate::Board;
use crate::Move;
use crate::Piece;
use crate::PieceType;
use crate::Player;
use crate::PlayerT;
use crate::White;
use crate::{Bitboard, Square};
use crate::{Black, BoardFlags};

mod bishop;
mod king;
mod knight;
mod pawn;
mod piece_type;
mod queen;
mod rook;

use crate::move_generation::piece_type::PieceTypeT;
use crate::piece::PieceType::King;
use std::iter::Chain;

impl Board {
    /// Lazy iterator of all legal moves
    pub fn moves<'a>(&'a mut self) -> impl Iterator<Item = Move> + 'a {
        self.pseudo_legal_moves()
            .filter(move |mov| self.check_legal(*mov))
    }

    /// See [pseudo_legal_moves_for]
    pub fn pseudo_legal_moves(&self) -> Box<dyn Iterator<Item = Move>> {
        self.pseudo_legal_moves_for(self.player())
    }

    /// Lazy iterator of all pseudo-legal moves. Pseudo-legal means they ignore:
    /// 1. Check
    /// 2. King captures
    /// 3. Castling through check
    pub fn pseudo_legal_moves_for(&self, player: Player) -> Box<dyn Iterator<Item = Move>> {
        match player {
            Player::White => Box::new(self.moves_of_type(AllMoves(White))),
            Player::Black => Box::new(self.moves_of_type(AllMoves(Black))),
        }
    }

    /// Lazy iterator of all pseudo-legal moves. Pseudo-legal means they ignore:
    /// 1. Check
    /// 2. King captures
    /// 3. Castling through check
    pub fn pseudo_legal_moves_for_typed(&self, player: impl PlayerT) -> impl Iterator<Item = Move> {
        self.moves_of_type(AllMoves(player))
    }

    /// Lazy iterator of all capturing moves
    pub fn capturing_moves(&self, player: impl PlayerT) -> impl Iterator<Item = Move> {
        self.moves_of_type(CapturingMoves(player))
    }

    fn moves_of_type<P: PlayerT, M: Movement<P>>(&self, movement: M) -> impl Iterator<Item = Move> {
        movement.moves(self)
    }

    pub fn check_legal(&mut self, mov: Move) -> bool {
        // TODO: this is a very inefficient way to confirm if in check
        let me = self.player();

        // TODO: disallow castling through check
        let king_move = self.pieces()[mov.from()].map(Piece::piece_type) == Some(PieceType::King);
        let castling = king_move && (mov.from().file() - mov.to().file()).abs() == 2;
        if castling && self.check(me) {
            return false;
        }

        let pmov = self.make_move(mov);
        let in_check = self.check(me);
        let captured_king = pmov.capture() == Some(King);
        self.unmake_move(pmov);
        !(in_check || captured_king)
    }

    pub fn check(&self, king_player: Player) -> bool {
        let king = Piece::new(king_player, PieceType::King);
        if let Some(king_pos) = self.bitboard_piece(king).squares().next() {
            self.pseudo_legal_moves_for(king_player.opponent())
                .any(|mov| mov.to() == king_pos)
        } else {
            false
        }
    }

    pub fn checkmate(&mut self) -> bool {
        let me = self.player();
        for mov in self.pseudo_legal_moves() {
            let pmov = self.make_move(mov);
            let check = self.check(me);
            self.unmake_move(pmov);
            if !check {
                return false;
            }
        }
        true
    }
}

pub trait Movement<P: PlayerT> {
    type Moves: Iterator<Item = Move>;

    fn movement(
        &self,
        piece_type: &impl PieceTypeT,
        source: Square,
        occupancy: Bitboard,
        flags: BoardFlags,
    ) -> Bitboard;

    fn moves(&self, board: &Board) -> Self::Moves;
}

pub struct AllMoves<P>(P);

impl<P: PlayerT> Movement<P> for AllMoves<P> {
    #[allow(clippy::type_complexity)]
    type Moves = Chain6<
        king::Moves<P>,
        queen::Moves<P>,
        rook::Moves<P>,
        bishop::Moves<P>,
        knight::Moves<P>,
        pawn::Moves<P>,
    >;

    fn movement(
        &self,
        piece_type: &impl PieceTypeT,
        source: Square,
        occupancy: Bitboard,
        flags: BoardFlags,
    ) -> Bitboard {
        piece_type.movement(source, occupancy, self.0, flags)
    }

    fn moves(&self, board: &Board) -> Self::Moves {
        let mask = !board.occupancy_player(self.0.value());

        let king = king::moves(self.0, board, mask);
        let queen = queen::moves(self.0, board, mask);
        let rook = rook::moves(self.0, board, mask);
        let bishop = bishop::moves(self.0, board, mask);
        let knight = knight::moves(self.0, board, mask);
        let pawn = pawn::moves(self.0, board, mask);

        king.chain(queen)
            .chain(rook)
            .chain(bishop)
            .chain(knight)
            .chain(pawn)
    }
}

pub struct CapturingMoves<P>(P);

impl<P: PlayerT> Movement<P> for CapturingMoves<P> {
    #[allow(clippy::type_complexity)]
    type Moves = Chain6<
        king::Attacks<P>,
        queen::Attacks<P>,
        rook::Attacks<P>,
        bishop::Attacks<P>,
        knight::Attacks<P>,
        pawn::Attacks<P>,
    >;

    fn movement(
        &self,
        piece_type: &impl PieceTypeT,
        source: Square,
        occupancy: Bitboard,
        _: BoardFlags,
    ) -> Bitboard {
        piece_type.attacks(source, occupancy, self.0)
    }

    fn moves(&self, board: &Board) -> Self::Moves {
        let mask = board.occupancy_player(self.0.opponent().value());

        let king = king::attacks(self.0, board, mask);
        let queen = queen::attacks(self.0, board, mask);
        let rook = rook::attacks(self.0, board, mask);
        let bishop = bishop::attacks(self.0, board, mask);
        let knight = knight::attacks(self.0, board, mask);
        let pawn = pawn::attacks(self.0, board, mask);

        king.chain(queen)
            .chain(rook)
            .chain(bishop)
            .chain(knight)
            .chain(pawn)
    }
}

type Chain6<K, Q, R, B, N, P> = Chain<Chain<Chain<Chain<Chain<K, Q>, R>, B>, N>, P>;

#[cfg(test)]
#[macro_use]
mod tests {
    use super::*;
    use crate::board::tests::fen;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;

    #[macro_export]
    macro_rules! mov {
        ($mov:expr) => {
            stringify!($mov).parse::<$crate::Move>().unwrap()
        };
    }

    #[macro_export]
    macro_rules! assert_moves {
        ($board:expr, [$($moves:expr),* $(,)*]) => {
            let mut moves: Vec<$crate::Move> = $board.moves().collect();
            moves.sort();

            let mut expected_moves: Vec<$crate::Move> = [
                $($crate::mov!($moves)),*
            ].iter().cloned().collect();
            expected_moves.sort();

            assert_eq!(moves, expected_moves, "\n{}", $board);
        };
    }

    #[test]
    fn can_generate_all_possible_starting_moves_for_white() {
        let mut board = Board::default();

        assert_moves!(
            board,
            [
                a2a3, b2b3, c2c3, d2d3, e2e3, f2f3, g2g3, h2h3, a2a4, b2b4, c2c4, d2d4, e2e4, f2f4,
                g2g4, h2h4, b1a3, b1c3, g1h3, g1f3
            ]
        );
    }

    #[test]
    fn can_generate_all_possible_starting_moves_for_black() {
        let mut board = Board::default();
        board.make_move(mov!(a2a3));

        assert_moves!(
            board,
            [
                a7a6, b7b6, c7c6, d7d6, e7e6, f7f6, g7g6, h7h6, a7a5, b7b5, c7c5, d7d5, e7e5, f7f5,
                g7g5, h7h5, b8a6, b8c6, g8h6, g8f6
            ]
        );
    }

    #[test]
    fn cannot_make_a_move_that_leaves_king_in_check() {
        let mut board = fen("8/8/8/8/8/8/KP5r/8 w");
        // Note that the pawn is not allowed to move
        assert_moves!(board, [a2a1, a2b1, a2a3, a2b3,]);
    }

    #[test]
    fn capturing_moves_are_all_pseudo_legal_moves_that_capture_a_piece() {
        let mut board = fen("8/8/8/8/1b1N4/1Q6/p3n3/8 w");
        let capturing_moves: HashSet<Move> = board.capturing_moves(White).collect();

        let expected: HashSet<Move> = board
            .pseudo_legal_moves()
            .filter(|mov| {
                let pieces_before = board.occupancy().count();
                let pmov = board.make_move(*mov);
                let pieces_after = board.occupancy().count();
                board.unmake_move(pmov);
                pieces_before != pieces_after
            })
            .collect();

        assert_eq!(capturing_moves, expected);
    }
}
