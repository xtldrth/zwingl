use std::io;

use crate::utf8_reader::{Error as Utf8ReaderError, Utf8Reader};

fn is_alphanumeric(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn identifier_or_keyword(s: String) -> TokenKind {
    use TokenKind::*;
    match s.as_str() {
        "true" => True,
        "false" => False,
        "let" => Let,
        "const" => Const,
        "if" => If,
        "else" => Else,
        "fn" => Fn,
        "struct" => Struct,
        "for" => For,
        "return" => Return,
        "in" => In,
        "u8" => UintType(8),
        "u16" => UintType(16),
        "u32" => UintType(32),
        "u64" => UintType(64),
        "i8" => IntType(8),
        "i16" => IntType(16),
        "i32" => IntType(32),
        "i64" => IntType(64),
        "f16" => FloatType(16),
        "f32" => FloatType(32),
        "f64" => FloatType(64),
        "str" => StringType,
        _ => Identifier(s),
    }
}

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

#[derive(Clone)]
pub struct Token {
    pub starts_at_line: usize,
    pub starts_at_column: usize,
    pub ends_at_line: usize,
    pub ends_at_column: usize,
    kind: TokenKind,
}

impl Token {
    pub fn kind(&self) -> TokenKind {
        self.kind.clone()
    }
}

#[derive(Clone, Debug)]
pub enum TokenKind {
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
    Rng,         // ..
    RngInc,      // ..=
    Colon,       // :
    ColonColon,  // ::
    Semicolon,   // ;
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
    Char(char), // starts with ' TODO: add special symbols support like '\n' and so on
    Int(i128),
    Float(f64), // TODO: add this format .01
    True,       //  true
    False,      //  false

    Let,                //  let
    Const,              // const
    If,                 // if
    Else,               //  else
    Fn,                 //  fn
    Struct,             // struct
    For,                //  for
    Return,             //  return
    In,                 //  in
    Identifier(String), //  starts with _ or any letter and can contain any letter or digit or '_'

    FloatType(usize),
    IntType(usize),
    UintType(usize),
    StringType,
    EOF,
}

impl PartialEq for TokenKind {
    fn eq(&self, other: &Self) -> bool {
        use std::mem::discriminant;
        discriminant(self) == discriminant(other)
    }
}

pub struct Lexer<R> {
    chars_reader: Utf8Reader<R>,
    current_char: Option<char>,
    second_char: Option<char>,
    third_char: Option<char>,
    col: usize,
    line: usize,
    is_eof_reached: bool,
}

impl<R: io::Read> Lexer<R> {
    pub fn new(chars_reader: Utf8Reader<R>) -> Result<Self, Error> {
        let mut l = Self {
            chars_reader,
            current_char: None,
            second_char: None,
            third_char: None,
            col: 0,
            line: 1,
            is_eof_reached: false,
        };
        l.current_char = match l.chars_reader.next() {
            Some(res) => match res {
                Ok(c) => Some(c),
                Err(e) => return Err(Error::ReaderError(e)),
            },
            None => None,
        };
        l.second_char = match l.chars_reader.next() {
            Some(res) => match res {
                Ok(c) => Some(c),
                Err(e) => return Err(Error::ReaderError(e)),
            },
            None => None,
        };
        l.third_char = match l.chars_reader.next() {
            Some(res) => match res {
                Ok(c) => Some(c),
                Err(e) => return Err(Error::ReaderError(e)),
            },
            None => None,
        };
        Ok(l)
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

    fn peek(&self) -> Option<char> {
        self.current_char
    }
    fn peek_2nd(&self) -> Option<char> {
        self.second_char
    }
    fn peek_3rd(&self) -> Option<char> {
        self.third_char
    }
    fn seek(&mut self) -> Result<Option<char>, Error> {
        self.current_char = self.second_char;
        self.second_char = self.third_char;
        self.third_char = self.get_next_char()?;
        self.col += 1;
        Ok(self.current_char)
    }

    fn advice_if_match(
        &mut self,
        expected: char,
        default: TokenKind,
        matched: TokenKind,
        next_char: Option<char>,
    ) -> Result<TokenKind, Error> {
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

    fn next_token_kind(&mut self) -> Result<Option<TokenKind>, Error> {
        use TokenKind::*;
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
                return self.next_token_kind();
            }
            '(' => LParen,
            ')' => RParen,
            '{' => LBrace,
            '}' => RBrace,
            '[' => LBraket,
            ']' => RBraket,
            ',' => Comma,
            ';' => Semicolon,
            '.' => {
                let next_char = self.seek()?;
                let token = match self.advice_if_match('.', Dot, Rng, next_char)? {
                    Rng => self.advice_if_match('=', Rng, RngInc, self.current_char)?,
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
                        Sub => self.advice_if_match('>', Sub, ArrowRight, next_char)?,
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
                        return self.next_token_kind();
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
            '~' => TokenKind::BitNot,
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
            '0'..='9' => return Ok(Some(self.parse_number()?)),
            '_' | 'a'..='z' | 'A'..='Z' => return Ok(Some(self.parse_identifier_or_keyword()?)),
            _ => todo!(),
        };
        self.seek()?;
        Ok(Some(token))
    }
    fn parse_identifier_or_keyword(&mut self) -> Result<TokenKind, Error> {
        let mut chars = Vec::new();
        while let Some(c) = self.current_char
            && is_alphanumeric(c)
        {
            chars.push(c);
            self.seek()?;
        }
        Ok(identifier_or_keyword(chars.iter().collect::<String>()))
    }
    fn parse_number(&mut self) -> Result<TokenKind, Error> {
        let mut is_float = false;
        let mut num_chars = Vec::<char>::new();
        while let Some(c) = self.current_char
            && (c.is_numeric() || c == '_' || c == '.')
        {
            match c {
                '_' => {
                    if let Some(lc) = num_chars.last()
                        && *lc == '_'
                    {
                        return Err(Error::LexerError {
                            cause: "_ must separate successive digit".into(),
                            line: self.line,
                            column: self.col,
                        });
                    }
                }
                '.' => {
                    if let Some(ch) = self.peek_2nd()
                        && !ch.is_numeric()
                    {
                        break;
                    }
                    if is_float {
                        break;
                    } else {
                        num_chars.push(c);
                        is_float = true;
                    }
                }
                '0'..='9' => num_chars.push(c),
                _ => unreachable!(),
            }
            self.seek()?;
        }
        if is_float {
            return match num_chars.iter().collect::<String>().parse::<f64>() {
                Err(e) => Err(Error::LexerError {
                    cause: format!("float parsing error: {e}"),
                    line: self.line,
                    column: self.col,
                }),
                Ok(n) => Ok(TokenKind::Float(n)),
            };
        }
        match num_chars.iter().collect::<String>().parse::<i128>() {
            Err(e) => Err(Error::LexerError {
                cause: format!("int parsing error: {e}"),
                line: self.line,
                column: self.col,
            }),
            Ok(n) => Ok(TokenKind::Int(n)),
        }
    }

    fn parse_char(&mut self) -> Result<TokenKind, Error> {
        match self.seek()? {
            Some(c) => {
                if c == '\\' {
                    todo!();
                } else if c == '\'' {
                    Err(Error::LexerError {
                        cause: "empty char literal".into(),
                        line: self.line,
                        column: self.col,
                    })
                } else if let Some(next_char) = self.seek()?
                    && next_char == '\''
                {
                    self.seek()?;
                    Ok(TokenKind::Char(c))
                } else {
                    Err(Error::LexerError {
                        cause: "unterminated char literal".into(),
                        line: self.line,
                        column: self.col,
                    })
                }
            }
            None => Err(Error::LexerError {
                cause: "unterminated char literal".into(),
                line: self.line,
                column: self.col,
            }),
        }
    }

    fn parse_string(&mut self) -> Result<TokenKind, Error> {
        let mut string_chars = Vec::<char>::new();
        while let Some(char) = self.seek()? {
            match char {
                '"' => {
                    return {
                        self.seek()?;
                        Ok(TokenKind::String(string_chars.iter().collect::<String>()))
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

    fn new_token(&self, start_line: usize, start_col: usize, kind: TokenKind) -> Token {
        return Token {
            starts_at_line: start_line,
            starts_at_column: start_col,
            ends_at_line: self.line,
            ends_at_column: self.col,
            kind,
        };
    }
}

impl<R: io::Read> Iterator for Lexer<R> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let start_col = self.col;
        let start_line = self.line;
        match self.next_token_kind() {
            Ok(o) => match o {
                Some(k) => Some(Ok(self.new_token(start_line, start_col, k))),
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

    fn new_lexer(input: &str) -> Lexer<Cursor<Vec<u8>>> {
        Lexer::new(Utf8Reader::new(
            Cursor::new(input.to_string().clone().into_bytes()).bytes(),
        ))
        .expect("unexpecetd error while creating lexer")
    }

    #[test]
    fn simple_tokens() {
        use crate::tokenizer::TokenKind::*;
        let string = r#"
(
)
{
}
[
]
,
.
<-
->
..
..=
:
::
+
++
-
--
*
/
%
=
:=
+=
-=
*=
/=
%=
|=
&=
<<=
>>=
^=
==
|
||
&
&&
<
<=
>
>=
<<
>>
~
^
!
!=
// This Commment should be skipped
true
false
let
const
if
else
fn
struct
for
return
in
_1dentifier
Identifier
iDentifier
identifi_er_
identifier
"simple string"
'c'
123
123.123
1_2_3
1_2_3.1_2_3
"#;

        let expected_token_kinds = vec![
            LParen,
            RParen,
            LBrace,
            RBrace,
            LBraket,
            RBraket,
            Comma,
            Dot,
            ArrowLeft,
            ArrowRight,
            Rng,
            RngInc,
            Colon,
            ColonColon,
            Add,
            Inc,
            Sub,
            Dec,
            Star,
            Div,
            Mod,
            Assign,
            ShortAssign,
            AddAssign,
            SubAssign,
            MulAssign,
            DivAssign,
            ModAssign,
            BitOrAssign,
            BitAndAssign,
            ShLAssign,
            ShRAssign,
            XorAssign,
            Equal,
            BitOr,
            Or,
            Amp,
            And,
            Less,
            LessEq,
            Great,
            GreatEq,
            ShL,
            ShR,
            BitNot,
            Xor,
            Not,
            NotEqual,
            //
            True,
            False,
            //
            Let,
            Const,
            If,
            Else,
            Fn,
            Struct,
            For,
            Return,
            In,
            Identifier("_1dentifier".into()),
            Identifier("Identifier".into()),
            Identifier("iDentifier".into()),
            Identifier("identifi_er_".into()),
            Identifier("identifier".into()),
            String("simple string".into()),
            Char('c'),
            //
            Int(123),
            Float(123.123),
            Int(123),
            Float(123.123),
            EOF,
        ];

        let lexer = new_lexer(&string);

        for (token_result, expected_token_kind) in lexer.zip(expected_token_kinds) {
            let token = token_result
                .map_err(|e| {
                    format!(
                        "unexpeceted error, expected token {:?}, got error: {:?}",
                        expected_token_kind, e
                    )
                })
                .unwrap();
            assert_eq!(token.kind, expected_token_kind);
            match (token.kind, expected_token_kind) {
                (Identifier(g), Identifier(e)) | (String(g), String(e)) => assert_eq!(g, e),
                (Char(g), Char(e)) => assert_eq!(g, e),
                (_, _) => (),
            }
        }
    }

    #[test]
    fn empty_input() {
        use crate::tokenizer::TokenKind::*;
        let mut lexer = new_lexer("");
        if let Some(token_result) = lexer.next() {
            assert_eq!(token_result.unwrap().kind, EOF)
        }
    }

    #[test]
    fn string() {
        use crate::tokenizer::TokenKind::*;
        let mut lexer = new_lexer("\"this is a string\"");
        if let Some(token_result) = lexer.next() {
            assert_eq!(
                token_result.unwrap().kind,
                String("this is a string".to_string())
            )
        }
    }

    #[test]
    fn numbers() {
        use crate::tokenizer::TokenKind::*;
        let mut lexer = new_lexer("12 12.12 12..12 12.1_2, 1 2 3 1.a");
        let expected_token_kinds = vec![
            Int(12),
            Float(12.12),
            Int(12),
            Rng,
            Int(12),
            Float(12.12),
            Comma,
            Int(1),
            Int(2),
            Int(3),
            Int(1),
            Dot,
            Identifier("a".into()),
        ];
        for (i, (got, expected)) in lexer.zip(expected_token_kinds).enumerate().into_iter() {
            match got {
                Ok(t) => assert!(
                    t.kind() == expected,
                    "tokens don't match at index {i}\nExpected:\n{:?}\nGot:\n{:?}\n",
                    expected,
                    t.kind(),
                ),
                Err(e) => panic!("unexpecetd: {e:?}"),
            }
        }
    }
}
