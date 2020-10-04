use std::borrow::Borrow;
use std::fmt;

use anyhow::{anyhow, Error};
use std::ops::Index;
use std::ops::{Add, Sub};
use std::str::FromStr;

#[derive(PartialOrd, Ord, PartialEq, Eq, Copy, Clone, Hash)]
pub struct File(u8);

impl File {
    pub const A: File = File(0);
    pub const B: File = File(1);
    pub const C: File = File(2);
    pub const D: File = File(3);
    pub const E: File = File(4);
    pub const F: File = File(5);
    pub const G: File = File(6);
    pub const H: File = File(7);

    pub const VALUES: [Self; 8] = [
        File::A,
        File::B,
        File::C,
        File::D,
        File::E,
        File::F,
        File::G,
        File::H,
    ];

    const STRS: FileMap<&'static str> = FileMap::new(["a", "b", "c", "d", "e", "f", "g", "h"]);

    pub fn from_index(index: u8) -> Self {
        File(index)
    }

    pub fn to_index(self) -> u8 {
        self.0
    }
}

impl Add<i8> for File {
    type Output = Self;

    fn add(self, offset: i8) -> Self {
        Self::VALUES[(self.to_index() as i8 + offset) as usize]
    }
}

impl Sub<File> for File {
    type Output = i8;

    fn sub(self, rhs: File) -> Self::Output {
        self.to_index() as i8 - rhs.to_index() as i8
    }
}

impl fmt::Debug for File {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_str(&File::STRS[self].to_ascii_uppercase())
    }
}

impl fmt::Display for File {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        f.write_str(File::STRS[self])
    }
}

impl FromStr for File {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let file = match s {
            "a" => File::A,
            "b" => File::B,
            "c" => File::C,
            "d" => File::D,
            "e" => File::E,
            "f" => File::F,
            "g" => File::G,
            "h" => File::H,
            x => return Err(anyhow!("unrecognised file '{}'", x)),
        };
        Ok(file)
    }
}

pub struct FileMap<T>([T; 8]);

impl<T> FileMap<T> {
    pub const fn new(values: [T; 8]) -> FileMap<T> {
        FileMap(values)
    }
}

impl<T, F: Borrow<File>> Index<F> for FileMap<T> {
    type Output = T;

    fn index(&self, file: F) -> &T {
        &self.0[file.borrow().to_index() as usize]
    }
}
