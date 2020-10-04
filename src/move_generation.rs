use crate::bitboard::SquareIterator;
use crate::Bitboard;
use crate::BlackPlayer;
use crate::Board;
use crate::Move;
use crate::Piece;
use crate::PieceType;
use crate::Player;
use crate::PlayerType;
use crate::Square;
use crate::WhitePlayer;
use std::marker::PhantomData;

mod castling;
mod pawn;
mod piece_type;

use self::castling::CastlingIter;
use self::piece_type::BishopType;
use self::piece_type::KingType;
use self::piece_type::KnightType;
use self::piece_type::PieceTypeT;
use self::piece_type::QueenType;
use self::piece_type::RookType;
use crate::move_generation::pawn::{PawnCapturesIter, PawnMovesIter};
use crate::piece::PieceType::King;

impl Board {
    /// Lazy iterator of all legal moves
    pub fn moves<'a>(&'a mut self) -> impl Iterator<Item = Move> + 'a {
        // TODO: this is a very inefficient way to confirm if in check
        // TODO: disallow castling when in check or through check
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
            Player::White => Box::new(self.moves_of_type::<AllMoves<WhitePlayer>>()),
            Player::Black => Box::new(self.moves_of_type::<AllMoves<BlackPlayer>>()),
        }
    }

    /// Lazy iterator of all pseudo-legal moves. Pseudo-legal means they ignore:
    /// 1. Check
    /// 2. King captures
    /// 3. Castling through check
    pub fn pseudo_legal_moves_for_typed<P: PlayerType>(&self) -> impl Iterator<Item = Move> {
        self.moves_of_type::<AllMoves<P>>()
    }

    /// Lazy iterator of all capturing moves
    pub fn capturing_moves<P: PlayerType>(&self) -> impl Iterator<Item = Move> {
        self.moves_of_type::<CapturingMoves<P>>()
    }

    fn moves_of_type<M: Movement>(&self) -> impl Iterator<Item = Move> {
        let king = M::piece::<KingType>(self);
        let queen = M::piece::<QueenType>(self);
        let rook = M::piece::<RookType>(self);
        let bishop = M::piece::<BishopType>(self);
        let knight = M::piece::<KnightType>(self);
        let pawn = M::pawn(self).flat_map(Move::with_valid_promotions::<M::Player>);
        let castle = M::castling(&self);

        king.chain(queen)
            .chain(rook)
            .chain(bishop)
            .chain(knight)
            .chain(pawn)
            .chain(castle)
    }

    pub fn check_legal(&mut self, mov: Move) -> bool {
        // TODO: this is a very inefficient way to confirm if in check
        // TODO: disallow castling when in check or through check
        let me = self.player();
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

struct MovesIter<P> {
    mask: Bitboard,
    occupancy: Bitboard,
    sources: SquareIterator,
    target_iter: Option<TargetIter>,
    _phantom: PhantomData<P>,
}

struct TargetIter {
    source: Square,
    targets: SquareIterator,
}

impl<PT: PieceTypeT> MovesIter<PT> {
    fn new(board: &Board, mask: Bitboard) -> Self {
        let piece = Piece::new(board.player(), PT::PIECE_TYPE);
        MovesIter {
            mask,
            occupancy: board.occupancy(),
            sources: board.bitboard_piece(piece).squares(),
            target_iter: None,
            _phantom: PhantomData,
        }
    }
}

impl<PT: PieceTypeT> Iterator for MovesIter<PT> {
    type Item = Move;

    fn next(&mut self) -> Option<Move> {
        loop {
            if self.target_iter.is_none() {
                let source = self.sources.next()?;
                let targets = (PT::movement(source, self.occupancy) & self.mask).squares();
                self.target_iter = Some(TargetIter { source, targets });
            }

            let target_iter = self.target_iter.as_mut().unwrap();

            let source = target_iter.source;

            if let Some(target) = target_iter.targets.next() {
                return Some(Move::new(source, target));
            } else {
                self.target_iter = None;
            }
        }
    }
}

trait Movement {
    type PawnIter: Iterator<Item = Move>;
    type Castling: Iterator<Item = Move>;
    type Player: PlayerType;

    /// Iterator of pawn moves
    fn pawn(board: &Board) -> Self::PawnIter;

    /// Iterator of moves for a given piece
    fn piece<PT: PieceTypeT>(board: &Board) -> MovesIter<PT> {
        MovesIter::new(board, Self::movement_mask(board))
    }

    /// Iterator of pseudo-legal castling moves
    fn castling(board: &Board) -> Self::Castling;

    /// Mask of valid target squares to control what moves are generated.
    /// For example, we can restrict to capturing moves by masking to "squares occupied by enemy
    /// pieces" (except for en-passant but screw en-passant).
    fn movement_mask(board: &Board) -> Bitboard;
}

struct AllMoves<P>(PhantomData<P>);

impl<P: PlayerType> Movement for AllMoves<P> {
    type PawnIter = PawnMovesIter<P>;
    type Castling = CastlingIter<P>;
    type Player = P;

    fn pawn(board: &Board) -> PawnMovesIter<P> {
        PawnMovesIter::new(board)
    }

    fn castling(board: &Board) -> Self::Castling {
        CastlingIter::new(board)
    }

    fn movement_mask(board: &Board) -> Bitboard {
        !board.occupancy_player(P::PLAYER)
    }
}

struct CapturingMoves<P>(PhantomData<P>);

impl<P: PlayerType> Movement for CapturingMoves<P> {
    type PawnIter = PawnCapturesIter<P>;
    type Castling = std::iter::Empty<Move>;
    type Player = P;

    fn pawn(board: &Board) -> PawnCapturesIter<P> {
        PawnCapturesIter::new(board)
    }

    fn castling(_: &Board) -> Self::Castling {
        std::iter::empty()
    }

    fn movement_mask(board: &Board) -> Bitboard {
        board.occupancy_player(P::Opp::PLAYER)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::tests::fen;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;

    macro_rules! mov {
        ($mov:expr) => {
            stringify!($mov).parse::<Move>().unwrap()
        };
    }

    macro_rules! assert_moves {
        ($board:expr, [$($moves:expr),* $(,)*]) => {
            let mut moves: Vec<Move> = $board.moves().collect();
            moves.sort();

            let mut expected_moves: Vec<Move> = [
                $(mov!($moves)),*
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
    fn pawn_cannot_move_at_end_of_board() {
        // Such a situation is impossible in normal chess, but it's an edge case that could cause
        // something to go out of bounds.
        let mut board = fen("8/8/8/8/8/8/8/p7 b");
        assert_moves!(board, []);
    }

    #[test]
    fn pawn_cannot_capture_piece_directly_in_front_of_it() {
        let mut board = fen("8/5p2/5N2/3p1N2/3Q4/8/8/8 b");
        assert_moves!(board, []);
    }

    #[test]
    fn pawn_can_capture_pieces_on_diagonal() {
        let mut board = fen("8/8/8/3p4/2NPN3/8/8/8 b");
        assert_moves!(board, [d5c4, d5e4]);
    }

    #[test]
    fn pawn_cannot_capture_same_player_pieces() {
        let mut board = fen("8/8/8/3p4/3Pp3/4P3/8/8 b");
        assert_moves!(board, []);
    }

    #[test]
    fn pawn_cannot_double_push_if_blocked() {
        let mut board = fen("8/8/8/8/8/p7/P7/8 w");
        assert_moves!(board, []);
    }

    #[test]
    fn pawn_cannot_double_push_when_not_at_initial_position() {
        let mut board = fen("8/8/8/8/8/P7/8/8 w");
        assert_moves!(board, [a3a4]);
    }

    #[ignore]
    #[test]
    fn pawn_can_take_another_pawn_en_passant_immediately_after_double_push() {
        let mut board = fen("8/8/8/8/1p6/1N6/P7/8 w");
        board.make_move(mov!(a2a4));
        assert_moves!(board, [b4a3]);
    }

    #[ignore]
    #[test]
    fn pawn_cannot_take_another_pawn_en_passant_in_other_situations() {
        let mut board = fen("8/8/8/8/1p6/PN6/8/8 w");
        board.make_move(mov!(a3a4));
        assert_moves!(board, []);
    }

    #[test]
    fn pawn_can_be_promoted_at_end_of_board() {
        let mut board = fen("8/P7/8/8/8/8/8/8 w");
        assert_moves!(board, [a7a8N, a7a8B, a7a8R, a7a8Q]);
    }

    #[test]
    fn pawn_can_capture_and_promote_at_end_of_board() {
        let mut board = fen("nq6/P7/8/8/8/8/8/8 w");
        assert_moves!(board, [a7b8N, a7b8B, a7b8R, a7b8Q]);
    }

    #[test]
    fn king_can_move_and_capture_one_square_in_any_direction() {
        let mut board = fen("8/8/8/8/8/1Kp5/2P5/8 w");
        // b3b2 is missing because it puts the king in check
        assert_moves!(board, [b3a2, b3a3, b3c3, b3a4, b3b4, b3c4,]);
    }

    #[test]
    fn king_can_castle() {
        let mut board = fen("8/8/8/8/8/8/r6r/R3K2R w");
        assert_moves!(
            board,
            [e1c1, e1g1, e1d1, e1f1, a1b1, a1c1, a1d1, a1a2, h1g1, h1f1, h1h2]
        );
        let mut board = fen("r3k2r/R6R/8/8/8/8/8/8 b");
        assert_moves!(
            board,
            [e8c8, e8g8, e8d8, e8f8, a8b8, a8c8, a8d8, a8a7, h8g8, h8f8, h8h7]
        );
    }

    #[test]
    fn knight_can_move_and_capture_in_its_weird_way() {
        let mut board = fen("8/8/2p5/2P5/3p4/1N6/8/8 w");
        assert_moves!(board, [b3a1, b3c1, b3d2, b3d4, b3a5,]);
    }

    #[test]
    fn rook_can_move_and_capture_along_rank_and_file() {
        let mut board = fen("8/8/1p6/1P6/8/1Rq5/8/8 w");
        assert_moves!(board, [b3b1, b3b2, b3a3, b3c3, b3b4,]);
    }

    #[test]
    fn bishop_can_move_and_capture_diagonally() {
        let mut board = fen("8/8/8/2p5/2P5/1B6/8/3b4 w");
        assert_moves!(board, [b3d1, b3a2, b3c2, b3a4,]);
    }

    #[test]
    fn queen_can_move_and_capture_in_all_directions() {
        let mut board = fen("8/8/1p6/2p5/2P5/1QP5/8/3b4 w");
        assert_moves!(
            board,
            [b3d1, b3a2, b3c2, b3a4, b3a3, b3b1, b3b2, b3b4, b3b5, b3b6,]
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
        let capturing_moves: HashSet<Move> = board.capturing_moves::<WhitePlayer>().collect();

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
