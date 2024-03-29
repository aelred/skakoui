use anyhow::{anyhow, Error};
use std::fmt;
use std::fmt::{Display, Write};
use std::ops::Add;
use std::ops::Index;
use std::ops::Sub;
use std::str::FromStr;

#[derive(PartialOrd, Ord, PartialEq, Eq, Copy, Clone, Hash)]
pub struct Rank(u8);

impl Rank {
    pub const _1: Rank = Rank(0);
    pub const _2: Rank = Rank(1);
    pub const _3: Rank = Rank(2);
    pub const _4: Rank = Rank(3);
    pub const _5: Rank = Rank(4);
    pub const _6: Rank = Rank(5);
    pub const _7: Rank = Rank(6);
    pub const _8: Rank = Rank(7);

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

    pub fn from_index(index: u8) -> Self {
        Rank(index)
    }

    pub fn to_index(self) -> u8 {
        self.0
    }
}

impl Add<i8> for Rank {
    type Output = Self;

    fn add(self, offset: i8) -> Self {
        Rank((self.0 as i8 + offset) as u8)
    }
}

impl Sub<i8> for Rank {
    type Output = Self;

    fn sub(self, offset: i8) -> Self {
        Rank((self.0 as i8 - offset) as u8)
    }
}

impl Sub<Rank> for Rank {
    type Output = i8;

    fn sub(self, rhs: Rank) -> Self::Output {
        self.0 as i8 - rhs.0 as i8
    }
}

impl fmt::Debug for Rank {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_char('_')?;
        Display::fmt(self, f)
    }
}

impl fmt::Display for Rank {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.to_index() + 1))
    }
}

impl FromStr for Rank {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num: u8 = s.parse()?;
        if !(1..=9).contains(&num) {
            Err(anyhow!("unrecognised rank"))
        } else {
            Ok(Rank::from_index(num - 1))
        }
    }
}

pub struct RankMap<T>([T; 8]);

impl<T> RankMap<T> {
    pub const fn new(values: [T; 8]) -> RankMap<T> {
        RankMap(values)
    }
}

impl<T> Index<Rank> for RankMap<T> {
    type Output = T;

    fn index(&self, rank: Rank) -> &T {
        &self.0[rank.to_index() as usize]
    }
}
