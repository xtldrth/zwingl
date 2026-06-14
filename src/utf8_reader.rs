use std::io;

#[derive(Debug)]
pub enum Error {
    CorruptedData,
    IOError(io::Error),
}

pub struct Utf8Reader<R> {
    bytes: io::Bytes<R>,
}

impl<R: io::Read> Utf8Reader<R> {
    pub fn new(bytes: io::Bytes<R>) -> Self {
        return Self { bytes };
    }

    fn read_byte(&mut self) -> Option<Result<u8, Error>> {
        match self.bytes.next() {
            Some(result) => match result {
                Ok(b) => Some(Ok(b)),
                Err(e) => Some(Err(Error::IOError(e))),
            },
            None => None,
        }
    }
}

impl<R: io::Read> Iterator for Utf8Reader<R> {
    type Item = Result<char, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let byte = match self.read_byte()? {
            Ok(b) => b,
            Err(e) => return Some(Err(e)),
        };
        let (bytes_count, clear) = match byte & 0xF0 {
            0xF0 => (4, 0x0F),
            0xE0 => (3, 0x0F),
            0xC0 => (2, 0x0F),
            _ if 0x80 & byte == 0 => return Some(Ok(byte as char)),
            _ => return Some(Err(Error::CorruptedData)),
        };
        // praparing raw bytes to convet them into char
        let mut c: u32 = ((byte & clear) as u32) << 6 * (bytes_count - 1);
        for i in 2..=bytes_count {
            match self.read_byte() {
                Some(res) => match res {
                    Ok(b) => {
                        if b & 0xC0 != 0x80 {
                            return Some(Err(Error::CorruptedData));
                        }
                        c = c | ((b & 0x3F) as u32) << (6 * (bytes_count - i));
                    }
                    Err(e) => return Some(Err(e)),
                },
                None => {
                    return Some(Err(Error::CorruptedData));
                }
            }
        }
        match char::from_u32(c) {
            Some(c) => Some(Ok(c)),
            None => Some(Err(Error::CorruptedData)),
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::{Cursor, Read};

    use crate::utf8_reader::Utf8Reader;

    #[test]
    fn iterator_test() {
        let string = "Hello! Friedrichstraße 🦀 こんにちは 123".to_string();
        let reader = Utf8Reader::new(Cursor::new(string.clone().into_bytes()).bytes());
        let mut result_chars = Vec::<char>::new();
        reader
            .into_iter()
            .zip(string.chars())
            .enumerate()
            .for_each(|(i, (got, expected))| {
                let got = match got {
                    Ok(c) => c,
                    Err(e) => panic!("unexpected error at index {i}, {e:?}"),
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
