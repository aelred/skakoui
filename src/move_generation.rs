use crate::{bitboards, Bitboard, Board, BoardFlags, File, Move, Piece, PieceType, Player, Square};
use piece_type::PieceTypeT;

mod bishop;
mod king;
mod knight;
pub mod magic;
mod pawn;
mod piece_type;
mod queen;
mod rook;

pub use crate::move_generation::bishop::Bishop;
pub use crate::move_generation::king::King;
pub use crate::move_generation::knight::Knight;
pub use crate::move_generation::pawn::Pawn;
use crate::move_generation::piece_type::PieceT;
pub use crate::move_generation::queen::Queen;
pub use crate::move_generation::rook::Rook;
use std::iter::Chain;

impl Board {
    /// Lazy iterator of all legal moves
    pub fn moves<'a>(&'a mut self) -> impl Iterator<Item = Move> + 'a {
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
        let me = self.player();

        let my_king = Piece::new(me, PieceType::King);

        let king_move = self[mov.from()] == Some(my_king);
        let castling = king_move && (mov.from().file() - mov.to().file()).abs() == 2;
        if castling {
            let through = if mov.to().file() == File::KINGSIDE {
                me.castle_kingside_through()
            } else {
                me.castle_queenside_through()
            };
            let attacks = self.attacks(me.opponent());

            if through & attacks != bitboards::EMPTY {
                return false;
            }
        }

        // TODO: can we avoid actually making the move?
        let pmov = self.make_move(mov);
        let king_board = self.bitboard_piece(my_king);
        let attacks = self.attacks(me.opponent());
        let out_of_check = king_board & attacks == bitboards::EMPTY;
        self.unmake_move(pmov);
        out_of_check
    }

    pub fn check(&self, king_player: impl Player) -> bool {
        let king = Piece::new(king_player, PieceType::King);
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
        let king = self.attacks_for_piece(PieceT::new(player, King));
        let queen = self.attacks_for_piece(PieceT::new(player, Queen));
        let rook = self.attacks_for_piece(PieceT::new(player, Rook));
        let bishop = self.attacks_for_piece(PieceT::new(player, Bishop));
        let knight = self.attacks_for_piece(PieceT::new(player, Knight));
        let pawn = self.attacks_for_piece(PieceT::new(player, Pawn));
        king | queen | rook | bishop | knight | pawn
    }

    fn attacks_for_piece<P: Player, PT: PieceTypeT>(&self, piece: PieceT<P, PT>) -> Bitboard {
        let pt = &piece.piece_type;
        let mut attacks = bitboards::EMPTY;
        for source in self.bitboard_piece(piece.value()).squares() {
            attacks |= pt.attacks(source, self.occupancy(), piece.player, self.flags());
        }
        attacks
    }
}

pub trait Movement: Copy {
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
        piece_type: &impl PieceTypeT,
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
    use super::*;
    use crate::board::tests::fen;
    use crate::{bitboard, mov, White};
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;

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
