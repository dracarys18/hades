use super::symbol::{ALPHABETIC, ASCII_DIGITS, OPS, PUNCTUATION, WHITESPACE};
use memchr::memchr;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Bytes {
    inner: Vec<Byte>,
}

impl Bytes {
    pub fn from(byte: &[Byte]) -> Self {
        Self {
            inner: byte.to_vec(),
        }
    }

    pub fn new(inner: &[u8]) -> Bytes {
        let bytes = inner.iter().map(|&b| Byte::new(b)).collect::<Vec<Byte>>();
        Bytes { inner: bytes }
    }

    pub fn to_string(&self) -> String {
        self.inner.iter().map(|b| b.as_char()).collect()
    }

    pub fn consume_while<F>(&self, start_pos: usize, mut predicate: F) -> (usize, &ByteSlice)
    where
        F: FnMut(Byte) -> bool,
    {
        let mut pos = start_pos;
        while pos < self.inner.len() {
            if let Some(&byte) = self.inner.get(pos) {
                if predicate(byte) {
                    pos += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        (pos, &self[start_pos..pos])
    }
}

impl std::ops::Index<std::ops::Range<usize>> for Bytes {
    type Output = ByteSlice;
    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        ByteSlice::new(&self.inner[index])
    }
}

impl<'a> IntoIterator for &'a Bytes {
    type Item = Byte;
    type IntoIter = std::iter::Copied<std::slice::Iter<'a, Byte>>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter().copied()
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Byte {
    needle: u8,
}

impl std::fmt::Display for Byte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.needle.is_ascii_graphic() || self.needle == b' ' {
            write!(f, "{}", self.needle as char)
        } else {
            write!(f, "\\x{:02x}", self.needle)
        }
    }
}

impl PartialEq<u8> for Byte {
    fn eq(&self, other: &u8) -> bool {
        self.needle == *other
    }

    fn ne(&self, other: &u8) -> bool {
        self.needle != *other
    }
}

impl PartialEq<char> for Byte {
    fn eq(&self, other: &char) -> bool {
        self.needle == *other as u8
    }
    fn ne(&self, other: &char) -> bool {
        self.needle != *other as u8
    }
}

impl Byte {
    pub fn new(needle: u8) -> Self {
        Self { needle }
    }

    pub fn as_char(&self) -> char {
        self.needle as char
    }

    pub fn as_u8(&self) -> u8 {
        self.needle
    }

    pub fn is_whitespace(&self) -> bool {
        memchr(self.needle, WHITESPACE).is_some()
    }

    pub fn is_digit(&self) -> bool {
        memchr(self.needle, ASCII_DIGITS).is_some()
    }

    pub fn is_alphabetic(&self) -> bool {
        memchr(self.needle, ALPHABETIC).is_some()
    }

    pub fn is_alphanumeric(&self) -> bool {
        self.is_alphabetic() || self.is_digit()
    }

    pub fn is_operator(&self) -> bool {
        memchr::memchr(self.needle, OPS).is_some()
    }

    pub fn is_punctuation(&self) -> bool {
        memchr::memchr(self.needle, PUNCTUATION).is_some()
    }

    pub fn is_string_start(&self) -> bool {
        self.needle == b'"'
    }
}

#[repr(transparent)]
pub struct ByteSlice {
    inner: [Byte],
}

impl ByteSlice {
    pub fn new(inner: &[Byte]) -> &Self {
        // SAFETY: ByteSlice is repr(transparent) over [Byte]
        unsafe { &*(inner as *const [Byte] as *const ByteSlice) }
    }

    pub fn contains(&self, byte: Byte) -> bool {
        self.inner.iter().any(|&b| b == byte)
    }

    pub fn contains_byte(&self, byte: u8) -> bool {
        self.inner.iter().any(|&b| b.eq(&byte))
    }

    pub fn to_string(&self) -> String {
        self.inner.iter().map(|b| b.as_char()).collect()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Byte> {
        self.inner.iter()
    }
}
