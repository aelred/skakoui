use crate::File;
use crate::Rank;
use crate::Square;
use std::fmt;
use std::ops::BitAnd;
use std::ops::BitAndAssign;
use std::ops::BitOr;
use std::ops::BitOrAssign;
use std::ops::BitXor;
use std::ops::BitXorAssign;
use std::ops::Not;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Bitboard(u64);

impl Bitboard {
    #[inline]
    pub fn new(num: u64) -> Self {
        Bitboard(num)
    }

    #[inline]
    pub fn get(self, square: Square) -> bool {
        Self::from(square) & self != bitboards::EMPTY
    }

    #[inline]
    pub fn set(&mut self, square: Square) {
        *self |= Self::from(square);
    }

    #[inline]
    pub fn reset(&mut self, square: Square) {
        *self &= !Self::from(square);
    }

    /// Resets the bit at `from` and sets the bit at `to`.
    ///
    /// This method assumes that `from` is already set and `to` is already reset. If this is not
    /// the case, the result is undefined.
    #[inline]
    pub fn move_bit(&mut self, from: Square, to: Square) {
        debug_assert!(self.get(from));
        debug_assert!(!self.get(to));

        let move_board = Self::from(from) | Self::from(to);
        *self ^= move_board;
    }

    #[inline]
    #[must_use]
    pub fn shift_rank(self, offset: u32) -> Self {
        Bitboard(self.0.checked_shl(offset * 8).unwrap_or(0))
    }

    #[inline]
    #[must_use]
    pub fn shift_rank_neg(self, offset: u32) -> Self {
        Bitboard(self.0.checked_shr(offset * 8).unwrap_or(0))
    }

    #[inline]
    #[must_use]
    pub fn shift_file(self, offset: u32) -> Self {
        let mask = bitboards::FILES_FILLED[8 - offset as usize];
        Bitboard((self & mask).0 << offset)
    }

    #[inline]
    #[must_use]
    pub fn shift_file_neg(self, offset: u32) -> Self {
        let mask = !bitboards::FILES_FILLED[offset as usize];
        Bitboard((self & mask).0 >> offset)
    }

    /// Returns set squares in order from a1 to g8.
    #[inline]
    pub fn squares(self) -> SquareIterator {
        SquareIterator(self)
    }

    #[inline]
    pub fn first_set(self) -> Square {
        Square::from_index(self.index_of_lsb_set())
    }

    #[inline]
    pub fn last_set(self) -> Square {
        Square::from_index(self.index_of_msb_set())
    }

    #[inline]
    pub fn count(self) -> u8 {
        self.0.count_ones() as u8
    }

    #[inline]
    fn index_of_lsb_set(self) -> u8 {
        self.0.trailing_zeros() as u8
    }

    #[inline]
    fn index_of_msb_set(self) -> u8 {
        63 - self.0.leading_zeros() as u8
    }

    #[inline]
    fn reset_lsb(&mut self) {
        self.0 &= self.0 - 1;
    }
}

pub struct SquareIterator(Bitboard);

impl Iterator for SquareIterator {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Square> {
        if self.0 == bitboards::EMPTY {
            return None;
        }

        let lsb_set = self.0.index_of_lsb_set();

        self.0.reset_lsb();

        let square = Square::from_index(lsb_set as u8);

        Some(square)
    }
}

impl From<Square> for Bitboard {
    #[inline]
    fn from(square: Square) -> Self {
        Bitboard(1 << u64::from(square.to_index()))
    }
}

