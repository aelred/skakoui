use crate::bitboard::SquareIterator;
use crate::move_generation::piece_type::PieceTypeT;
use crate::{bitboards, Bitboard, Board, BoardFlags, Move, Piece, PieceType, PlayerT, Square};
use std::iter::FlatMap;

#[derive(Default)]
pub struct Pawn;
impl PieceTypeT for Pawn {
    const PIECE_TYPE: PieceType = PieceType::Pawn;

    fn movement(
        &self,
        source: Square,
        occupancy: Bitboard,
        player: impl PlayerT,
        _: BoardFlags,
    ) -> Bitboard {
        let (pushes, double_pushes) = move_boards(source.into(), occupancy, player);
        self.attacks(source, occupancy, player) | pushes | double_pushes
    }

    fn attacks(&self, source: Square, occupancy: Bitboard, player: impl PlayerT) -> Bitboard {
        let (captures_east, captures_west) = capture_boards(source.into(), occupancy, player);
        captures_east | captures_west
    }
}

pub type Moves<P> = FlatMap<PawnMovesIter<P>, Vec<Move>, fn(Move) -> Vec<Move>>;
pub type Attacks<P> = FlatMap<PawnCapturesIter<P>, Vec<Move>, fn(Move) -> Vec<Move>>;

pub fn moves<P: PlayerT>(_: P, board: &Board, _: Bitboard) -> Moves<P> {
    PawnMovesIter::from_board(board, P::default()).flat_map(Move::with_valid_promotions::<P>)
}

pub fn attacks<P: PlayerT>(_: P, board: &Board, _: Bitboard) -> Attacks<P> {
    PawnCapturesIter::from_board(board, P::default()).flat_map(Move::with_valid_promotions::<P>)
}

pub struct PawnMovesIter<P> {
    player: P,
    pushes: SquareIterator,
    double_pushes: SquareIterator,
    captures: PawnCapturesIter<P>,
}

impl<P: PlayerT> PawnMovesIter<P> {
    fn new(
        sources: Bitboard,
        occupancy: Bitboard,
        opponent_occupancy: Bitboard,
        player: P,
    ) -> Self {
        let free_spaces = !occupancy;
        let pawns_forward = player.advance_bitboard(sources);
        let pushes = pawns_forward & free_spaces;
        let double_mask = bitboards::RANKS[player.pawn_rank() + player.multiplier()];
        let double_pushes = player.advance_bitboard(pushes & double_mask) & free_spaces;

        Self {
            player,
            pushes: pushes.squares(),
            double_pushes: double_pushes.squares(),
            captures: PawnCapturesIter::new(sources, opponent_occupancy, player),
        }
    }

    fn from_board(board: &Board, player: P) -> Self {
        let piece = Piece::new(player.value(), PieceType::Pawn);
        let pawns = board.bitboard_piece(piece);
        let opponent_occupancy = board.occupancy_player(player.opponent().value());
        Self::new(*pawns, board.occupancy(), opponent_occupancy, player)
    }
}

impl<P: PlayerT> Iterator for PawnMovesIter<P> {
    type Item = Move;

    fn next(&mut self) -> Option<Move> {
        if let Some(target) = self.pushes.next() {
            let source = target.shift_rank(-self.player.multiplier());
            return Some(Move::new(source, target));
        }

        if let Some(target) = self.double_pushes.next() {
            let source = target.shift_rank(-self.player.multiplier() * 2);
            return Some(Move::new(source, target));
        }

        self.captures.next()
    }
}

pub struct PawnCapturesIter<P> {
    player: P,
    captures_east: SquareIterator,
    captures_west: SquareIterator,
}

impl<P: PlayerT> PawnCapturesIter<P> {
    fn new(sources: Bitboard, occupancy: Bitboard, player: P) -> Self {
        let (captures_east, captures_west) = capture_boards(sources, occupancy, player);

        Self {
            player,
            captures_east: captures_east.squares(),
            captures_west: captures_west.squares(),
        }
    }

    fn from_board(board: &Board, player: P) -> Self {
        let piece = Piece::new(player.value(), PieceType::Pawn);
        let pawns = board.bitboard_piece(piece);
        let opponent_occupancy = board.occupancy_player(player.opponent().value());
        Self::new(*pawns, opponent_occupancy, player)
    }
}

impl<P: PlayerT> Iterator for PawnCapturesIter<P> {
    type Item = Move;

    fn next(&mut self) -> Option<Move> {
        if let Some(target) = self.captures_east.next() {
            let source = target.shift_rank(-self.player.multiplier()).shift_file(1);
            return Some(Move::new(source, target));
        }

        if let Some(target) = self.captures_west.next() {
            let source = target.shift_rank(-self.player.multiplier()).shift_file(-1);
            return Some(Move::new(source, target));
        }

        None
    }
}

fn move_boards(
    sources: Bitboard,
    occupancy: Bitboard,
    player: impl PlayerT,
) -> (Bitboard, Bitboard) {
    let free_spaces = !occupancy;
    let pawns_forward = player.advance_bitboard(sources);
    let pushes = pawns_forward & free_spaces;
    let double_mask = bitboards::RANKS[player.pawn_rank() + player.multiplier()];
    let double_pushes = player.advance_bitboard(pushes & double_mask) & free_spaces;
    (pushes, double_pushes)
}

fn capture_boards(
    sources: Bitboard,
    targets: Bitboard,
    player: impl PlayerT,
) -> (Bitboard, Bitboard) {
    let pawns_forward = player.advance_bitboard(sources);
    (
        pawns_forward.shift_file_neg(1) & targets,
        pawns_forward.shift_file(1) & targets,
    )
}

#[cfg(test)]
mod tests {
    use crate::board::tests::fen;
    use crate::{assert_moves, mov};

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
}
