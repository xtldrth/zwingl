// just because i want it 😉

use core::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    CorruptedData(u8),
    UnterminatedUtf8Char,
    CharParsingError(u32),
    IOError(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CorruptedData(byte) => write!(
                f,
                "corrupted data, not a valid UTF-8 charachter, (byte 0x{:02X})",
                byte
            ),
            Self::IOError(e) => write!(f, "{}", e),
            Self::UnterminatedUtf8Char => write!(f, "unexpected bytes sequence end"),
            Self::CharParsingError(u) => write!(f, "corrupted character {:x}", u),
        }
    }
}

pub struct Utf8Reader<R> {
    bytes: io::Bytes<R>,
}

impl<R: io::Read> Utf8Reader<R> {
    pub fn new(bytes: io::Bytes<R>) -> Self {
        Self { bytes }
    }

    fn read_byte(&mut self) -> Option<Result<u8, Error>> {
        match self.bytes.next() {
            Some(result) => Some(result.map_err(Error::IOError)),
            None => None,
        }
    }
}

const _4_BITS_MASK: u8 = 0xF0;
const _3_BITS_MASK: u8 = 0xE0;
const _2_BITS_MASK: u8 = 0xC0;
const _1_BIT_MASK: u8 = 0x80;

impl<R: io::Read> Iterator for Utf8Reader<R> {
    /// returns char and its size
    type Item = Result<(char, u8), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let byte = match self.read_byte()? {
            Ok(b) => b,
            Err(e) => return Some(Err(e)),
        };
        let (bytes_count, clear) = match byte & 0xF0 {
            _4_BITS_MASK => (4, 0x0F),
            _3_BITS_MASK => (3, 0x0F),
            _ if byte & _2_BITS_MASK == _2_BITS_MASK => (2, 0x1F),
            _ if _1_BIT_MASK & byte == 0 => return Some(Ok((byte as char, 1))),
            _ => return Some(Err(Error::CorruptedData(byte))),
        };
        // praparing raw bytes to convet them into char
        let mut c: u32 = ((byte & clear) as u32) << 6 * (bytes_count - 1);
        for i in 2..=bytes_count {
            match self.read_byte() {
                Some(res) => match res {
                    Ok(b) => {
                        // check that all next bytes starting with 10
                        if b & 0xC0 != 0x80 {
                            return Some(Err(Error::CorruptedData(b)));
                        }
                        // clearing this staring symbols and adding bits sequence
                        // to this character
                        c = c | ((b & 0x3F) as u32) << (6 * (bytes_count - i));
                    }
                    Err(e) => return Some(Err(e)),
                },
                None => {
                    return Some(Err(Error::UnterminatedUtf8Char));
                }
            }
        }
        Some(match char::from_u32(c) {
            Some(c) => Ok((c, bytes_count)),
            None => Err(Error::CharParsingError(c)),
        })
    }
}

#[cfg(test)]
mod test {
    use std::io::Read;

    use crate::utf8_reader::Utf8Reader;

    #[test]
    fn chars_with_different_sizes() {
        let chars = ["a", "¢", "€", "🚀"];
        for (i, c) in chars.iter().enumerate() {
            let expected_char = chars[i].chars().nth(0).unwrap();
            let op = format!("char( '{expected_char}' ) with len {}", i + 1);
            let mut reader = Utf8Reader::new(c.as_bytes().bytes());
            let (c, size) = reader
                .next()
                .expect(format!("{op}:").as_str())
                .map_err(|e| format!("{op}: unexpected error: {e}"))
                .unwrap();
            assert!(
                size as usize == i + 1,
                "{op}: size doesn't match\nexpected: {}\ngot: {}",
                i + 1,
                size
            );
            assert!(
                c == expected_char,
                "{op}: \nexpected: {expected_char}\ngot: {c}",
            );
        }
    }

    #[test]
    fn iterator_test() {
        let string = "Hello! Как дела? Friedrichstraße 🦀 こんにちは 123";
        for byte in "🦀".as_bytes() {
            print!("{:08b}  ", byte);
        }
        let reader = Utf8Reader::new(string.as_bytes().bytes());
        let mut result_chars = Vec::<char>::new();
        reader
            .into_iter()
            .zip(string.chars())
            .enumerate()
            .for_each(|(i, (got, expected))| {
                let (got, _) = match got {
                    Ok(c) => c,
                    Err(e) => panic!("unexpected error at index {i}, {e}"),
                };
                assert!(
                    got == expected,
                    "characters doesn't match at index {i}, expected: {expected}, got {got}\n expected bits\t{:032b}\n got bits \t{:032b}",
                    expected as i32, got as i32
                );
                result_chars.push(got);
            });
        assert_eq!(result_chars.iter().cloned().collect::<String>(), string);
    }
}
