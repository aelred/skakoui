use std::borrow::Borrow;
use std::fmt;
use std::ops::Add;
use std::ops::Index;
use std::str::FromStr;

#[derive(PartialOrd, Ord, PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct File(usize);

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

    #[inline]
    pub fn from_index(index: usize) -> Self {
        File(index)
    }

    #[inline]
    pub fn to_index(self) -> usize {
        self.0
    }
}

impl Add<isize> for File {
    type Output = Self;

    #[inline]
    fn add(self, offset: isize) -> Self {
        Self::VALUES[(self.to_index() as isize + offset) as usize]
    }
}

impl fmt::Display for File {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        const STRS: FileMap<&str> = FileMap::new(["a", "b", "c", "d", "e", "f", "g", "h"]);
        f.write_str(STRS[self])
    }
}

impl FromStr for File {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let file = match s {
            "a" => File::A,
            "b" => File::B,
            "c" => File::C,
            "d" => File::D,
            "e" => File::E,
            "f" => File::F,
            "g" => File::G,
            "h" => File::H,
            _ => return Err("unrecognised file".to_string()),
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

    #[inline]
    fn index(&self, file: F) -> &T {
        &self.0[file.borrow().to_index()]
    }
}
