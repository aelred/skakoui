use enum_map::Enum;

use crate::piece::Piece;
use crate::{
    bitboard::SquareIterator, bitboards, move_generation::Movement, Bishop, Bitboard, Board,
    BoardFlags, King, Knight, Move, Pawn, Player, Queen, Rook, Square,
};
use anyhow::Error;
use std::fmt;
use std::str::FromStr;

pub trait PieceType: Copy + Clone {
    fn value(self) -> PieceTypeV;

    /// Returns all moves for this piece when placed at the given square.
    ///
    /// Usually the same as [attacks] except also including moves that don't capture, like castling.
    fn movement(
        &self,
        source: Square,
        occupancy: Bitboard,
        player: impl Player,
        flags: BoardFlags,
    ) -> Bitboard {
        self.attacks(source, occupancy, player, flags)
            | self.other_moves(source, occupancy, player, flags)
    }

    /// Returns all squares this piece can attack when placed at the given square.
    ///
    /// This assumes that any occupied square can be captured - even though it might be friendly.
    /// Friendly captures are filtered out later.
    fn attacks(
        &self,
        source: Square,
        occupancy: Bitboard,
        player: impl Player,
        _: BoardFlags,
    ) -> Bitboard;

    /// Any non-attacking moves
    fn other_moves(&self, _: Square, _: Bitboard, _: impl Player, _: BoardFlags) -> Bitboard {
        bitboards::EMPTY
    }
}

#[derive(Debug, Eq, PartialEq, Enum, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum PieceTypeV {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceTypeV {
    pub fn to_char(self) -> char {
        match self {
            PieceTypeV::King => 'K',
            PieceTypeV::Queen => 'Q',
            PieceTypeV::Rook => 'R',
            PieceTypeV::Bishop => 'B',
            PieceTypeV::Knight => 'N',
            PieceTypeV::Pawn => 'P',
        }
    }
}

impl PieceType for PieceTypeV {
    fn value(self) -> PieceTypeV {
        self
    }

    fn movement(
        &self,
        source: Square,
        occupancy: Bitboard,
        player: impl Player,
        flags: BoardFlags,
    ) -> Bitboard {
        match self {
            PieceTypeV::King => King.movement(source, occupancy, player, flags),
            PieceTypeV::Queen => Queen.movement(source, occupancy, player, flags),
            PieceTypeV::Rook => Rook.movement(source, occupancy, player, flags),
            PieceTypeV::Bishop => Bishop.movement(source, occupancy, player, flags),
            PieceTypeV::Knight => Knight.movement(source, occupancy, player, flags),
            PieceTypeV::Pawn => Pawn.movement(source, occupancy, player, flags),
        }
    }

    fn attacks(
        &self,
        source: Square,
        occupancy: Bitboard,
        player: impl Player,
        flags: BoardFlags,
    ) -> Bitboard {
        match self {
            PieceTypeV::King => King.attacks(source, occupancy, player, flags),
            PieceTypeV::Queen => Queen.attacks(source, occupancy, player, flags),
            PieceTypeV::Rook => Rook.attacks(source, occupancy, player, flags),
            PieceTypeV::Bishop => Bishop.attacks(source, occupancy, player, flags),
            PieceTypeV::Knight => Knight.attacks(source, occupancy, player, flags),
            PieceTypeV::Pawn => Pawn.attacks(source, occupancy, player, flags),
        }
    }

    fn other_moves(
        &self,
        source: Square,
        occupancy: Bitboard,
        player: impl Player,
        flags: BoardFlags,
    ) -> Bitboard {
        match self {
            PieceTypeV::King => King.other_moves(source, occupancy, player, flags),
            PieceTypeV::Queen => Queen.other_moves(source, occupancy, player, flags),
            PieceTypeV::Rook => Rook.other_moves(source, occupancy, player, flags),
            PieceTypeV::Bishop => Bishop.other_moves(source, occupancy, player, flags),
            PieceTypeV::Knight => Knight.other_moves(source, occupancy, player, flags),
            PieceTypeV::Pawn => Pawn.other_moves(source, occupancy, player, flags),
        }
    }
}

impl fmt::Display for PieceTypeV {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_str(&self.to_fen())
    }
}

impl FromStr for PieceTypeV {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_fen(s)
    }
}

/// Type-level representation of [PieceType].
pub trait PieceTypeT: PieceType + Sized + Default {}

pub struct MovesIter<P, PT, M> {
    occupancy: Bitboard,
    mask: Bitboard,
    sources: SquareIterator,
    source: Square,
    targets: SquareIterator,
    piece: Piece<P, PT>,
    movement: M,
    flags: BoardFlags,
}

impl<P: Player, PT: PieceType, M: Movement> MovesIter<P, PT, M> {
    pub(crate) fn new(board: &Board, piece: Piece<P, PT>, movement: M, mask: Bitboard) -> Self {
        // arbitrary source square with no targets to avoid empty case
        let source = Square::A1;
        let targets = bitboards::EMPTY.squares();
        Self {
            occupancy: board.occupancy(),
            mask,
            sources: board.bitboard_piece(piece).squares(),
            source,
            targets,
            piece,
            movement,
            flags: board.flags(),
        }
    }
}

impl<P: Player, PT: PieceType, M: Movement> Iterator for MovesIter<P, PT, M> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        let target = loop {
            match self.targets.next() {
                None => {
                    self.source = self.sources.next()?;
                    // TODO: use right movement here
                    let targets = self.movement.movement(
                        &self.piece.piece_type,
                        self.source,
                        self.occupancy,
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
