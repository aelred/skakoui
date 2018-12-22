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

impl Board {
    pub fn moves<'a>(&'a mut self) -> impl Iterator<Item = Move> + 'a {
        let pseudo_legal_moves = self.pseudo_legal_moves();

        // TODO: this is a very inefficient way to confirm if in check
        pseudo_legal_moves.filter(move |mov| {
            self.make_move(*mov);
            let in_check = self.can_take_king();
            self.unmake_move(*mov);
            !in_check
        })
    }

    pub fn pseudo_legal_moves(&self) -> Box<dyn Iterator<Item = Move>> {
        match self.player() {
            Player::White => Box::new(self.moves_of_type::<AllMoves<WhitePlayer>>()),
            Player::Black => Box::new(self.moves_of_type::<AllMoves<BlackPlayer>>()),
        }
    }

    pub fn capturing_moves(&self) -> Box<dyn Iterator<Item = Move>> {
        match self.player() {
            Player::White => Box::new(self.moves_of_type::<CapturingMoves<WhitePlayer>>()),
            Player::Black => Box::new(self.moves_of_type::<CapturingMoves<BlackPlayer>>()),
        }
    }

    fn moves_of_type<M: Movement>(&self) -> AllPiecesIter<M::PawnIter> {
        AllPiecesIter {
            king: M::piece::<KingType>(self),
            queen: M::piece::<QueenType>(self),
            rook: M::piece::<RookType>(self),
            bishop: M::piece::<BishopType>(self),
            knight: M::piece::<KnightType>(self),
            pawn: M::pawn(self),
        }
    }

    fn can_take_king(&self) -> bool {
        let king = Piece::new(self.player().opponent(), PieceType::King);

        if let Some(king_pos) = self.bitboard_piece(king).squares().next() {
            for mov in self.pseudo_legal_moves() {
                if mov.to() != king_pos {
                    continue;
                }

                let mut after_move = self.clone();
                after_move.make_move(mov);
                if after_move.count(king) == 0 {
                    return true;
                }
            }

            false
        } else {
            false
        }
    }
}

struct AllPiecesIter<I> {
    king: MovesIter<KingType>,
    queen: MovesIter<QueenType>,
    rook: MovesIter<RookType>,
    bishop: MovesIter<BishopType>,
    knight: MovesIter<KnightType>,
    pawn: I,
}

impl<I: Iterator<Item = Move>> Iterator for AllPiecesIter<I> {
    type Item = Move;

    fn next(&mut self) -> Option<Move> {
        if let Some(mov) = self.king.next() {
            return Some(mov);
        }
        if let Some(mov) = self.queen.next() {
            return Some(mov);
        }
        if let Some(mov) = self.rook.next() {
            return Some(mov);
        }
        if let Some(mov) = self.bishop.next() {
            return Some(mov);
        }
        if let Some(mov) = self.knight.next() {
            return Some(mov);
        }
        self.pawn.next()
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
                let attacks = PT::movement(source, self.occupancy) & self.mask;
                let targets = attacks.squares();
                self.target_iter = Some(TargetIter { source, targets });
            }

            let target_iter = self.target_iter.as_mut().unwrap();

            let source = target_iter.source;

