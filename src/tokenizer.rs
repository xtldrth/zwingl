use std::{any::Any, io};

use crate::utf8_reader::{Error as Utf8ReaderError, Utf8Reader};

#[derive(Debug)]
pub enum Error {
    ReaderError(Utf8ReaderError),
    LexerError {
        cause: String,
        line: usize,
        column: usize,
    },
    EmptyData,
}

#[derive(Clone, Debug)]
pub enum Token {
    LParen,      // (
    RParen,      // )
    LBrace,      // {
    RBrace,      // }
    LBraket,     // [
    RBraket,     // ]
    Comma,       // ,
    Dot,         // .
    ArrowLeft,   // <-
    ArrowRight,  // ->
    Range,       // ..
    RangeIncl,   // ..=
    Colon,       // :
    ColonColon,  // ::
    Add,         // +
    Inc,         // ++
    Sub,         // -
    Dec,         // --
    Star,        // *
    Div,         // /
    Mod,         // %
    Assign,      // =
    ShortAssign, // :=

    AddAssign, // +=
    SubAssign, // -=
    MulAssign, // *=
    DivAssign, // /=
    ModAssign, // %=

    BitOrAssign,  // |=
    BitAndAssign, // &=
    ShLAssign,    // <<=
    ShRAssign,    // >>=
    XorAssign,    // ^=
    Equal,        // ==
    BitOr,        // |
    Or,           // ||
    Amp,          // &
    And,          // &&
    Less,         // <
    LessEq,       // <=
    Great,        // >
    GreatEq,      // >=
    ShL,          // <<
    ShR,          // >>
    BitNot,       // ~
    Xor,          // ^
    Not,          // !
    NotEqual,     // !=
    Comment,      // //

    String(String),
    RawString(String), // starts with ` 	TODO:
    Char,              // starts with ' 	TODO:
    Int,               // TODO: 123 or 1_2_3, or 1_23
    Float,             // TODO: same as int but with '.'
    True,              //  TODO: true
    False,             //  TODO: false

    Let,                //  TODO: let
    Const,              //  TODO: const
    If,                 // if TODO:
    Else,               // TODO: else
    Fn,                 // TODO: fn
    Struct,             // TODO:struct
    For,                // TODO: for
    Return,             // TODO: return
    Identifier(String), // TODO: starts with _ or any letter and can contain any letter or digit or '_'
    EOF,
}

pub struct Lexer<'r, R> {
    chars_reader: &'r mut Utf8Reader<R>,
    current_char: Option<char>,
    col: usize,
    line: usize,
    is_eof_reached: bool,
}

impl<'r, R: io::Read> Lexer<'r, R> {
    pub fn new(chars_reader: &'r mut Utf8Reader<R>) -> Result<Self, Error> {
        let curr_char = chars_reader.next();
        Ok(Self {
            chars_reader,
            current_char: match curr_char {
                Some(r) => match r {
                    Ok(c) => Some(c),
                    Err(e) => return Err(Error::ReaderError(e)),
                },
                None => return Err(Error::EmptyData),
            },
            col: 0,
            line: 1,
            is_eof_reached: false,
        })
    }

    fn get_next_char(&mut self) -> Result<Option<char>, Error> {
        match self.chars_reader.next() {
            Some(res) => match res {
                Ok(c) => Ok(Some(c)),
                Err(e) => Err(Error::ReaderError(e)),
            },
            None => Ok(None),
        }
    }
    fn seek(&mut self) -> Result<Option<char>, Error> {
        self.current_char = self.get_next_char()?;
        self.col += 1;
        Ok(self.current_char)
    }

    fn advice_if_match(
        &mut self,
        expected: char,
        default: Token,
        matched: Token,
        next_char: Option<char>,
    ) -> Result<Token, Error> {
        match next_char {
            Some(c) => {
                if c != expected {
                    return Ok(default);
                }
                self.current_char = self.seek()?;
                return Ok(matched);
            }
            None => return Ok(default),
        }
    }

    fn skip_comment(&mut self) -> Result<(), Error> {
        while let Some(c) = self.seek()?
            && c != '\n'
        {}
        Ok(())
    }

