use crate::utf8_reader::Error as Utf8ReaderError;
use std::fmt;

mod lexer;
mod tokens;

pub use lexer::*;
pub use tokens::*;

#[derive(Debug)]
pub enum Error {
    ReaderError(Utf8ReaderError),
    LexerError {
        cause: String,
        line: usize,
        column: usize,
    },
    InvalidSpan,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReaderError(e) => write!(f, "{}", e)?,
            Self::InvalidSpan => write!(f, "invalid span")?,
            Self::LexerError {
                cause,
                line,
                column,
            } => write!(f, "{}:{}: {}", line, column, cause)?,
        }
        Ok(())
    }
}

type Result<T> = std::result::Result<T, Error>;
