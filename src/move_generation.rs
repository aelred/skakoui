use crate::bitboard::SquareIterator;
use crate::bitboards;
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

mod piece_type;

use self::piece_type::BishopType;
use self::piece_type::KingType;
use self::piece_type::KnightType;
use self::piece_type::PieceTypeT;
use self::piece_type::QueenType;
use self::piece_type::RookType;
use crate::piece::PieceType::King;

impl Board {
    /// Lazy iterator of all legal moves
    #[inline]
    pub fn moves<'a>(&'a mut self) -> impl Iterator<Item = Move> + 'a {
        let me = self.player();

        // TODO: this is a very inefficient way to confirm if in check
        self.pseudo_legal_moves().filter(move |mov| {
            let captured = self.make_move(*mov);
            let in_check = self.check(me);
            let captured_king = captured == Some(King);
            self.unmake_move(*mov);
            !(in_check || captured_king)
        })
    }

    /// Lazy iterator of all pseudo-legal moves (moves ignoring check)
    #[inline]
    pub fn pseudo_legal_moves(&self) -> Box<dyn Iterator<Item = Move>> {
        self.pseudo_legal_moves_for(self.player())
    }

    /// Lazy iterator of all pseudo-legal moves (moves ignoring check)
    #[inline]
    pub fn pseudo_legal_moves_for(&self, player: Player) -> Box<dyn Iterator<Item = Move>> {
        match player {
            Player::White => Box::new(self.moves_of_type::<AllMoves<WhitePlayer>>()),
            Player::Black => Box::new(self.moves_of_type::<AllMoves<BlackPlayer>>()),
        }
    }

    /// Lazy iterator of all capturing moves
    #[inline]
    pub fn capturing_moves(&self) -> Box<dyn Iterator<Item = Move>> {
        match self.player() {
            Player::White => Box::new(self.moves_of_type::<CapturingMoves<WhitePlayer>>()),
            Player::Black => Box::new(self.moves_of_type::<CapturingMoves<BlackPlayer>>()),
        }
    }

    #[inline]
    fn moves_of_type<M: Movement>(&self) -> impl Iterator<Item = Move> {
        let mut king = M::piece::<KingType>(self);
        let mut queen = M::piece::<QueenType>(self);
        let mut rook = M::piece::<RookType>(self);
        let mut bishop = M::piece::<BishopType>(self);
        let mut knight = M::piece::<KnightType>(self);
        let mut pawn = M::pawn(self).flat_map(Move::with_valid_promotions::<M::Player>);
        std::iter::from_fn(move || {
            if let Some(mov) = king.next() {
                return Some(mov);
            }
            if let Some(mov) = queen.next() {
                return Some(mov);
            }
            if let Some(mov) = rook.next() {
                return Some(mov);
            }
            if let Some(mov) = bishop.next() {
                return Some(mov);
            }
            if let Some(mov) = knight.next() {
                return Some(mov);
            }
            pawn.next()
        })
    }

    #[inline]
    pub fn check(&self, king_player: Player) -> bool {
        let king = Piece::new(king_player, PieceType::King);
        if let Some(king_pos) = self.bitboard_piece(king).squares().next() {
            self.pseudo_legal_moves_for(king_player.opponent())
                .any(|mov| mov.to() == king_pos)
        } else {
            false
        }
    }

    #[inline]
    pub fn checkmate(&mut self) -> bool {
        let me = self.player();
        let moves: Vec<Move> = self.moves().collect();
        for mov in moves {
            self.make_move(mov);
            let check = self.check(me);
            self.unmake_move(mov);
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
    #[inline]
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

    #[inline]
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

struct PawnMovesIter<P> {
    pushes: SquareIterator,
    double_pushes: SquareIterator,
    captures: PawnCapturesIter<P>,
    _phantom: PhantomData<P>,
}

impl<P: PlayerType> Iterator for PawnMovesIter<P> {
    type Item = Move;

    #[inline]
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

struct PawnCapturesIter<P> {
    captures_east: SquareIterator,
    captures_west: SquareIterator,
    _phantom: PhantomData<P>,
}

impl<P: PlayerType> Iterator for PawnCapturesIter<P> {
    type Item = Move;

    #[inline]
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

trait Movement {
    type PawnIter: Iterator<Item = Move>;
    type Player: PlayerType;

    fn pawn(board: &Board) -> Self::PawnIter;

    fn piece<PT: PieceTypeT>(board: &Board) -> MovesIter<PT> {
        MovesIter::new(board, Self::movement_mask(board))
    }

    fn movement_mask(board: &Board) -> Bitboard;
}

struct AllMoves<P>(PhantomData<P>);
impl<P: PlayerType> Movement for AllMoves<P> {
    type PawnIter = PawnMovesIter<P>;
    type Player = P;

    #[inline]
    fn pawn(board: &Board) -> PawnMovesIter<P> {
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
            captures: CapturingMoves::pawn(board),
            _phantom: PhantomData,
        }
    }

    #[inline]
    fn movement_mask(board: &Board) -> Bitboard {
        !board.occupancy_player(P::PLAYER)
    }
}

struct CapturingMoves<P>(PhantomData<P>);
impl<P: PlayerType> Movement for CapturingMoves<P> {
    type PawnIter = PawnCapturesIter<P>;
    type Player = P;

    #[inline]
    fn pawn(board: &Board) -> PawnCapturesIter<P> {
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

    #[inline]
    fn movement_mask(board: &Board) -> Bitboard {
        board.occupancy_player(P::Opp::PLAYER)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;

    const __: Option<Piece> = None;
    const WK: Option<Piece> = Some(Piece::WK);
    const WQ: Option<Piece> = Some(Piece::WQ);
    const WR: Option<Piece> = Some(Piece::WR);
    const WB: Option<Piece> = Some(Piece::WB);
    const WN: Option<Piece> = Some(Piece::WN);
    const WP: Option<Piece> = Some(Piece::WP);
    const BQ: Option<Piece> = Some(Piece::BQ);
    const BR: Option<Piece> = Some(Piece::BR);
    const BB: Option<Piece> = Some(Piece::BB);
    const BN: Option<Piece> = Some(Piece::BN);
    const BP: Option<Piece> = Some(Piece::BP);

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

        let mut board = Board::new(
            [
                [BP, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::Black,
        );

        assert_moves!(board, []);
    }

    #[test]
    fn pawn_cannot_capture_piece_directly_in_front_of_it() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, WQ, __, __, __, __],
                [__, __, __, BP, __, WN, __, __],
                [__, __, __, __, __, WN, __, __],
                [__, __, __, __, __, BP, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::Black,
        );

        assert_moves!(board, []);
    }

    #[test]
    fn pawn_can_capture_pieces_on_diagonal() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, WN, WP, WN, __, __, __],
                [__, __, __, BP, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::Black,
        );

        assert_moves!(board, [d5c4, d5e4]);
    }

    #[test]
    fn pawn_cannot_capture_same_player_pieces() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, WP, __, __, __],
                [__, __, __, WP, BP, __, __, __],
                [__, __, __, BP, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::Black,
        );

        assert_moves!(board, []);
    }

    #[test]
    fn pawn_cannot_double_push_if_blocked() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [WP, __, __, __, __, __, __, __],
                [BP, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_moves!(board, []);
    }

    #[test]
    fn pawn_cannot_double_push_when_not_at_initial_position() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [WP, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_moves!(board, [a3a4]);
    }

    #[ignore]
    #[test]
    fn pawn_can_take_another_pawn_en_passant_immediately_after_double_push() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [WP, __, __, __, __, __, __, __],
                [__, WN, __, __, __, __, __, __],
                [__, BP, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        board.make_move(mov!(a2a4));

        assert_moves!(board, [b4a3]);
    }

    #[ignore]
    #[test]
    fn pawn_cannot_take_another_pawn_en_passant_in_other_situations() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [WP, WN, __, __, __, __, __, __],
                [__, BP, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        board.make_move(mov!(a3a4));

        assert_moves!(board, []);
    }

    #[test]
    fn pawn_can_be_promoted_at_end_of_board() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [WP, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_moves!(board, [a7a8N, a7a8B, a7a8R, a7a8Q]);
    }

    #[test]
    fn pawn_can_capture_and_promote_at_end_of_board() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [WP, __, __, __, __, __, __, __],
                [BN, BQ, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_moves!(board, [a7b8N, a7b8B, a7b8R, a7b8Q]);
    }

    #[test]
    fn king_can_move_and_capture_one_square_in_any_direction() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, WP, __, __, __, __, __],
                [__, WK, BP, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        // Kb3b2 is missing because it puts the king in check
        assert_moves!(board, [b3a2, b3a3, b3c3, b3a4, b3b4, b3c4,]);
    }

    #[test]
    fn knight_can_move_and_capture_in_its_weird_way() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, WN, __, __, __, __, __, __],
                [__, __, __, BP, __, __, __, __],
                [__, __, WP, __, __, __, __, __],
                [__, __, BP, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_moves!(board, [b3a1, b3c1, b3d2, b3d4, b3a5,]);
    }

    #[test]
    fn rook_can_move_and_capture_along_rank_and_file() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, WR, BQ, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, WP, __, __, __, __, __, __],
                [__, BP, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_moves!(board, [b3b1, b3b2, b3a3, b3c3, b3b4,]);
    }

    #[test]
    fn bishop_can_move_and_capture_diagonally() {
        let mut board = Board::new(
            [
                [__, __, __, BB, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, WB, __, __, __, __, __, __],
                [__, __, WP, __, __, __, __, __],
                [__, __, BP, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_moves!(board, [b3d1, b3a2, b3c2, b3a4,]);
    }

    #[test]
    fn queen_can_move_and_capture_in_all_directions() {
        let mut board = Board::new(
            [
                [__, __, __, BB, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, WQ, WP, __, __, __, __, __],
                [__, __, WP, __, __, __, __, __],
                [__, __, BP, __, __, __, __, __],
                [__, BP, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        assert_moves!(
            board,
            [b3d1, b3a2, b3c2, b3a4, b3a3, b3b1, b3b2, b3b4, b3b5, b3b6,]
        );
    }

    #[test]
    fn cannot_make_a_move_that_leaves_king_in_check() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [WK, WP, __, __, __, __, __, BR],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        // Note that the pawn is not allowed to move
        assert_moves!(board, [a2a1, a2b1, a2a3, a2b3,]);
    }

    #[test]
    fn capturing_moves_are_all_pseudo_legal_moves_that_capture_a_piece() {
        let mut board = Board::new(
            [
                [__, __, __, __, __, __, __, __],
                [BP, __, __, __, BN, __, __, __],
                [__, WQ, __, __, __, __, __, __],
                [__, BB, __, WN, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
                [__, __, __, __, __, __, __, __],
            ],
            Player::White,
        );

        let capturing_moves: HashSet<Move> = board.capturing_moves().collect();

        let expected: HashSet<Move> = board
            .pseudo_legal_moves()
            .filter(|mov| {
                let pieces_before = board.occupancy().count();
                board.make_move(*mov);
                let pieces_after = board.occupancy().count();
                board.unmake_move(*mov);
                pieces_before != pieces_after
            })
            .collect();

        assert_eq!(capturing_moves, expected);
    }

    /// Look up "perft" for more details (and bigger numbers!)
    #[test]
    fn number_of_moves_is_as_expected() {
        // Expected number of moves at different depths
        let expected_moves_at_depth = vec![
            1, 20, 400, 8902, 197_281,
            // 4_865_609 - TODO: fails
        ];

        let mut board = Board::default();

        fn count_moves(board: &mut Board, depth: usize) -> usize {
            if depth == 0 {
                return 1;
            }

            let mut count = 0;

            let moves: Vec<Move> = board.moves().collect();

            // Optimisation - skip making and un-making last moves
            if depth == 1 {
                return moves.len();
            }

            for mov in moves {
                board.make_move(mov);
                count += count_moves(board, depth - 1);
                board.unmake_move(mov);
            }

            count
        }

        for (depth, expected_moves) in expected_moves_at_depth.iter().enumerate() {
            let actual_moves = count_moves(&mut board, depth);

            assert_eq!(*expected_moves, actual_moves);
        }
    }
}
