use std::iter::Chain;

pub use crate::move_generation::bishop::Bishop;
pub use crate::move_generation::king::King;
pub use crate::move_generation::knight::Knight;
pub use crate::move_generation::pawn::Pawn;
pub use crate::move_generation::piece_type::PieceType;
pub use crate::move_generation::piece_type::PieceTypeV;
pub use crate::move_generation::queen::Queen;
pub use crate::move_generation::rook::Rook;
pub use crate::piece::Piece;
use crate::{bitboards, Bitboard, Board, BoardFlags, File, Move, PlayedMove, Player, Square};

mod bishop;
mod king;
mod knight;
pub mod magic;
mod pawn;
mod piece_type;
mod queen;
mod rook;

impl Board {
    /// Lazy iterator of all legal moves
    pub fn moves(&mut self) -> impl Iterator<Item = Move> + '_ {
        self.pseudo_legal_moves()
            .filter(move |mov| self.check_legal(*mov))
    }

    /// See [pseudo_legal_moves_for]
    pub fn pseudo_legal_moves(&self) -> impl Iterator<Item = Move> {
        self.pseudo_legal_moves_for(self.player())
    }

    /// Lazy iterator of all pseudo-legal moves. Pseudo-legal means they ignore:
    /// 1. Moving into check
    /// 3. Castling through check
    pub fn pseudo_legal_moves_for(&self, player: impl Player) -> impl Iterator<Item = Move> {
        self.moves_of_type(AllMoves(player))
    }

    /// Lazy iterator of all capturing moves
    pub fn capturing_moves(&self, player: impl Player) -> impl Iterator<Item = Move> {
        self.moves_of_type(CapturingMoves(player))
    }

    fn moves_of_type(&self, movement: impl Movement) -> impl Iterator<Item = Move> {
        movement.moves(self)
    }

    pub fn check_legal(&mut self, mov: Move) -> bool {
        match self.make_if_legal(mov) {
            None => false,
            Some(pmov) => {
                self.unmake_move(pmov);
                true
            }
        }
    }

    pub fn make_if_legal(&mut self, mov: Move) -> Option<PlayedMove> {
        let me = self.player();

        let my_king = Piece::new(me, King);
        let piece = self[mov.from()].unwrap();

        let castling = piece == my_king.value() && (mov.from().file() - mov.to().file()).abs() == 2;
        if castling {
            let through = if mov.to().file() == File::KINGSIDE {
                me.castle_kingside_through()
            } else {
                me.castle_queenside_through()
            };
            let attacks = self.attacks(me.opponent());

            if through & attacks != bitboards::EMPTY {
                return None;
            }
        }

        let pmov = self.make_move(mov);
        let king_board = self.bitboard_piece(my_king);
        let attacks = self.attacks(me.opponent());
        let in_check = king_board & attacks != bitboards::EMPTY;
        if in_check {
            // TODO: can we avoid actually making+unmaking the move in this case?
            self.unmake_move(pmov);
            None
        } else {
            Some(pmov)
        }
    }

    pub fn check(&self, king_player: impl Player) -> bool {
        let king = Piece::new(king_player, King);
        match self.bitboard_piece(king).squares().next() {
            Some(king_pos) => self
                .pseudo_legal_moves_for(king_player.opponent())
                .any(|mov| mov.to() == king_pos),
            None => false,
        }
    }

    pub fn checkmate(&mut self) -> bool {
        let in_check = self.check(self.player());
        let mut no_legal_moves = || self.moves().next().is_none();
        in_check && no_legal_moves()
    }

    fn attacks(&self, player: impl Player) -> Bitboard {
        let king = self.attacks_for_piece(Piece::new(player, King));
        let queen = self.attacks_for_piece(Piece::new(player, Queen));
        let rook = self.attacks_for_piece(Piece::new(player, Rook));
        let bishop = self.attacks_for_piece(Piece::new(player, Bishop));
        let knight = self.attacks_for_piece(Piece::new(player, Knight));
        let pawn = self.attacks_for_piece(Piece::new(player, Pawn));
        king | queen | rook | bishop | knight | pawn
    }

    fn attacks_for_piece(&self, piece: Piece<impl Player, impl PieceType>) -> Bitboard {
        let pt = &piece.piece_type;
        let mut attacks = bitboards::EMPTY;
        for source in self.bitboard_piece(piece).squares() {
            attacks |= pt.attacks(source, self.occupancy(), piece.player, self.flags());
        }
        attacks
    }
}