            if let Some(target) = target_iter.targets.next() {
                return Some(Move::new(PT::PIECE_TYPE, source, target));
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

    fn next(&mut self) -> Option<Move> {
        if let Some(target) = self.pushes.next() {
            let source = target.shift_rank(-P::DIRECTION);
            return Some(Move::new(PieceType::Pawn, source, target));
        }

        if let Some(target) = self.double_pushes.next() {
            let source = target.shift_rank(-P::DIRECTION * 2);
            return Some(Move::new(PieceType::Pawn, source, target));
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

    fn next(&mut self) -> Option<Move> {
        if let Some(target) = self.captures_east.next() {
            let source = target.shift_rank(-P::DIRECTION).shift_file(1);
            return Some(Move::new(PieceType::Pawn, source, target));
        }

        if let Some(target) = self.captures_west.next() {
            let source = target.shift_rank(-P::DIRECTION).shift_file(-1);
            return Some(Move::new(PieceType::Pawn, source, target));
        }

        None
    }
}

trait Movement {
    type PawnIter: Iterator<Item = Move>;

    fn pawn(board: &Board) -> Self::PawnIter;

    fn piece<PT: PieceTypeT>(board: &Board) -> MovesIter<PT> {
        MovesIter::new(board, Self::movement_mask(board))
    }

    fn movement_mask(board: &Board) -> Bitboard;
}

struct AllMoves<P>(PhantomData<P>);
impl<P: PlayerType> Movement for AllMoves<P> {
    type PawnIter = PawnMovesIter<P>;

    fn pawn(board: &Board) -> PawnMovesIter<P> {
        let piece = Piece::new(P::PLAYER, PieceType::Pawn);

        let pawns = board.bitboard_piece(piece);
        let free_spaces = !board.occupancy();

        let pawns_forward = P::advance_bitboard(pawns);

        let pushes = pawns_forward & free_spaces;

        let double_mask = bitboards::RANKS[P::PAWN_RANK + P::DIRECTION];
        let double_pushes = P::advance_bitboard(&(pushes & double_mask)) & free_spaces;

        PawnMovesIter {
            pushes: pushes.squares(),
            double_pushes: double_pushes.squares(),
            captures: CapturingMoves::pawn(board),
            _phantom: PhantomData,
        }
    }

    fn movement_mask(board: &Board) -> Bitboard {
        !board.occupancy_player(P::PLAYER)
    }
}

struct CapturingMoves<P>(PhantomData<P>);
impl<P: PlayerType> Movement for CapturingMoves<P> {
    type PawnIter = PawnCapturesIter<P>;

    fn pawn(board: &Board) -> PawnCapturesIter<P> {
        let piece = Piece::new(P::PLAYER, PieceType::Pawn);
        let pawns = board.bitboard_piece(piece);
        let pawns_forward = P::advance_bitboard(pawns);

        let opponent_pieces = board.occupancy_player(P::Opp::PLAYER);

        let captures_east = pawns_forward.shift_file_neg(1) & opponent_pieces;
        let captures_west = pawns_forward.shift_file(1) & opponent_pieces;

        PawnCapturesIter {
            captures_east: captures_east.squares(),
            captures_west: captures_west.squares(),
            _phantom: PhantomData,
        }
    }

    fn movement_mask(board: &Board) -> Bitboard {
        board.occupancy_player(P::Opp::PLAYER)
    }
}

trait PieceTypeT {
    const PIECE_TYPE: PieceType;
    fn movement(source: Square, occupancy: Bitboard) -> Bitboard;
}

struct KingType;
impl PieceTypeT for KingType {
    const PIECE_TYPE: PieceType = PieceType::King;

    #[inline]
    fn movement(source: Square, _: Bitboard) -> Bitboard {
        bitboards::KING_MOVES[source]
    }
}

struct KnightType;
impl PieceTypeT for KnightType {
    const PIECE_TYPE: PieceType = PieceType::Knight;

    #[inline]
    fn movement(source: Square, _: Bitboard) -> Bitboard {
        bitboards::KNIGHT_MOVES[source]
    }
}

struct RookType;
impl PieceTypeT for RookType {
    const PIECE_TYPE: PieceType = PieceType::Rook;

    #[inline]
    fn movement(source: Square, occupancy: Bitboard) -> Bitboard {
        slide::<NorthSouth>(source, occupancy) | slide::<EastWest>(source, occupancy)
    }
}

struct BishopType;
impl PieceTypeT for BishopType {
    const PIECE_TYPE: PieceType = PieceType::Bishop;

    #[inline]
    fn movement(source: Square, occupancy: Bitboard) -> Bitboard {
        slide::<Diagonal>(source, occupancy) | slide::<AntiDiagonal>(source, occupancy)
    }
}

struct QueenType;
impl PieceTypeT for QueenType {
    const PIECE_TYPE: PieceType = PieceType::Queen;

    #[inline]
    fn movement(source: Square, occupancy: Bitboard) -> Bitboard {
        slide::<NorthSouth>(source, occupancy)
            | slide::<EastWest>(source, occupancy)
            | slide::<Diagonal>(source, occupancy)
            | slide::<AntiDiagonal>(source, occupancy)
    }
}

fn slide<Dir: SlideDirection>(source: Square, occupancy: Bitboard) -> Bitboard {
    let pos_movement = Dir::positive_bitboard(source);
    let mut blockers = pos_movement & occupancy;
    // Set the last square so there is always a blocking square (no need to branch)
    blockers.set(Square::H8);
    let blocking_square = blockers.first_set();
    let pos_movement = pos_movement ^ Dir::positive_bitboard(blocking_square);

    let neg_movement = Dir::negative_bitboard(source);
    let mut blockers = neg_movement & occupancy;
    // Set the last square so there is always a blocking square (no need to branch)
    blockers.set(Square::A1);
    let blocking_square = blockers.last_set();
    let neg_movement = neg_movement ^ Dir::negative_bitboard(blocking_square);

    pos_movement | neg_movement
}

trait SlideDirection {
    fn positive_bitboard(source: Square) -> Bitboard;
    fn negative_bitboard(source: Square) -> Bitboard;
}

struct NorthSouth;
impl SlideDirection for NorthSouth {
    #[inline]
    fn positive_bitboard(source: Square) -> Bitboard {
        bitboards::NORTH[source]
    }

    #[inline]
    fn negative_bitboard(source: Square) -> Bitboard {
        bitboards::SOUTH[source]
    }
}

struct EastWest;
impl SlideDirection for EastWest {
    #[inline]
    fn positive_bitboard(source: Square) -> Bitboard {
        bitboards::EAST[source]
    }

    #[inline]
    fn negative_bitboard(source: Square) -> Bitboard {
        bitboards::WEST[source]
    }
}

struct Diagonal;
impl SlideDirection for Diagonal {
    #[inline]
    fn positive_bitboard(source: Square) -> Bitboard {
        bitboards::POSITIVE_DIAGONALS[source]
    }

    #[inline]
    fn negative_bitboard(source: Square) -> Bitboard {
        bitboards::NEGATIVE_DIAGONALS[source]
    }
}

struct AntiDiagonal;
impl SlideDirection for AntiDiagonal {
    #[inline]
    fn positive_bitboard(source: Square) -> Bitboard {
        bitboards::POSITIVE_ANTIDIAGONALS[source]
    }

    #[inline]
    fn negative_bitboard(source: Square) -> Bitboard {
        bitboards::NEGATIVE_ANTIDIAGONALS[source]
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
                Pa2a3, Pb2b3, Pc2c3, Pd2d3, Pe2e3, Pf2f3, Pg2g3, Ph2h3, Pa2a4, Pb2b4, Pc2c4, Pd2d4,
                Pe2e4, Pf2f4, Pg2g4, Ph2h4, Nb1a3, Nb1c3, Ng1h3, Ng1f3
            ]
        );
    }

    #[test]
    fn can_generate_all_possible_starting_moves_for_black() {
        let mut board = Board::default();
        board.make_move(mov!(Pa2a3));

        assert_moves!(
            board,
            [
                Pa7a6, Pb7b6, Pc7c6, Pd7d6, Pe7e6, Pf7f6, Pg7g6, Ph7h6, Pa7a5, Pb7b5, Pc7c5, Pd7d5,
                Pe7e5, Pf7f5, Pg7g5, Ph7h5, Nb8a6, Nb8c6, Ng8h6, Ng8f6
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

        assert_moves!(board, [Pd5c4, Pd5e4]);
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

        assert_moves!(board, [Pa3a4]);
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

        board.make_move(mov!(Pa2a4));

        assert_moves!(board, [Pb4a3]);
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

        board.make_move(mov!(Pa3a4));

        assert_moves!(board, []);
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
        assert_moves!(board, [Kb3a2, Kb3a3, Kb3c3, Kb3a4, Kb3b4, Kb3c4,]);
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

        assert_moves!(board, [Nb3a1, Nb3c1, Nb3d2, Nb3d4, Nb3a5,]);
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

        assert_moves!(board, [Rb3b1, Rb3b2, Rb3a3, Rb3c3, Rb3b4,]);
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

        assert_moves!(board, [Bb3d1, Bb3a2, Bb3c2, Bb3a4,]);
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
            [Qb3d1, Qb3a2, Qb3c2, Qb3a4, Qb3a3, Qb3b1, Qb3b2, Qb3b4, Qb3b5, Qb3b6,]
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
        assert_moves!(board, [Ka2a1, Ka2b1, Ka2a3, Ka2b3,]);
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
}
