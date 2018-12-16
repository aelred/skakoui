use enum_map::Enum;
use std::fmt;
use std::ops::Add;
use std::ops::Sub;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Square {
    file: File,
    rank: Rank,
}

impl Square {
    pub const A1: Self = Square {
        file: File::A,
        rank: Rank::_1,
    };
    pub const A2: Self = Square {
        file: File::A,
        rank: Rank::_2,
    };
    pub const A3: Self = Square {
        file: File::A,
        rank: Rank::_3,
    };
    pub const A4: Self = Square {
        file: File::A,
        rank: Rank::_4,
    };
    pub const A5: Self = Square {
        file: File::A,
        rank: Rank::_5,
    };
    pub const A6: Self = Square {
        file: File::A,
        rank: Rank::_6,
    };
    pub const A7: Self = Square {
        file: File::A,
        rank: Rank::_7,
    };
    pub const A8: Self = Square {
        file: File::A,
        rank: Rank::_8,
    };
    pub const B1: Self = Square {
        file: File::B,
        rank: Rank::_1,
    };
    pub const B2: Self = Square {
        file: File::B,
        rank: Rank::_2,
    };
    pub const B3: Self = Square {
        file: File::B,
        rank: Rank::_3,
    };
    pub const B4: Self = Square {
        file: File::B,
        rank: Rank::_4,
    };
    pub const B5: Self = Square {
        file: File::B,
        rank: Rank::_5,
    };
    pub const B6: Self = Square {
        file: File::B,
        rank: Rank::_6,
    };
    pub const B7: Self = Square {
        file: File::B,
        rank: Rank::_7,
    };
    pub const B8: Self = Square {
        file: File::B,
        rank: Rank::_8,
    };
    pub const C1: Self = Square {
        file: File::C,
        rank: Rank::_1,
    };
    pub const C2: Self = Square {
        file: File::C,
        rank: Rank::_2,
    };
    pub const C3: Self = Square {
        file: File::C,
        rank: Rank::_3,
    };
    pub const C4: Self = Square {
        file: File::C,
        rank: Rank::_4,
    };
    pub const C5: Self = Square {
        file: File::C,
        rank: Rank::_5,
    };
    pub const C6: Self = Square {
        file: File::C,
        rank: Rank::_6,
    };
    pub const C7: Self = Square {
        file: File::C,
        rank: Rank::_7,
    };
    pub const C8: Self = Square {
        file: File::C,
        rank: Rank::_8,
    };
    pub const D1: Self = Square {
        file: File::D,
        rank: Rank::_1,
    };
    pub const D2: Self = Square {
        file: File::D,
        rank: Rank::_2,
    };
    pub const D3: Self = Square {
        file: File::D,
        rank: Rank::_3,
    };
    pub const D4: Self = Square {
        file: File::D,
        rank: Rank::_4,
    };
    pub const D5: Self = Square {
        file: File::D,
        rank: Rank::_5,
    };
    pub const D6: Self = Square {
        file: File::D,
        rank: Rank::_6,
    };
    pub const D7: Self = Square {
        file: File::D,
        rank: Rank::_7,
    };
    pub const D8: Self = Square {
        file: File::D,
        rank: Rank::_8,
    };
    pub const E1: Self = Square {
        file: File::E,
        rank: Rank::_1,
    };
    pub const E2: Self = Square {
        file: File::E,
        rank: Rank::_2,
    };
    pub const E3: Self = Square {
        file: File::E,
        rank: Rank::_3,
    };
    pub const E4: Self = Square {
        file: File::E,
        rank: Rank::_4,
    };
    pub const E5: Self = Square {
        file: File::E,
        rank: Rank::_5,
    };
    pub const E6: Self = Square {
        file: File::E,
        rank: Rank::_6,
    };
    pub const E7: Self = Square {
        file: File::E,
        rank: Rank::_7,
    };
    pub const E8: Self = Square {
        file: File::E,
        rank: Rank::_8,
    };
    pub const F1: Self = Square {
        file: File::F,
        rank: Rank::_1,
    };
    pub const F2: Self = Square {
        file: File::F,
        rank: Rank::_2,
    };
    pub const F3: Self = Square {
        file: File::F,
        rank: Rank::_3,
    };
    pub const F4: Self = Square {
        file: File::F,
        rank: Rank::_4,
    };
    pub const F5: Self = Square {
        file: File::F,
        rank: Rank::_5,
    };
    pub const F6: Self = Square {
        file: File::F,
        rank: Rank::_6,
    };
    pub const F7: Self = Square {
        file: File::F,
        rank: Rank::_7,
    };
    pub const F8: Self = Square {
        file: File::F,
        rank: Rank::_8,
    };
    pub const G1: Self = Square {
        file: File::G,
        rank: Rank::_1,
    };
    pub const G2: Self = Square {
        file: File::G,
        rank: Rank::_2,
    };
    pub const G3: Self = Square {
        file: File::G,
        rank: Rank::_3,
    };
    pub const G4: Self = Square {
        file: File::G,
        rank: Rank::_4,
    };
    pub const G5: Self = Square {
        file: File::G,
        rank: Rank::_5,
    };
    pub const G6: Self = Square {
        file: File::G,
        rank: Rank::_6,
    };
    pub const G7: Self = Square {
        file: File::G,
        rank: Rank::_7,
    };
    pub const G8: Self = Square {
        file: File::G,
        rank: Rank::_8,
    };
    pub const H1: Self = Square {
        file: File::H,
        rank: Rank::_1,
    };
    pub const H2: Self = Square {
        file: File::H,
        rank: Rank::_2,
    };
    pub const H3: Self = Square {
        file: File::H,
        rank: Rank::_3,
    };
    pub const H4: Self = Square {
        file: File::H,
        rank: Rank::_4,
    };
    pub const H5: Self = Square {
        file: File::H,
        rank: Rank::_5,
    };
    pub const H6: Self = Square {
        file: File::H,
        rank: Rank::_6,
    };
    pub const H7: Self = Square {
        file: File::H,
        rank: Rank::_7,
    };
    pub const H8: Self = Square {
        file: File::H,
        rank: Rank::_8,
    };

