use crate::File;
use crate::Rank;
use crate::Square;
use std::fmt;
use std::ops::Index;

// TODO: it would be nice if bitboards were in the same order as FEN
#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Default,
    derive_more::Not,
    derive_more::BitAnd,
    derive_more::BitAndAssign,
    derive_more::BitOr,
    derive_more::BitOrAssign,
    derive_more::BitXor,
    derive_more::BitXorAssign,
)]
pub struct Bitboard(u64);

impl Bitboard {
    pub const fn new(num: u64) -> Self {
        Bitboard(num)
    }

    pub fn set(&mut self, square: Square) {
        *self |= Self::from(square);
    }

    pub fn reset(&mut self, square: Square) {
        *self &= !Self::from(square);
    }

    /// Resets the bit at `from` and sets the bit at `to`.
    ///
    /// This method assumes that `from` is already set and `to` is already reset. If this is not
    /// the case, the result is undefined.
    pub fn move_bit(&mut self, from: Square, to: Square) {
        debug_assert!(self[from], "{:?} should be set: \n{}", from, self);
        debug_assert!(!self[to], "{:?} should be unset: \n{}", to, self);

        let move_board = Self::from(from) | Self::from(to);
        *self ^= move_board;
    }

    #[must_use]
    #[inline]
    pub fn shift_rank(self, offset: i32) -> Self {
        let result = if offset > 0 {
            self.0.checked_shl((offset * 8) as u32)
        } else {
            self.0.checked_shr((-offset * 8) as u32)
        };
        Bitboard(result.unwrap_or(0))
    }

    #[must_use]
    #[inline]
    pub fn shift_file(self, offset: i32) -> Self {
        if offset > 0 {
            let mask = bitboards::FILES_FILLED[8 - offset as usize];
            Bitboard((self & mask).0 << offset)
        } else {
            let mask = !bitboards::FILES_FILLED[-offset as usize];
            Bitboard((self & mask).0 >> -offset)
        }
    }

    /// Returns set squares in order from a1 to g8.
    pub fn squares(self) -> SquareIterator {
        SquareIterator(self)
    }

    pub fn first_set(self) -> Square {
        Square::from_index(self.0.trailing_zeros() as u8)
    }

    pub fn last_set(self) -> Square {
        Square::from_index(63 - self.0.leading_zeros() as u8)
    }

    pub fn count(self) -> u8 {
        self.0.count_ones() as u8
    }

    pub const fn reverse(self) -> Self {
        Self(self.0.swap_bytes())
    }

    pub fn powerset(self) -> impl Iterator<Item = Bitboard> {
        let mut x = 0u64;

        // idk how this works, but it does
        std::iter::from_fn(move || {
            let result = x;
            x = x.wrapping_sub(self.0) & self.0;
            if x != 0 {
                Some(Bitboard(result))
            } else {
                None
            }
        }).chain(std::iter::once(self))
    }
}

impl Index<Square> for Bitboard {
    type Output = bool;

    #[inline]
    fn index(&self, index: Square) -> &Self::Output {
        if Self::from(index) & *self != bitboards::EMPTY {
            &TRUE
        } else {
            &FALSE
        }
    }
}

// GLOBAL constants that we can borrow in Index<Square> for Bitboard
const TRUE: bool = true;
const FALSE: bool = false;

pub struct SquareIterator(Bitboard);

impl Iterator for SquareIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        if self.0 == bitboards::EMPTY {
            return None;
        }

        let first_square = self.0.first_set();
        self.0.0 &= self.0.0 - 1;
        Some(first_square)
    }
}

impl From<Square> for Bitboard {
    fn from(square: Square) -> Self {
        Bitboard(1 << u64::from(square.to_index()))
    }
}

impl From<Bitboard> for u64 {
    fn from(bitboard: Bitboard) -> Self {
        bitboard.0
    }
}

impl fmt::Debug for Bitboard {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        writeln!(f, "bitboard! {{")?;