pub trait Movement: Copy {
    type Moves: Iterator<Item = Move>;

    fn movement(
        &self,
        piece_type: &impl PieceType,
        source: Square,
        occupancy: Bitboard,
        flags: BoardFlags,
    ) -> Bitboard;

    fn moves(&self, board: &Board) -> Self::Moves;
}

#[derive(Copy, Clone)]
pub struct AllMoves<P>(P);

impl<P: Player> Movement for AllMoves<P> {
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
        piece_type: &impl PieceType,
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

#[derive(Copy, Clone)]
pub struct CapturingMoves<P>(P);

impl<P: Player> Movement for CapturingMoves<P> {
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
        piece_type: &impl PieceType,
        source: Square,
        occupancy: Bitboard,
        flags: BoardFlags,
    ) -> Bitboard {
        piece_type.attacks(source, occupancy, self.0, flags)
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
    use std::collections::HashSet;

    use pretty_assertions::assert_eq;

    use crate::board::tests::fen;
    use crate::{bitboard, mov, White};

    use super::*;

    #[macro_export]
    macro_rules! assert_moves {
        ($board:expr, [$($moves:expr),* $(,)*]) => {
            let mut moves: Vec<$crate::Move> = $board.moves().collect();
            moves.sort();

            let mut expected_moves: Vec<$crate::Move> = [
                $($crate::mov!($moves)),*
            ].iter().cloned().collect();
            expected_moves.sort();

            assert_eq!(moves, expected_moves, "\n{:?}", $board);
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
    fn when_checked_by_unprotected_adjacent_pc_then_king_can_move_capture_or_other_can_capture() {
        let mut board = fen("n7/k7/BP6/8/8/8/8/8 b");
        assert_moves!(board, [a7b8, a7a6, a7b6, a8b6]);
    }

    #[test]
    fn when_checked_by_protected_adjacent_pc_then_king_can_move_or_other_can_capture() {
        let mut board = fen("n7/k7/BP6/P7/8/8/8/8 b");
        assert_moves!(board, [a7b8, a7a6, a8b6]);
    }

    #[test]
    fn when_checked_by_distant_ray_piece_then_king_can_move_or_other_can_block_or_capture() {
        let mut board = fen("kb6/8/8/8/8/8/R7/r7 b");
        assert_moves!(board, [a8b7, b8a7, a1a2]);
    }

    #[test]
    fn when_checked_by_knight_then_king_can_move_or_other_can_capture() {
        let mut board = fen("k7/b7/1N6/8/8/8/8/8 b");
        assert_moves!(board, [a8b8, a8b7, a7b6]);
    }

    #[test]
    fn when_double_checked_by_distant_pieces_then_king_can_move() {
        let mut board = fen("k1R5/8/2n5/R7/8/8/8/8 b");
        assert_moves!(board, [a8b7]);
    }

    #[test]
    fn when_double_checked_by_adjacent_unprotected_pieces_then_king_can_move_or_capture() {
        let mut board = fen("kR6/1Bq5/8/8/8/8/8/8 b");
        assert_moves!(board, [a8a7, a8b8]);
    }

    #[test]
    fn absolute_pinned_pieces_can_only_move_along_pinned_ray() {
        let mut board = fen("3R2Q1/1Q1r1q2/2n5/Rp1kb2R/8/3p1b2/8/3R3B b");
        assert_moves!(
            board,
            [d7d8, d7d6, f7g8, f7e6, f3h1, f3g2, f3e4, d3d2, d5d6, d5e6, d5e4, d5d4, d5c4, d5c5]
        );
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

    #[test]
    fn attacked_squares_are_all_squares_that_could_be_captured() {
        let board = fen("8/5P2/K7/8/8/2Q5/1R4BN/8 w");
        let expect = bitboard! {
            X X X . X . X X
            X X X . . . X .
            . X X . . X . .
            X X X X X . . .
            . X X X X . X .
            X X . X X X X X
            X X X X X X X .
            . X X . X X . X
        };
        let attacks = board.attacks(White);
        assert_eq!(
            attacks, expect,
            "\nAttacks:\n{}\nExpected:\n{}\nBoard:\n{}",
            attacks, expect, board
        );
    }
}
