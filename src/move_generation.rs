use crate::Bitboard;
use crate::Black;
use crate::Board;
use crate::Move;
use crate::Piece;
use crate::PieceType;
use crate::Player;
use crate::PlayerT;
use crate::White;

mod bishop;
mod king;
mod knight;
mod pawn;
mod piece_type;
mod queen;
mod rook;

use crate::move_generation::bishop::BishopType;
use crate::move_generation::king::KingType;
use crate::move_generation::knight::KnightType;
use crate::move_generation::pawn::PawnType;
use crate::move_generation::piece_type::{Movable, PieceT};
use crate::move_generation::queen::QueenType;
use crate::move_generation::rook::RookType;
use crate::piece::PieceType::King;

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
            Player::White => Box::new(self.moves_of_type(White, AllMoves(White))),
            Player::Black => Box::new(self.moves_of_type(Black, AllMoves(Black))),
        }
    }

    /// Lazy iterator of all pseudo-legal moves. Pseudo-legal means they ignore:
    /// 1. Check
    /// 2. King captures
    /// 3. Castling through check
    pub fn pseudo_legal_moves_for_typed(&self, player: impl PlayerT) -> impl Iterator<Item = Move> {
        self.moves_of_type(player, AllMoves(player))
    }

    /// Lazy iterator of all capturing moves
    pub fn capturing_moves(&self, player: impl PlayerT) -> impl Iterator<Item = Move> {
        self.moves_of_type(player, CapturingMoves(player))
    }

    fn moves_of_type<P: PlayerT, M: Movement<P>>(
        &self,
        player: P,
        movement: M,
    ) -> impl Iterator<Item = Move> {
        let mask = movement.movement_mask(self);

        let king = PieceT::new(player, KingType).moves(self, mask);
        let queen = PieceT::new(player, QueenType).moves(self, mask);
        let rook = PieceT::new(player, RookType).moves(self, mask);
        let bishop = PieceT::new(player, BishopType).moves(self, mask);
        let knight = PieceT::new(player, KnightType).moves(self, mask);
        let pawn = PieceT::new(player, PawnType).moves(self, mask);

        king.chain(queen)
            .chain(rook)
            .chain(bishop)
            .chain(knight)
            .chain(pawn)
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

trait Movement<P: PlayerT> {
    /// Mask of valid target squares to control what moves are generated.
    /// For example, we can restrict to capturing moves by masking to "squares occupied by enemy
    /// pieces" (except for en-passant but screw en-passant).
    fn movement_mask(&self, board: &Board) -> Bitboard;
}

struct AllMoves<P>(P);

impl<P: PlayerT> Movement<P> for AllMoves<P> {
    fn movement_mask(&self, board: &Board) -> Bitboard {
        !board.occupancy_player(self.0.value())
    }
}

struct CapturingMoves<P>(P);

impl<P: PlayerT> Movement<P> for CapturingMoves<P> {
    fn movement_mask(&self, board: &Board) -> Bitboard {
        board.occupancy_player(self.0.opponent().value())
    }
}

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