        for rank in Rank::VALUES.iter().rev() {
            write!(f, "\t")?;
            for file in File::VALUES.iter() {
                let square = Square::new(*file, *rank);
                let char = if self[square] { 'X' } else { '.' };
                write!(f, "{} ", char)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "}}")
    }
}

impl fmt::Display for Bitboard {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        let files_str: String = File::VALUES.iter().map(|f| format!("{} ", f)).collect();
        f.write_fmt(format_args!("  {}\n", files_str))?;

        for rank in Rank::VALUES.iter().rev() {
            f.write_fmt(format_args!("{} ", rank))?;
            for file in &File::VALUES {
                let square = Square::new(*file, *rank);

                f.write_str(if self[square] { "X " } else { ". " })?;
            }
            f.write_fmt(format_args!("{}\n", rank))?;
        }

        f.write_fmt(format_args!("  {}", files_str))
    }
}

/// Example usage:
///
/// ```
/// use skakoui::bitboard;
///
/// bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . X . . X . .
///     . . . . . . . .
///     . . . . . . . .
///     . X . . . . X .
///     . . X X X X . .
///     . . . . . . . .
/// };
/// ```
#[macro_export]
macro_rules! bitboard {
    ($($square:tt)*) => {
    {
        let mut bb = $crate::bitboards::EMPTY;
        let mut square = $crate::Square::A8;
        let bits = vec![$(stringify!($square)),*];

        for bit in bits {
            if bit != "." {
                bb.set(square);
            }
            if (square.file() != $crate::File::H) {
                square = square.shift_file(1);
            } else if (square.rank() > $crate::Rank::_1){
                square = $crate::Square::new($crate::File::A, square.rank() - 1);
            }
        }

        bb
        }
    };
}

pub mod bitboards {
    #![allow(clippy::large_digit_groups)] // We have a lot of bitboards which clippy doesn't like

    use super::Bitboard;
    use crate::file::FileMap;
    use crate::RankMap;
    use crate::Square;
    use crate::SquareMap;
    use lazy_static::lazy_static;

    pub const EMPTY: Bitboard = Bitboard(0);
    pub const FULL: Bitboard = Bitboard(u64::MAX);

    pub const FILES: FileMap<Bitboard> = FileMap::new([
        Bitboard(0b_00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001),
        Bitboard(0b_00000010_00000010_00000010_00000010_00000010_00000010_00000010_00000010),
        Bitboard(0b_00000100_00000100_00000100_00000100_00000100_00000100_00000100_00000100),
        Bitboard(0b_00001000_00001000_00001000_00001000_00001000_00001000_00001000_00001000),
        Bitboard(0b_00010000_00010000_00010000_00010000_00010000_00010000_00010000_00010000),
        Bitboard(0b_00100000_00100000_00100000_00100000_00100000_00100000_00100000_00100000),
        Bitboard(0b_01000000_01000000_01000000_01000000_01000000_01000000_01000000_01000000),
        Bitboard(0b_10000000_10000000_10000000_10000000_10000000_10000000_10000000_10000000),
    ]);

    pub const RANKS: RankMap<Bitboard> = RankMap::new([
        Bitboard(0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_11111111),
        Bitboard(0b_00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000),
        Bitboard(0b_00000000_00000000_00000000_00000000_00000000_11111111_00000000_00000000),
        Bitboard(0b_00000000_00000000_00000000_00000000_11111111_00000000_00000000_00000000),
        Bitboard(0b_00000000_00000000_00000000_11111111_00000000_00000000_00000000_00000000),
        Bitboard(0b_00000000_00000000_11111111_00000000_00000000_00000000_00000000_00000000),
        Bitboard(0b_00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000),
        Bitboard(0b_11111111_00000000_00000000_00000000_00000000_00000000_00000000_00000000),
    ]);

