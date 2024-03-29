use crate::move_generation::piece_type::{PieceType, PieceTypeT};
use crate::piece::Piece;
use crate::{
    bitboard::SquareIterator, bitboards, Bitboard, Black, Board, BoardFlags, Move, PieceTypeV,
    Player, Square, White,
};
use std::iter::FlatMap;

#[derive(Default, Copy, Clone)]
pub struct Pawn;
impl PieceType for Pawn {
    fn value(self) -> PieceTypeV {
        PieceTypeV::Pawn
    }

    fn attacks(
        &self,
        source: Square,
        _: Bitboard,
        player: impl Player,
        flags: BoardFlags,
    ) -> Bitboard {
        let (captures_east, captures_west) =
            capture_boards(source.into(), bitboards::FULL, player, flags);
        captures_east | captures_west
    }

    fn other_moves(
        &self,
        source: Square,
        occupancy: Bitboard,
        player: impl Player,
        _: BoardFlags,
    ) -> Bitboard {
        let (pushes, double_pushes) = move_boards(source.into(), occupancy, player);
        pushes | double_pushes
    }
}

impl PieceTypeT for Pawn {}

pub type Moves<P> = FlatMap<PawnMovesIter<P>, Promotions, fn(Move) -> Promotions>;
pub type Attacks<P> = FlatMap<PawnCapturesIter<P>, Promotions, fn(Move) -> Promotions>;

pub fn moves<P: Player>(player: P, board: &Board, _: Bitboard) -> Moves<P> {
    PawnMovesIter::from_board(board, player).flat_map(Promotions::new)
}

pub fn attacks<P: Player>(player: P, board: &Board, _: Bitboard) -> Attacks<P> {
    PawnCapturesIter::from_board(board, player).flat_map(Promotions::new)
}

pub struct PawnMovesIter<P> {
    player: P,
    pushes: SquareIterator,
    double_pushes: SquareIterator,
    captures: PawnCapturesIter<P>,
}

impl<P: Player> PawnMovesIter<P> {
    fn new(
        sources: Bitboard,
        occupancy: Bitboard,
        opponent_occupancy: Bitboard,
        player: P,
        flags: BoardFlags,
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
            captures: PawnCapturesIter::new(sources, opponent_occupancy, player, flags),
        }
    }

    fn from_board(board: &Board, player: P) -> Self {
        let piece = Piece::newv(player, Pawn);
        let pawns = board.bitboard_piece(piece);
        let opponent_occupancy = board.occupancy_player(player.opponent().value());
        Self::new(
            pawns,
            board.occupancy(),
            opponent_occupancy,
            player,
            board.flags(),
        )
    }
}

impl<P: Player> Iterator for PawnMovesIter<P> {
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

impl<P: Player> PawnCapturesIter<P> {
    fn new(sources: Bitboard, occupancy: Bitboard, player: P, flags: BoardFlags) -> Self {
        let (captures_east, captures_west) = capture_boards(sources, occupancy, player, flags);

        Self {
            player,
            captures_east: captures_east.squares(),
            captures_west: captures_west.squares(),
        }
    }

    fn from_board(board: &Board, player: P) -> Self {
        let piece = Piece::newv(player, Pawn);
        let pawns = board.bitboard_piece(piece);
        let opponent_occupancy = board.occupancy_player(player.opponent().value());
        Self::new(pawns, opponent_occupancy, player, board.flags())
    }
}

impl<P: Player> Iterator for PawnCapturesIter<P> {
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

const PROMOTIONS: [Option<PieceTypeV>; 4] = [
    Some(PieceTypeV::Queen),
    Some(PieceTypeV::Rook),
    Some(PieceTypeV::Bishop),
    Some(PieceTypeV::Knight),
];

const NO_PROMOTION: [Option<PieceTypeV>; 1] = [None];

pub struct Promotions {
    mov: Move,
    promotions: std::slice::Iter<'static, Option<PieceTypeV>>,
}

impl Promotions {
    fn new(mov: Move) -> Self {
        let rank = mov.to().rank();
        let promotions = if rank == White.back_rank() || rank == Black.back_rank() {
            PROMOTIONS.iter()
        } else {
            NO_PROMOTION.iter()
        };
        Self { mov, promotions }
    }
}

impl Iterator for Promotions {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.promotions.next().map(|piece_type| {
            *self.mov.mut_promoting() = *piece_type;
            self.mov
        })
    }
}

fn move_boards(
    sources: Bitboard,
    occupancy: Bitboard,
    player: impl Player,
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
    mut targets: Bitboard,
    player: impl Player,
    flags: BoardFlags,
) -> (Bitboard, Bitboard) {
    if let Some(ep_square) = flags.en_passant_square(player) {
        targets.set(ep_square);
    }
    let pawns_forward = player.advance_bitboard(sources);
    (
        pawns_forward.shift_file(-1) & targets,
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

    #[test]
    fn pawn_can_take_another_pawn_en_passant_immediately_after_double_push() {
        let mut board = fen("8/8/8/8/1p6/1N6/P7/8 w");
        board.make_move(mov!(a2a4));
        assert_moves!(board, [b4a3]);
    }

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
