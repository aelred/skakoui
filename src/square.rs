use crate::file::File;
use crate::Rank;
use anyhow::anyhow;
use std::borrow::Borrow;
use std::fmt;
use std::hash::Hash;
use std::ops::Index;
use std::ops::IndexMut;
use std::str::FromStr;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Square(u8);

impl Square {
    pub const A1: Square = Square(0);
    pub const B1: Square = Square(1);
    pub const C1: Square = Square(2);
    pub const D1: Square = Square(3);
    pub const E1: Square = Square(4);
    pub const F1: Square = Square(5);
    pub const G1: Square = Square(6);
    pub const H1: Square = Square(7);
    pub const A2: Square = Square(8);
    pub const B2: Square = Square(9);
    pub const C2: Square = Square(10);
    pub const D2: Square = Square(11);
    pub const E2: Square = Square(12);
    pub const F2: Square = Square(13);
    pub const G2: Square = Square(14);
    pub const H2: Square = Square(15);
    pub const A3: Square = Square(16);
    pub const B3: Square = Square(17);
    pub const C3: Square = Square(18);
    pub const D3: Square = Square(19);
    pub const E3: Square = Square(20);
    pub const F3: Square = Square(21);
    pub const G3: Square = Square(22);
    pub const H3: Square = Square(23);
    pub const A4: Square = Square(24);
    pub const B4: Square = Square(25);
    pub const C4: Square = Square(26);
    pub const D4: Square = Square(27);
    pub const E4: Square = Square(28);
    pub const F4: Square = Square(29);
    pub const G4: Square = Square(30);
    pub const H4: Square = Square(31);
    pub const A5: Square = Square(32);
    pub const B5: Square = Square(33);
    pub const C5: Square = Square(34);
    pub const D5: Square = Square(35);
    pub const E5: Square = Square(36);
    pub const F5: Square = Square(37);
    pub const G5: Square = Square(38);
    pub const H5: Square = Square(39);
    pub const A6: Square = Square(40);
    pub const B6: Square = Square(41);
    pub const C6: Square = Square(42);
    pub const D6: Square = Square(43);
    pub const E6: Square = Square(44);
    pub const F6: Square = Square(45);
    pub const G6: Square = Square(46);
    pub const H6: Square = Square(47);
    pub const A7: Square = Square(48);
    pub const B7: Square = Square(49);
    pub const C7: Square = Square(50);
    pub const D7: Square = Square(51);
    pub const E7: Square = Square(52);
    pub const F7: Square = Square(53);
    pub const G7: Square = Square(54);
    pub const H7: Square = Square(55);
    pub const A8: Square = Square(56);
    pub const B8: Square = Square(57);
    pub const C8: Square = Square(58);
    pub const D8: Square = Square(59);
    pub const E8: Square = Square(60);
    pub const F8: Square = Square(61);
    pub const G8: Square = Square(62);
    pub const H8: Square = Square(63);

    pub fn all() -> impl Iterator<Item = Self> {
        (0..64).map(Square::from_index)
    }

    pub fn new(file: File, rank: Rank) -> Self {
        let index = file.to_index() + rank.to_index() * 8;
        Self::from_index(index)
    }

    pub fn from_index(index: u8) -> Self {
        Square(index)
    }

    pub fn to_index(self) -> u8 {
        self.0
    }

    pub fn file(self) -> File {
        File::from_index(self.to_index() % 8)
    }

    pub fn rank(self) -> Rank {
        Rank::from_index(self.to_index() / 8)
    }

    pub fn shift_rank(self, offset: i8) -> Self {
        Self::from_index((self.to_index() as i8 + offset * 8) as u8)
    }

    pub fn shift_file(self, offset: i8) -> Self {
        Self::from_index((self.to_index() as i8 + offset) as u8)
    }

    pub fn color(self) -> SquareColor {
        // bitwise magic here
        if ((9 * u32::from(self.to_index())) & 8) == 0 {
            SquareColor::Black
        } else {
            SquareColor::White
        }
    }
}

impl fmt::Debug for Square {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}{}", self.file(), self.rank()))
    }
}

impl fmt::Display for Square {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_fmt(format_args!("{}{}", self.file(), self.rank()))
    }
}

impl FromStr for Square {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, anyhow::Error> {
        let file = s
            .get(..1)
            .ok_or_else(|| anyhow!("couldn't index string"))?
            .parse()?;
        let rank = s
            .get(1..)
            .ok_or_else(|| anyhow!("couldn't index string"))?
            .parse()?;
        Ok(Square::new(file, rank))
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct SquareMap<T>([T; 64]);

impl<T> SquareMap<T> {
    pub const fn new(values: [T; 64]) -> SquareMap<T> {
        SquareMap(values)
    }

    pub fn from<F: Fn(Square) -> T>(f: F) -> Self {
        let arr = array_init::array_init(|i| f(Square(i as u8)));
        SquareMap::new(arr)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Square, &T)> {
        self.0
            .iter()
            .enumerate()
            .map(|(index, item)| (Square(index as u8), item))
    }
}

impl<T, S: Borrow<Square>> Index<S> for SquareMap<T> {
    type Output = T;

    fn index(&self, square: S) -> &T {
        &self.0[square.borrow().to_index() as usize]
    }
}

impl<T, S: Borrow<Square>> IndexMut<S> for SquareMap<T> {
    fn index_mut(&mut self, square: S) -> &mut T {
        &mut self.0[square.borrow().to_index() as usize]
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

    #[test]
    fn files_are_ordered_letters() {
        assert!(File::A < File::B);
        assert!(File::B < File::C);
        assert!(File::C < File::D);
        assert!(File::D < File::E);
        assert!(File::E < File::F);
        assert!(File::F < File::G);
        assert!(File::G < File::H);
    }

    #[test]
    fn ranks_are_ordered_numbers() {
        assert!(Rank::_1 < Rank::_2);
        assert!(Rank::_2 < Rank::_3);
        assert!(Rank::_3 < Rank::_4);
        assert!(Rank::_4 < Rank::_5);
        assert!(Rank::_5 < Rank::_6);
        assert!(Rank::_6 < Rank::_7);
        assert!(Rank::_7 < Rank::_8);
    }

    #[test]
    fn ranks_can_be_displayed() {
        assert_eq!("1", Rank::_1.to_string());
        assert_eq!("8", Rank::_8.to_string());
    }

    #[test]
    fn files_can_be_displayed() {
        assert_eq!("a", File::A.to_string());
        assert_eq!("d", File::D.to_string());
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
    fn can_shift_file_of_square() {
        assert_eq!(Square::C3.shift_file(0), Square::C3);
        assert_eq!(Square::C3.shift_file(1), Square::D3);
        assert_eq!(Square::C3.shift_file(-1), Square::B3);
        assert_eq!(Square::C3.shift_file(-2), Square::A3);
        assert_eq!(Square::C3.shift_file(5), Square::H3);
    }
}