impl Not for Bitboard {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        Bitboard(!self.0)
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOr<&Bitboard> for Bitboard {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: &Self) -> Self {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Bitboard {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl fmt::Debug for Bitboard {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_fmt(format_args!("Bitboard({:b})", self.0))
    }
}

impl fmt::Display for Bitboard {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        let files_str: String = File::VALUES.iter().map(File::to_string).collect();
        f.write_fmt(format_args!("  {}\n", files_str))?;

        for rank in Rank::VALUES.iter().rev() {
            f.write_fmt(format_args!("{} ", rank))?;
            for file in &File::VALUES {
                let square = Square::new(*file, *rank);

                f.write_str(if self.get(square) { "█" } else { " " })?;
            }
            f.write_fmt(format_args!(" {}\n", rank))?;
        }

        f.write_fmt(format_args!("  {}", files_str))
    }
}

pub mod bitboards {
    #![allow(clippy::large_digit_groups)] // We have a lot of bitboards which clippy doesn't like

    use super::Bitboard;
    use crate::file::FileMap;
    use crate::File;
    use crate::Rank;
    use crate::RankMap;
    use crate::Square;
    use crate::SquareMap;
    use lazy_static::lazy_static;

    pub const EMPTY: Bitboard = Bitboard(0);

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

    lazy_static! {
        pub static ref FILES_FILLED: [Bitboard; 9] = {
            let fill_0 = EMPTY;
            let fill_1 = fill_0 | FILES[File::A];
            let fill_2 = fill_1 | FILES[File::B];
            let fill_3 = fill_2 | FILES[File::C];
            let fill_4 = fill_3 | FILES[File::D];
            let fill_5 = fill_4 | FILES[File::E];
            let fill_6 = fill_5 | FILES[File::F];
            let fill_7 = fill_6 | FILES[File::G];
            let fill_8 = fill_7 | FILES[File::H];
            [
                fill_0, fill_1, fill_2, fill_3, fill_4, fill_5, fill_6, fill_7, fill_8,
            ]
        };
        pub static ref RANKS_FILLED: [Bitboard; 9] = {
            let fill_0 = EMPTY;
            let fill_1 = fill_0 | RANKS[Rank::_1];
            let fill_2 = fill_1 | RANKS[Rank::_2];
            let fill_3 = fill_2 | RANKS[Rank::_3];
            let fill_4 = fill_3 | RANKS[Rank::_4];
            let fill_5 = fill_4 | RANKS[Rank::_5];
            let fill_6 = fill_5 | RANKS[Rank::_6];
            let fill_7 = fill_6 | RANKS[Rank::_7];
            let fill_8 = fill_7 | RANKS[Rank::_8];
            [
                fill_0, fill_1, fill_2, fill_3, fill_4, fill_5, fill_6, fill_7, fill_8,
            ]
        };
        pub static ref DIAGONALS: SquareMap<Bitboard> = {
            SquareMap::from(|square: Square| {
                let sq = square.to_index() as isize;
                let maindia =
                    0b_10000000_01000000_00100000_00010000_00001000_00000100_00000010_00000001;
                let diag = 8 * (sq & 7) - (sq & 56);
                let nort = -diag & (diag >> 31);
                let sout = diag & (-diag >> 31);
                Bitboard((maindia >> sout) << nort)
            })
        };
        pub static ref ANTIDIAGONALS: SquareMap<Bitboard> = {
            SquareMap::from(|square: Square| {
                let sq = square.to_index() as isize;
                let maindia =
                    0b_00000001_00000010_00000100_00001000_00010000_00100000_01000000_10000000;
                let diag = 56 - 8 * (sq & 7) - (sq & 56);
                let nort = -diag & (diag >> 31);
                let sout = diag & (-diag >> 31);
                Bitboard((maindia >> sout) << nort)
            })
        };
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::large_digit_groups)] // We have a lot of bitboards which clippy doesn't like

    use super::*;
    use pretty_assertions::assert_eq;
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

        bitboard = bitboard.shift_rank_neg(2);
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

        bitboard = bitboard.shift_file_neg(2);
        assert_eq!(bitboard, Bitboard(0b00000000_00101010));

        bitboard = bitboard.shift_file(8);
        assert_eq!(bitboard, bitboards::EMPTY);
    }
}
