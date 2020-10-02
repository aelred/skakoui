use crate::bitboards;
use crate::Bitboard;
use crate::PieceType;
use crate::Square;

/// Type-level representation of [PieceType].
pub trait PieceTypeT {
    const PIECE_TYPE: PieceType;

    /// Returns all moves for this piece when placed at the given square.
    ///
    /// This assumes that any occupied square can be captured - even though it might be friendly.
    /// Friendly captures are filtered out later.
    fn movement(source: Square, occupancy: Bitboard) -> Bitboard;
}

pub struct KingType;
impl PieceTypeT for KingType {
    const PIECE_TYPE: PieceType = PieceType::King;

    fn movement(source: Square, _: Bitboard) -> Bitboard {
        bitboards::KING_MOVES[source]
    }
}

pub struct KnightType;
impl PieceTypeT for KnightType {
    const PIECE_TYPE: PieceType = PieceType::Knight;

    fn movement(source: Square, _: Bitboard) -> Bitboard {
        bitboards::KNIGHT_MOVES[source]
    }
}

pub struct RookType;
impl PieceTypeT for RookType {
    const PIECE_TYPE: PieceType = PieceType::Rook;

    fn movement(source: Square, occupancy: Bitboard) -> Bitboard {
        slide::<NorthSouth>(source, occupancy) | slide::<EastWest>(source, occupancy)
    }
}

pub struct BishopType;
impl PieceTypeT for BishopType {
    const PIECE_TYPE: PieceType = PieceType::Bishop;

    fn movement(source: Square, occupancy: Bitboard) -> Bitboard {
        slide::<Diagonal>(source, occupancy) | slide::<AntiDiagonal>(source, occupancy)
    }
}

pub struct QueenType;
impl PieceTypeT for QueenType {
    const PIECE_TYPE: PieceType = PieceType::Queen;

    fn movement(source: Square, occupancy: Bitboard) -> Bitboard {
        slide::<NorthSouth>(source, occupancy)
            | slide::<EastWest>(source, occupancy)
            | slide::<Diagonal>(source, occupancy)
            | slide::<AntiDiagonal>(source, occupancy)
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
