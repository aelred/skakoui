use crate::{
    bitboard::SquareIterator, bitboards, move_generation::Movement, Bitboard, Board, BoardFlags,
    Move, Piece, PieceType, Player, Square,
};

#[derive(Default)]
pub struct PieceT<P, PT> {
    pub player: P,
    pub piece_type: PT,
}

impl<P: Player, PT: PieceTypeT> PieceT<P, PT> {
    pub(crate) fn new(player: P, piece_type: PT) -> Self {
        Self { player, piece_type }
    }

    pub fn value(&self) -> Piece {
        Piece::new(self.player.value(), self.piece_type.value())
    }
}

/// Type-level representation of [PieceType].
pub trait PieceTypeT: Sized + Default {
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

pub struct MovesIter<P, PT, M> {
    occupancy: Bitboard,
    mask: Bitboard,
    sources: SquareIterator,
    source: Square,
    targets: SquareIterator,
    piece: PieceT<P, PT>,
    movement: M,
    flags: BoardFlags,
}

impl<P: Player, PT: PieceTypeT, M: Movement> MovesIter<P, PT, M> {
    pub(crate) fn new(board: &Board, piece: PieceT<P, PT>, movement: M, mask: Bitboard) -> Self {
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
            movement,
            flags: board.flags(),
        }
    }
}

impl<P: Player, PT: PieceTypeT, M: Movement> Iterator for MovesIter<P, PT, M> {
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
