use crate::file::File;
use crate::Rank;
use enum_map::Enum;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Enum)]
pub enum Square {
    A1 = 0,
    B1 = 1,
    C1 = 2,
    D1 = 3,
    E1 = 4,
    F1 = 5,
    G1 = 6,
    H1 = 7,
    A2 = 8,
    B2 = 9,
    C2 = 10,
    D2 = 11,
    E2 = 12,
    F2 = 13,
    G2 = 14,
    H2 = 15,
    A3 = 16,
    B3 = 17,
    C3 = 18,
    D3 = 19,
    E3 = 20,
    F3 = 21,
    G3 = 22,
    H3 = 23,
    A4 = 24,
    B4 = 25,
    C4 = 26,
    D4 = 27,
    E4 = 28,
    F4 = 29,
    G4 = 30,
    H4 = 31,
    A5 = 32,
    B5 = 33,
    C5 = 34,
    D5 = 35,
    E5 = 36,
    F5 = 37,
    G5 = 38,
    H5 = 39,
    A6 = 40,
    B6 = 41,
    C6 = 42,
    D6 = 43,
    E6 = 44,
    F6 = 45,
    G6 = 46,
    H6 = 47,
    A7 = 48,
    B7 = 49,
    C7 = 50,
    D7 = 51,
    E7 = 52,
    F7 = 53,
    G7 = 54,
    H7 = 55,
    A8 = 56,
    B8 = 57,
    C8 = 58,
    D8 = 59,
    E8 = 60,
    F8 = 61,
    G8 = 62,
    H8 = 63,
}

impl Square {
    pub fn new(file: File, rank: Rank) -> Self {
        let index = file.to_index() + rank.to_index() * File::VALUES.len();
        Self::from_index(index)
    }

    #[inline]
    pub fn from_index(index: usize) -> Self {
        Enum::<usize>::from_usize(index)
    }

    #[inline]
    pub fn to_index(self) -> usize {
        self as usize
    }

    #[inline]
    pub fn file(self) -> File {
        File::from_index(self.to_index() % 8)
    }

    #[inline]
    pub fn rank(self) -> Rank {
        Rank::from_index(self.to_index() / 8)
    }

    #[inline]
    pub fn shift_rank(self, offset: isize) -> Self {
        Self::from_index((self.to_index() as isize + offset * 8) as usize)
    }

    #[inline]
    pub fn shift_file(self, offset: isize) -> Self {
        Self::from_index((self.to_index() as isize + offset) as usize)
    }

    #[inline]
    pub fn color(self) -> SquareColor {
        // bitwise magic here
        if ((9 * self.to_index()) & 8) == 0 {
            SquareColor::Black
        } else {
            SquareColor::White
        }
    }
}

impl fmt::Debug for Square {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_fmt(format_args!("Square::{:?}{}", self.file(), self.rank()))
    }
}

impl fmt::Display for Square {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_fmt(format_args!("{}{}", self.file(), self.rank()))
    }
}

impl FromStr for Square {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Box<dyn Error>> {
        let file = s.get(..1).ok_or("couldn't index string")?.parse()?;
        let rank = s.get(1..).ok_or("coudln't index string")?.parse()?;
        Ok(Square::new(file, rank))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SquareColor {
    Black,
    White,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::File::*;
    use crate::Rank::*;

    #[test]
    fn files_are_ordered_letters() {
        assert!(A < B);
        assert!(B < C);
        assert!(C < D);
        assert!(D < E);
        assert!(E < F);
        assert!(F < G);
        assert!(G < H);
    }

    #[test]
    fn ranks_are_ordered_numbers() {
        assert!(_1 < _2);
        assert!(_2 < _3);
        assert!(_3 < _4);
        assert!(_4 < _5);
        assert!(_5 < _6);
        assert!(_6 < _7);
        assert!(_7 < _8);
    }

    #[test]
    fn ranks_can_be_displayed() {
        assert_eq!("1", _1.to_string());
        assert_eq!("8", _8.to_string());
    }

    #[test]
    fn files_can_be_displayed() {
        assert_eq!("a", A.to_string());
        assert_eq!("d", D.to_string());
    }

    #[test]
    fn squares_can_be_displayed() {
        assert_eq!("g5", Square::G5.to_string());
    }

    #[test]
    fn can_get_color_of_square() {
        assert_eq!(Square::A1.color(), SquareColor::Black);
        assert_eq!(Square::B1.color(), SquareColor::White);
        assert_eq!(Square::B2.color(), SquareColor::Black);
        assert_eq!(Square::B1.color(), SquareColor::White);
    }

    #[test]
    fn can_create_square_from_index() {
        assert_eq!(Square::from_index(0), Square::A1);
        assert_eq!(Square::from_index(1), Square::B1);
        assert_eq!(Square::from_index(10), Square::C2);
        assert_eq!(Square::from_index(63), Square::H8);
    }

    #[test]
    fn can_shift_rank_of_square() {
        assert_eq!(Square::C3.shift_rank(0), Square::C3);
        assert_eq!(Square::C3.shift_rank(1), Square::C4);
        assert_eq!(Square::C3.shift_rank(-1), Square::C2);
        assert_eq!(Square::C3.shift_rank(-2), Square::C1);
        assert_eq!(Square::C3.shift_rank(5), Square::C8);
    }

    #[test]
    #[should_panic]
    fn can_not_shift_rank_south_of_board() {
        Square::C3.shift_rank(-3);
    }

    #[test]
    #[should_panic]
    fn can_not_shift_rank_north_of_board() {
        Square::C3.shift_rank(6);
    }

    #[test]
    fn can_shift_file_of_square() {
        assert_eq!(Square::C3.shift_file(0), Square::C3);
        assert_eq!(Square::C3.shift_file(1), Square::D3);
        assert_eq!(Square::C3.shift_file(-1), Square::B3);
        assert_eq!(Square::C3.shift_file(-2), Square::A3);
        assert_eq!(Square::C3.shift_file(5), Square::H3);
    }
}