    pub const FILES_FILLED: [Bitboard; 9] = [
        Bitboard(0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000),
        Bitboard(0b_00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001),
        Bitboard(0b_00000011_00000011_00000011_00000011_00000011_00000011_00000011_00000011),
        Bitboard(0b_00000111_00000111_00000111_00000111_00000111_00000111_00000111_00000111),
        Bitboard(0b_00001111_00001111_00001111_00001111_00001111_00001111_00001111_00001111),
        Bitboard(0b_00011111_00011111_00011111_00011111_00011111_00011111_00011111_00011111),
        Bitboard(0b_00111111_00111111_00111111_00111111_00111111_00111111_00111111_00111111),
        Bitboard(0b_01111111_01111111_01111111_01111111_01111111_01111111_01111111_01111111),
        Bitboard(0b_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111),
    ];

    pub const RANKS_FILLED: [Bitboard; 9] = [
        Bitboard(0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000),
        Bitboard(0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_11111111),
        Bitboard(0b_00000000_00000000_00000000_00000000_00000000_00000000_11111111_11111111),
        Bitboard(0b_00000000_00000000_00000000_00000000_00000000_11111111_11111111_11111111),
        Bitboard(0b_00000000_00000000_00000000_00000000_11111111_11111111_11111111_11111111),
        Bitboard(0b_00000000_00000000_00000000_11111111_11111111_11111111_11111111_11111111),
        Bitboard(0b_00000000_00000000_11111111_11111111_11111111_11111111_11111111_11111111),
        Bitboard(0b_00000000_11111111_11111111_11111111_11111111_11111111_11111111_11111111),
        Bitboard(0b_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111),
    ];

    pub const DIAGONAL: Bitboard =
        Bitboard(0b_10000000_01000000_00100000_00010000_00001000_00000100_00000010_00000001);

    pub const ANTIDIAGONAL: Bitboard =
        Bitboard(0b_00000001_00000010_00000100_00001000_00010000_00100000_01000000_10000000);

    pub const BORDER: Bitboard =
        Bitboard(0b_11111111_10000001_10000001_10000001_10000001_10000001_10000001_11111111);

