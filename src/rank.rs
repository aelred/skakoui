use enum_map::Enum;
use std::fmt;
use std::ops::Add;
use std::ops::Sub;

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
    pub const VALUES: [Self; 8] = [
        Rank::_1,
        Rank::_2,
        Rank::_3,
        Rank::_4,
        Rank::_5,
        Rank::_6,
        Rank::_7,
        Rank::_8,
    ];

    pub fn from_index(index: usize) -> Self {
        Enum::<usize>::from_usize(index)
    }

    pub fn to_index(self) -> usize {
        Enum::<usize>::to_usize(self)
    }
}

impl Add<isize> for Rank {
    type Output = Self;

    fn add(self, offset: isize) -> Self {
        Self::VALUES[(self.to_index() as isize + offset) as usize]
    }
}

impl Sub<isize> for Rank {
    type Output = Self;

    fn sub(self, offset: isize) -> Self {
        self + (-offset)
    }
}

impl fmt::Display for Rank {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.to_index() + 1))
    }
}