    pub fn new(file: File, rank: Rank) -> Self {
        Square { file, rank }
    }

    pub fn from_index(index: u32) -> Self {
        let index = index as usize;
        let quot = index / 8;
        let rem = index % 8;
        Square::new(File::from_index(rem), Rank::from_index(quot))
    }

    pub fn file(self) -> File {
        self.file
    }

    pub fn rank(self) -> Rank {
        self.rank
    }

    pub fn shift_rank(self, offset: isize) -> Square {
        Square {
            rank: self.rank + offset,
            file: self.file,
        }
    }

    pub fn shift_file(self, offset: isize) -> Square {
        Square {
            rank: self.rank,
            file: self.file + offset,
        }
    }

    pub fn color(self) -> SquareColor {
        if (self.file.to_index() + self.rank.to_index()) % 2 != 0 {
            SquareColor::White
        } else {
            SquareColor::Black
        }
    }
}

impl fmt::Debug for Square {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_fmt(format_args!("Square::{:?}{}", self.file, self.rank))
    }
}

impl fmt::Display for Square {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_fmt(format_args!("{}{}", self.file, self.rank))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SquareColor {
    Black,
    White,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Enum, Copy, Clone, Debug, Hash)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl File {
    pub const VALUES: [File; 8] = [
        File::A,
        File::B,
        File::C,
        File::D,
        File::E,
        File::F,
        File::G,
        File::H,
    ];

    pub fn from_index(index: usize) -> File {
        Enum::<usize>::from_usize(index)
    }

    pub fn to_index(self) -> usize {
        Enum::<usize>::to_usize(self)
    }
}

impl Add<isize> for File {
    type Output = File;

    fn add(self, offset: isize) -> File {
        Self::VALUES[(self.to_index() as isize + offset) as usize]
    }
}

impl fmt::Display for File {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        let s = match self {
            File::A => "a",
            File::B => "b",
            File::C => "c",
            File::D => "d",
            File::E => "e",
            File::F => "f",
            File::G => "g",
            File::H => "h",
        };
        f.write_str(s)
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Enum, Copy, Clone, Debug, Hash)]
pub enum Rank {
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
}

impl Rank {
    pub const VALUES: [Rank; 8] = [
        Rank::_1,
        Rank::_2,
        Rank::_3,
        Rank::_4,
        Rank::_5,
        Rank::_6,
        Rank::_7,
        Rank::_8,
    ];

    pub fn from_index(index: usize) -> Rank {
        Enum::<usize>::from_usize(index)
    }

    pub fn to_index(self) -> usize {
        Enum::<usize>::to_usize(self)
    }
}

impl Add<isize> for Rank {
    type Output = Rank;

    fn add(self, offset: isize) -> Rank {
        Self::VALUES[(self.to_index() as isize + offset) as usize]
    }
}

impl Sub<isize> for Rank {
    type Output = Rank;

    fn sub(self, offset: isize) -> Rank {
        self + (-offset)
    }
}

impl fmt::Display for Rank {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.to_index() + 1))
    }
}

#[cfg(test)]
mod tests {
    use super::File::*;
    use super::Rank::*;
    use super::*;

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

    #[test]
    #[should_panic]
    fn can_not_shift_file_east_of_board() {
        Square::C3.shift_file(-3);
    }

    #[test]
    #[should_panic]
    fn can_not_shift_file_west_of_board() {
        Square::C3.shift_file(6);
    }
}
