use crate::Square;
use std::fmt;
use std::ops::BitAnd;
use std::ops::BitAndAssign;
use std::ops::BitOr;
use std::ops::BitOrAssign;
use std::ops::BitXorAssign;
use std::ops::Not;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Bitboard(u64);

impl Bitboard {
    pub fn new(num: u64) -> Self {
        Bitboard(num)
    }

    pub fn get(self, square: Square) -> bool {
        Self::from(square) & self != bitboards::EMPTY
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
        debug_assert!(self.get(from));
        debug_assert!(!self.get(to));

        let move_board = Self::from(from) | Self::from(to);
        *self ^= move_board;
    }

    #[must_use]
    pub fn shift_rank(self, offset: u32) -> Self {
        Bitboard(self.0.checked_shl(offset * 8).unwrap_or(0))
    }

    #[must_use]
    pub fn shift_rank_neg(self, offset: u32) -> Self {
        Bitboard(self.0.checked_shr(offset * 8).unwrap_or(0))
    }

    #[must_use]
    pub fn shift_file(self, offset: u32) -> Self {
        let mask = bitboards::FILES_FILLED[8 - offset as usize];
        Bitboard((self & mask).0 << offset)
    }

    #[must_use]
    pub fn shift_file_neg(self, offset: u32) -> Self {
        let mask = !bitboards::FILES_FILLED[offset as usize];
        Bitboard((self & mask).0 >> offset)
    }

    pub fn squares(self) -> impl Iterator<Item = Square> {
        SquareIterator(self)
    }

    fn index_of_lsb_set(self) -> u32 {
        self.0.trailing_zeros()
    }

    fn unset_lsb(&mut self) {
        self.0 &= self.0 - 1;
    }
}

struct SquareIterator(Bitboard);

impl Iterator for SquareIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        if self.0 == bitboards::EMPTY {
            return None;
        }

        let lsb_set = self.0.index_of_lsb_set();

        self.0.unset_lsb();

        let square = Square::from_index(lsb_set);

        Some(square)
    }
}

impl From<Square> for Bitboard {
    fn from(square: Square) -> Self {
        let file = bitboards::FILES[square.file()];
        let rank = bitboards::RANKS[square.rank()];
        file & rank
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self {
        Bitboard(!self.0)
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOr<&Bitboard> for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: &Self) -> Self {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl fmt::Debug for Bitboard {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_fmt(format_args!("Bitboard({:b})", self.0))
    }
}

pub mod bitboards {
    #![allow(clippy::large_digit_groups)] // We have a lot of bitboards which clippy doesn't like

    use super::Bitboard;
    use crate::File;
    use crate::PieceType;
    use crate::Player;
    use crate::Rank;
    use enum_map::enum_map;
    use enum_map::EnumMap;
    use lazy_static::lazy_static;

    pub const EMPTY: Bitboard = Bitboard(0);

    lazy_static! {
        pub static ref START_POSITIONS: EnumMap<Player, EnumMap<PieceType, Bitboard>> = enum_map! {
            Player::White => enum_map! {
                PieceType::King => {
                    Bitboard(0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010000)
                }
                PieceType::Queen => {
                    Bitboard(0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001000)
                }
                PieceType::Rook => {
                    Bitboard(0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000001)
                }
                PieceType::Bishop => {
                    Bitboard(0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00100100)
                }
                PieceType::Knight => {
                    Bitboard(0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_01000010)
                }
                PieceType::Pawn => {
                    Bitboard(0b_00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000)
                }
            },
            Player::Black => enum_map! {
                PieceType::King => {
                    Bitboard(0b_00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
                }
                PieceType::Queen => {
                    Bitboard(0b_00001000_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
                }
                PieceType::Rook => {
                    Bitboard(0b_10000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
                }
                PieceType::Bishop => {
                    Bitboard(0b_00100100_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
                }
                PieceType::Knight => {
                    Bitboard(0b_01000010_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
                }
                PieceType::Pawn => {
                    Bitboard(0b_00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000)
                }
            }
        };
        pub static ref FILES: EnumMap<File, Bitboard> = {
            let b = 0b_00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001;
            EnumMap::from(|file: File| Bitboard(b << file.to_index()))
        };
        pub static ref RANKS: EnumMap<Rank, Bitboard> = {
            let b = 0b_11111111;
            EnumMap::from(|rank: Rank| Bitboard(b << (rank.to_index() * Rank::VALUES.len())))
        };
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PieceType;
    use crate::Player;
    use std::collections::HashSet;

    #[test]
    fn can_create_bitboard_from_square() {
        assert_eq!(Bitboard::from(Square::B2), Bitboard(0b00000010_00000000));
    }

    #[test]
    fn can_get_an_iterator_of_squares_from_bitboard() {
        let bitboard = (*bitboards::START_POSITIONS)[Player::White][PieceType::Pawn];

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
        let mut bitboard = Bitboard(0b11000000);

        bitboard.reset(Square::H1);

        assert_eq!(bitboard, Bitboard(0b01000000));
    }

    #[test]
    fn can_move_bit_on_bitboard() {
        let mut bitboard = Bitboard(0b10000001);

        bitboard.move_bit(Square::A1, Square::C1);

        assert_eq!(bitboard, Bitboard(0b10000100));
    }

    #[test]
    fn can_shift_rank_on_bitboard() {
        let mut bitboard = Bitboard(0b11111111_01010101);

        bitboard = bitboard.shift_rank(0);
        assert_eq!(bitboard, Bitboard(0b11111111_01010101));

        bitboard = bitboard.shift_rank(1);
        assert_eq!(bitboard, Bitboard(0b11111111_01010101_00000000));

        bitboard = bitboard.shift_rank_neg(2);
        assert_eq!(bitboard, Bitboard(0b11111111));

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