    lazy_static! {
        pub static ref DIAGONALS: SquareMap<Bitboard> = SquareMap::from(|square: Square| {
            let sq = square.to_index() as isize;
            let diag = 8 * (sq & 7) - (sq & 56);
            let nort = -diag & (diag >> 31);
            let sout = diag & (-diag >> 31);
            Bitboard((DIAGONAL.0 >> sout) << nort)
        });
        pub static ref ANTIDIAGONALS: SquareMap<Bitboard> = SquareMap::from(|square: Square| {
            let sq = square.to_index() as isize;
            let diag = 56 - 8 * (sq & 7) - (sq & 56);
            let nort = -diag & (diag >> 31);
            let sout = diag & (-diag >> 31);
            Bitboard((ANTIDIAGONAL.0 >> sout) << nort)
        });
        pub static ref NORTH: SquareMap<Bitboard> =
            SquareMap::from(|s| FILES[s.file()] & !RANKS_FILLED[s.rank().to_index() as usize + 1]);
        pub static ref SOUTH: SquareMap<Bitboard> =
            SquareMap::from(|s| FILES[s.file()] & RANKS_FILLED[s.rank().to_index() as usize]);
        pub static ref EAST: SquareMap<Bitboard> =
            SquareMap::from(|s| RANKS[s.rank()] & !FILES_FILLED[s.file().to_index() as usize + 1]);
        pub static ref WEST: SquareMap<Bitboard> =
            SquareMap::from(|s| RANKS[s.rank()] & FILES_FILLED[s.file().to_index() as usize]);
        pub static ref NORTH_EAST: SquareMap<Bitboard> =
            SquareMap::from(|s| DIAGONALS[s] & !FILES_FILLED[s.file().to_index() as usize + 1]);
        pub static ref SOUTH_WEST: SquareMap<Bitboard> =
            SquareMap::from(|s| DIAGONALS[s] & FILES_FILLED[s.file().to_index() as usize]);
        pub static ref NORTH_WEST: SquareMap<Bitboard> =
            SquareMap::from(|s| ANTIDIAGONALS[s] & !RANKS_FILLED[s.rank().to_index() as usize + 1]);
        pub static ref SOUTH_EAST: SquareMap<Bitboard> =
            SquareMap::from(|s| ANTIDIAGONALS[s] & RANKS_FILLED[s.rank().to_index() as usize]);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::large_digit_groups)] // We have a lot of bitboards which clippy doesn't like

    use super::*;
    use std::collections::HashSet;

    #[test]
    fn can_create_bitboard_from_square() {
        assert_eq!(Bitboard::from(Square::B2), Bitboard(0b00000010_00000000));
    }

    #[test]
    fn can_get_an_iterator_of_squares_from_bitboard() {
        let bitboard = Bitboard(0b_11111111_00000000);

        let squares: HashSet<Square> = bitboard.squares().collect();

        let expected_squares: HashSet<Square> = [
            Square::A2,
            Square::B2,
            Square::C2,
            Square::D2,
            Square::E2,
            Square::F2,
            Square::G2,
            Square::H2,
        ]
        .iter()
        .cloned()
        .collect();

        assert_eq!(squares, expected_squares);
    }

    #[test]
    fn can_set_bit_on_bitboard() {
        let mut bitboard = Bitboard(0b0_11000000);

        bitboard.set(Square::A2);

        assert_eq!(bitboard, Bitboard(0b1_11000000));
    }

    #[test]
    fn can_reset_bit_on_bitboard() {
        let mut bitboard = Bitboard(0b_11000000);

        bitboard.reset(Square::H1);

        assert_eq!(bitboard, Bitboard(0b_01000000));
    }

    #[test]
    fn can_move_bit_on_bitboard() {
        let mut bitboard = Bitboard(0b_10000001);

        bitboard.move_bit(Square::A1, Square::C1);

        assert_eq!(bitboard, Bitboard(0b_10000100));
    }

    #[test]
    fn can_shift_rank_on_bitboard() {
        let mut bitboard = Bitboard(0b11111111_01010101);

        bitboard = bitboard.shift_rank(0);
        assert_eq!(bitboard, Bitboard(0b11111111_01010101));

        bitboard = bitboard.shift_rank(1);
        assert_eq!(bitboard, Bitboard(0b11111111_01010101_00000000));

        bitboard = bitboard.shift_rank(-2);
        assert_eq!(bitboard, Bitboard(0b_11111111));

        bitboard = bitboard.shift_rank(8);
        assert_eq!(bitboard, bitboards::EMPTY);
    }

    #[test]
    fn can_shift_file_on_bitboard() {
        let mut bitboard = Bitboard(0b10000000_01010101);

        bitboard = bitboard.shift_file(0);
        assert_eq!(bitboard, Bitboard(0b10000000_01010101));

        bitboard = bitboard.shift_file(1);
        assert_eq!(bitboard, Bitboard(0b00000000_10101010));

        bitboard = bitboard.shift_file(-2);
        assert_eq!(bitboard, Bitboard(0b00000000_00101010));

        bitboard = bitboard.shift_file(8);
        assert_eq!(bitboard, bitboards::EMPTY);
    }

    #[test]
    fn can_get_powerset_of_bitboard() {
        let bitboard = Bitboard(0b10101);

        let powerset: HashSet<Bitboard> = bitboard.powerset().collect();

        let mut expected = HashSet::new();
        expected.insert(Bitboard(0b10101));
        expected.insert(Bitboard(0b10100));
        expected.insert(Bitboard(0b10001));
        expected.insert(Bitboard(0b10000));
        expected.insert(Bitboard(0b00101));
        expected.insert(Bitboard(0b00100));
        expected.insert(Bitboard(0b00001));
        expected.insert(Bitboard(0b00000));

        assert_eq!(powerset, expected);
    }
}