    fn next_token(&mut self) -> Result<Option<Token>, Error> {
        use Token::*;
        let current_char = match self.current_char {
            Some(c) => c,
            None => {
                return if self.is_eof_reached {
                    Ok(None)
                } else {
                    self.is_eof_reached = true;
                    Ok(Some(EOF))
                };
            }
        };
        let token = match current_char {
            _ if current_char.is_whitespace() => {
                if current_char == '\n' {
                    self.col = 1;
                    self.line += 1;
                }
                self.seek()?;
                return self.next_token();
            }
            '(' => LParen,
            ')' => RParen,
            '{' => LBrace,
            '}' => RBrace,
            '[' => LBraket,
            ']' => RBraket,
            ',' => Comma,
            '.' => {
                let next_char = self.seek()?;
                let token = match self.advice_if_match('.', Dot, Range, next_char)? {
                    Range => {
                        let next_char = self.seek()?;
                        self.advice_if_match('=', Range, RangeIncl, next_char)?
                    }
                    _ => Dot,
                };
                return Ok(Some(token));
            }
            ':' => {
                let next_char = self.seek()?;
                let token = match self.advice_if_match(':', Colon, ColonColon, next_char)? {
                    Colon => self.advice_if_match('=', Colon, ShortAssign, next_char)?,
                    _ => ColonColon,
                };
                return Ok(Some(token));
            }
            '+' => {
                let next_char = self.seek()?;
                let token = match self.advice_if_match('+', Add, Inc, next_char)? {
                    Add => self.advice_if_match('=', Add, AddAssign, next_char)?,
                    _ => Inc,
                };
                return Ok(Some(token));
            }
            '-' => {
                let next_char = self.seek()?;
                let token = match self.advice_if_match('-', Sub, Dec, next_char)? {
                    Sub => match self.advice_if_match('=', Sub, SubAssign, next_char)? {
                        Sub => self.advice_if_match('=', Sub, ArrowRight, next_char)?,
                        _ => SubAssign,
                    },
                    _ => Dec,
                };
                return Ok(Some(token));
            }
            '*' => {
                let next_char = self.seek()?;
                return Ok(Some(self.advice_if_match('=', Star, MulAssign, next_char)?));
            }
            '/' => {
                let next_char = self.seek()?;
                let token = match self.advice_if_match('/', Div, Comment, next_char)? {
                    Comment => {
                        self.skip_comment()?;
                        return self.next_token();
                    }
                    _ => self.advice_if_match('=', Div, DivAssign, next_char)?,
                };
                return Ok(Some(token));
            }
            '%' => {
                let next_char = self.seek()?;
                return Ok(Some(self.advice_if_match('=', Mod, ModAssign, next_char)?));
            }
            '=' => {
                let next_char = self.seek()?;
                return Ok(Some(self.advice_if_match('=', Assign, Equal, next_char)?));
            }
            '|' => {
                let next_char = self.seek()?;
                let token = match self.advice_if_match('=', BitOr, BitOrAssign, next_char)? {
                    BitOr => self.advice_if_match('|', BitOr, Or, next_char)?,
                    _ => BitOrAssign,
                };
                return Ok(Some(token));
            }
            '&' => {
                let next_char = self.seek()?;
                let token = match self.advice_if_match('=', Amp, BitAndAssign, next_char)? {
                    Amp => self.advice_if_match('&', Amp, And, next_char)?,
                    _ => BitAndAssign,
                };
                return Ok(Some(token));
            }
            '^' => {
                let next_char = self.seek()?;
                return Ok(Some(self.advice_if_match('=', Xor, XorAssign, next_char)?));
            }
            '~' => Token::BitNot,
            '!' => {
                let next_char = self.seek()?;
                return Ok(Some(self.advice_if_match('=', Not, NotEqual, next_char)?));
            }
            '<' => {
                let next_char = self.seek()?;
                let token = match self.advice_if_match('=', Less, LessEq, next_char)? {
                    Less => match self.advice_if_match('<', Less, ShL, next_char)? {
                        ShL => self.advice_if_match('=', ShL, ShLAssign, self.current_char)?,
                        _ => self.advice_if_match('-', Less, ArrowLeft, next_char)?,
                    },
                    _ => LessEq,
                };
                return Ok(Some(token));
            }
            '>' => {
                let next_char = self.seek()?;
                let token = match self.advice_if_match('=', Great, GreatEq, next_char)? {
                    Great => match self.advice_if_match('>', Great, ShR, next_char)? {
                        ShR => self.advice_if_match('=', ShR, ShRAssign, self.current_char)?,
                        _ => Great,
                    },
                    _ => GreatEq,
                };
                return Ok(Some(token));
            }
            '"' => return Ok(Some(self.parse_string()?)),
            '\'' => return Ok(Some(self.parse_char()?)),
            _ => todo!(),
        };
        self.seek()?;
        Ok(Some(token))
    }

    fn parse_char(&mut self) -> Result<Token, Error> {
        todo!()
    }

    fn parse_string(&mut self) -> Result<Token, Error> {
        let mut string_chars = Vec::<char>::new();
        while let Some(char) = self.seek()? {
            match char {
                '"' => {
                    return {
                        self.seek()?;
                        Ok(Token::String(string_chars.iter().collect::<String>()))
                    };
                }
                '\n' => {
                    return Err(Error::LexerError {
                        cause: "unterminated string".into(),
                        line: self.line,
                        column: self.col,
                    });
                }
                _ => string_chars.push(char),
            }
        }
        Err(Error::LexerError {
            cause: "unterminated string".into(),
            line: self.line,
            column: self.col,
        })
    }
}

impl<'r, R: io::Read> Iterator for Lexer<'r, R> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(o) => match o {
                Some(t) => Some(Ok(t)),
                None => None,
            },
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tokenizer::Lexer;
    use crate::utf8_reader::Utf8Reader;
    use std::io::{Cursor, Read};
    #[test]
    fn lexer_tokens_basic_test() {
        use crate::tokenizer::Token::*;
        let string = " +=-=---\n(([])) )}]".to_string();

        let expected_tokens = vec![
            AddAssign, SubAssign, Dec, Sub, LParen, LParen, LBraket, RBraket, RParen, RParen,
            RParen, RBrace, RBraket,
        ];

        let mut reader = Utf8Reader::new(Cursor::new(string.clone().into_bytes()).bytes());
        let mut lexer = Lexer::new(&mut reader).expect("unexpecetd error while creating lexer");

        for (token_result, expected_token) in lexer.zip(expected_tokens) {
            let token = token_result
                .map_err(|e| {
                    format!(
                        "unexpeceted error, expected token {:?}, got error: {:?}",
                        expected_token, e
                    )
                })
                .unwrap();
            assert_eq!(format!("{:?}", token), format!("{:?}", expected_token))
        }
        assert!(false);
    }
}
