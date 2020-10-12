use crate::bitboard::SquareIterator;
use crate::Square;
use crate::{bitboards, Board, Move, PlayerType};
use crate::{Bitboard, Piece};
use crate::{BoardFlags, PieceType};

pub struct PieceT<P, PT> {
    player: P,
    piece_type: PT,
}

impl<P: PlayerType, PT: PieceTypeT> PieceT<P, PT> {
    pub fn new(player: P, piece_type: PT) -> Self {
        Self { player, piece_type }
    }

    fn value(&self) -> Piece {
        Piece::new(self.player.value(), self.piece_type.value())
    }
}

pub trait Movable {
    type Moves: Iterator<Item = Move>;
    fn moves(self, board: &Board, mask: Bitboard) -> Self::Moves;
}

/// Type-level representation of [PieceType].
pub trait PieceTypeT: Sized {
    const PIECE_TYPE: PieceType;

    fn value(&self) -> PieceType {
        Self::PIECE_TYPE
    }

    /// Returns all moves for this piece when placed at the given square.
    ///
    /// Usually the same as [attacks] except also including moves that don't capture, like castling.
    fn movement(
        &self,
        source: Square,
        occupancy: Bitboard,
        _: impl PlayerType,
        _: BoardFlags,
    ) -> Bitboard {
        self.attacks(source, occupancy)
    }

    /// Returns all squares this piece can attack when placed at the given square.
    ///
    /// This assumes that any occupied square can be captured - even though it might be friendly.
    /// Friendly captures are filtered out later.
    fn attacks(&self, source: Square, occupancy: Bitboard) -> Bitboard;
}

pub struct MovesIter<P, PT> {
    occupancy: Bitboard,
    mask: Bitboard,
    sources: SquareIterator,
    source: Square,
    targets: SquareIterator,
    piece: PieceT<P, PT>,
    flags: BoardFlags,
}

impl<P: PlayerType, PT: PieceTypeT> MovesIter<P, PT> {
    pub(crate) fn new(board: &Board, piece: PieceT<P, PT>, mask: Bitboard) -> Self {
        // arbitrary source square with no targets to avoid empty case
        let source = Square::A1;
        let targets = bitboards::EMPTY.squares();
        Self {
            occupancy: board.occupancy(),
            mask,
            sources: board.bitboard_piece(piece.value()).squares(),
            source,
            targets,
            piece,
            flags: board.flags(),
        }
    }
}

impl<P: PlayerType, PT: PieceTypeT> Iterator for MovesIter<P, PT> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        let target = loop {
            match self.targets.next() {
                None => {
                    self.source = self.sources.next()?;
                    let targets = self.piece.piece_type.movement(
                        self.source,
                        self.occupancy,
                        self.piece.player,
                        self.flags,
                    );
                    self.targets = (targets & self.mask).squares();
                    continue;
                }
                Some(t) => break t,
            };
        };

        Some(Move::new(self.source, target))
    }
}

/// Slide a piece from the source square in the given direction.
pub fn slide(dir: impl SlideDirection, source: Square, occupancy: Bitboard) -> Bitboard {
    let pos_movement = dir.positive_bitboard(source);
    let mut blockers = pos_movement & occupancy;
    // Set the last square so there is always a blocking square (no need to branch)
    blockers.set(Square::H8);
    let blocking_square = blockers.first_set();
    let pos_movement = pos_movement ^ dir.positive_bitboard(blocking_square);

    let neg_movement = dir.negative_bitboard(source);
    let mut blockers = neg_movement & occupancy;
    // Set the last square so there is always a blocking square (no need to branch)
    blockers.set(Square::A1);
    let blocking_square = blockers.last_set();
    let neg_movement = neg_movement ^ dir.negative_bitboard(blocking_square);

    pos_movement | neg_movement
}
pub trait SlideDirection {
    fn positive_bitboard(&self, source: Square) -> Bitboard;
    fn negative_bitboard(&self, source: Square) -> Bitboard;
}

pub struct NorthSouth;
impl SlideDirection for NorthSouth {
    fn positive_bitboard(&self, source: Square) -> Bitboard {
        bitboards::NORTH[source]
    }

    fn negative_bitboard(&self, source: Square) -> Bitboard {
        bitboards::SOUTH[source]
    }
}

pub struct EastWest;
impl SlideDirection for EastWest {
    fn positive_bitboard(&self, source: Square) -> Bitboard {
        bitboards::EAST[source]
    }

    fn negative_bitboard(&self, source: Square) -> Bitboard {
        bitboards::WEST[source]
    }
}

pub struct Diagonal;
impl SlideDirection for Diagonal {
    fn positive_bitboard(&self, source: Square) -> Bitboard {
        bitboards::POSITIVE_DIAGONALS[source]
    }

    fn negative_bitboard(&self, source: Square) -> Bitboard {
        bitboards::NEGATIVE_DIAGONALS[source]
    }
}

pub struct AntiDiagonal;
impl SlideDirection for AntiDiagonal {
    fn positive_bitboard(&self, source: Square) -> Bitboard {
        bitboards::POSITIVE_ANTIDIAGONALS[source]
    }

    fn negative_bitboard(&self, source: Square) -> Bitboard {
        bitboards::NEGATIVE_ANTIDIAGONALS[source]
    }
}
