use crate::Piece;
use crate::PieceType;
use crate::Player;
use enum_map::EnumMap;
use std::ops::Index;
use std::ops::IndexMut;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct PieceMap<T>(EnumMap<Player, EnumMap<PieceType, T>>);

impl<T> PieceMap<T> {
    pub fn for_player(&self, player: Player) -> &EnumMap<PieceType, T> {
        &self.0[player]
    }

    pub fn iter(&self) -> impl Iterator<Item = (Piece, &T)> {
        self.0.iter().flat_map(|(player, inner)| {
            inner
                .iter()
                .map(move |(piece_type, item)| (Piece::new(player, piece_type), item))
        })
    }
}

impl<F: FnMut(Piece) -> T, T> From<F> for PieceMap<T> {
    fn from(mut f: F) -> Self {
        PieceMap(EnumMap::from(|player| {
            EnumMap::from(|piece_type| f(Piece::new(player, piece_type)))
        }))
    }
}

impl<T> Index<Piece> for PieceMap<T> {
    type Output = T;

    #[inline]
    fn index(&self, piece: Piece) -> &T {
        &self.0[piece.player()][piece.piece_type()]
    }
}

impl<T> IndexMut<Piece> for PieceMap<T> {
    #[inline]
    fn index_mut(&mut self, piece: Piece) -> &mut T {
        &mut self.0[piece.player()][piece.piece_type()]
    }
}
