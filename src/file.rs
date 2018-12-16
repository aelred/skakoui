use std::ops::Add;
use std::fmt;
use enum_map::Enum;

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
