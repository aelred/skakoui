use crate::bitboard::SquareIterator;
use crate::{bitboards, Board, PlayerType};
use crate::{Bitboard, Piece};
use crate::{BoardFlags, PieceType};
use crate::{File, Square};
use std::marker::PhantomData;

/// Type-level representation of [PieceType].
pub trait PieceTypeT: Sized {
    const PIECE_TYPE: PieceType;

    /// Returns all moves for this piece when placed at the given square.
    ///
    /// Usually the same as [attacks] except also including moves that don't capture, like castling.
    fn movement<P: PlayerType>(source: Square, occupancy: Bitboard, _: BoardFlags) -> Bitboard {
        Self::attacks(source, occupancy)
    }

    /// Returns all squares this piece can attack when placed at the given square.
    ///
    /// This assumes that any occupied square can be captured - even though it might be friendly.
    /// Friendly captures are filtered out later.
    fn attacks(source: Square, occupancy: Bitboard) -> Bitboard;

    fn moves<P: PlayerType>(self, board: &Board) -> MovesIter<P, Self> {
        let piece = Piece::new(P::PLAYER, Self::PIECE_TYPE);
        MovesIter {
            occupancy: board.occupancy(),
            sources: board.bitboard_piece(piece).squares(),
            flags: board.flags(),
            _phantom_p: PhantomData,
            _phantom_pt: PhantomData,
        }
    }
}

pub struct KingType;
impl PieceTypeT for KingType {
    const PIECE_TYPE: PieceType = PieceType::King;

    fn movement<P: PlayerType>(source: Square, occupancy: Bitboard, flags: BoardFlags) -> Bitboard {
        let mut movement = Self::attacks(source, occupancy);

        if flags.is_set(P::CASTLE_KINGSIDE_FLAG)
            && (P::CASTLE_KINGSIDE_CLEAR & occupancy).is_empty()
        {
            movement.set(Square::new(File::G, P::PLAYER.back_rank()));
        }

        if flags.is_set(P::CASTLE_QUEENSIDE_FLAG)
            && (P::CASTLE_QUEENSIDE_CLEAR & occupancy).is_empty()
        {
            movement.set(Square::new(File::C, P::PLAYER.back_rank()));
        }

        movement
    }

    fn attacks(source: Square, _: Bitboard) -> Bitboard {
        bitboards::KING_MOVES[source]
    }
}

pub struct KnightType;
impl PieceTypeT for KnightType {
    const PIECE_TYPE: PieceType = PieceType::Knight;

    fn attacks(source: Square, _: Bitboard) -> Bitboard {
        bitboards::KNIGHT_MOVES[source]
    }
}

pub struct RookType;
impl PieceTypeT for RookType {
    const PIECE_TYPE: PieceType = PieceType::Rook;

    fn attacks(source: Square, occupancy: Bitboard) -> Bitboard {
        slide::<NorthSouth>(source, occupancy) | slide::<EastWest>(source, occupancy)
    }
}

pub struct BishopType;
impl PieceTypeT for BishopType {
    const PIECE_TYPE: PieceType = PieceType::Bishop;

    fn attacks(source: Square, occupancy: Bitboard) -> Bitboard {
        slide::<Diagonal>(source, occupancy) | slide::<AntiDiagonal>(source, occupancy)
    }
}

pub struct QueenType;
impl PieceTypeT for QueenType {
    const PIECE_TYPE: PieceType = PieceType::Queen;

    fn attacks(source: Square, occupancy: Bitboard) -> Bitboard {
        slide::<NorthSouth>(source, occupancy)
            | slide::<EastWest>(source, occupancy)
            | slide::<Diagonal>(source, occupancy)
            | slide::<AntiDiagonal>(source, occupancy)
    }
}

pub struct MovesIter<P, PT> {
    occupancy: Bitboard,
    sources: SquareIterator,
    flags: BoardFlags,
    _phantom_p: PhantomData<P>,
    _phantom_pt: PhantomData<PT>,
}

impl<P: PlayerType, PT: PieceTypeT> Iterator for MovesIter<P, PT> {
    type Item = (Square, Bitboard);

    fn next(&mut self) -> Option<(Square, Bitboard)> {
        let source = self.sources.next()?;
        let targets = PT::movement::<P>(source, self.occupancy, self.flags);
        Some((source, targets))
    }
}

/// Slide a piece from the source square in the given direction.
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
    fn positive_bitboard(source: Square) -> Bitboard {
        bitboards::NORTH[source]
    }

    fn negative_bitboard(source: Square) -> Bitboard {
        bitboards::SOUTH[source]
    }
}

struct EastWest;
impl SlideDirection for EastWest {
    fn positive_bitboard(source: Square) -> Bitboard {
        bitboards::EAST[source]
    }

    fn negative_bitboard(source: Square) -> Bitboard {
        bitboards::WEST[source]
    }
}

struct Diagonal;
impl SlideDirection for Diagonal {
    fn positive_bitboard(source: Square) -> Bitboard {
        bitboards::POSITIVE_DIAGONALS[source]
    }

    fn negative_bitboard(source: Square) -> Bitboard {
        bitboards::NEGATIVE_DIAGONALS[source]
    }
}

struct AntiDiagonal;
impl SlideDirection for AntiDiagonal {
    fn positive_bitboard(source: Square) -> Bitboard {
        bitboards::POSITIVE_ANTIDIAGONALS[source]
    }

    fn negative_bitboard(source: Square) -> Bitboard {
        bitboards::NEGATIVE_ANTIDIAGONALS[source]
    }
}
