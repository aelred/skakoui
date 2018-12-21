use enum_map::Enum;
use std::error::Error;
use std::fmt;
use std::ops::Add;
use std::ops::Sub;
use std::str::FromStr;

#[derive(PartialOrd, Ord, PartialEq, Eq, Enum, Copy, Clone, Debug, Hash)]
pub enum Rank {
    _1 = 0,
    _2 = 1,
    _3 = 2,
    _4 = 3,
    _5 = 4,
    _6 = 5,
    _7 = 6,
    _8 = 7,
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

    #[inline]
    pub fn from_index(index: usize) -> Self {
        Enum::<usize>::from_usize(index)
    }

    #[inline]
    pub fn to_index(self) -> usize {
        self as usize
    }
}

impl Add<isize> for Rank {
    type Output = Self;

    #[inline]
    fn add(self, offset: isize) -> Self {
        Self::VALUES[(self.to_index() as isize + offset) as usize]
    }
}

impl Sub<isize> for Rank {
    type Output = Self;

    #[inline]
    fn sub(self, offset: isize) -> Self {
        self + (-offset)
    }
}

impl fmt::Display for Rank {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.to_index() + 1))
    }
}

impl FromStr for Rank {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Box<dyn Error>> {
        let num: usize = s.parse()?;
        if num < 1 || num > Rank::VALUES.len() + 1 {
            Err("unrecognised rank".to_string().into())
        } else {
            Ok(Rank::from_index(num - 1))
        }
    }
}
