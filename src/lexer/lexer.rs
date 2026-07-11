use super::{Error, Result, Span, Token, TokenKind};
use std::collections::VecDeque;
use std::io;

use crate::utf8_reader::Utf8Reader;

fn is_alphanumeric(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn identifier_or_keyword(s: &str) -> TokenKind {
    use TokenKind::*;
    match s {
        "_" => Underscore,
        "true" => True,
        "false" => False,
        "let" => Let,
        "const" => Const,
        "if" => If,
        "else" => Else,
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
        _ => Identifier,
    }
}

pub struct Lexer<R> {
    chars_reader: Utf8Reader<R>,
    lookahead: VecDeque<(char, u8)>,
    curr_byte_idx: usize,
    col: usize,
    line: usize,
}

impl<R: io::Read> Lexer<R> {
    pub fn new(chars_reader: Utf8Reader<R>) -> Result<Self> {
        const LOOKAHEAD_SIZE: usize = 3;
        let lookahead = VecDeque::with_capacity(LOOKAHEAD_SIZE);
        let mut l = Self {
            chars_reader,
            lookahead,
            curr_byte_idx: 0,
            col: 0,
            line: 1,
        };
        for _ in 0..LOOKAHEAD_SIZE {
            if let Some(c) = l.get_next_char_with_size()? {
                l.lookahead.push_back(c);
            }
        }
        Ok(l)
    }

    fn get_next_char_with_size(&mut self) -> Result<Option<(char, u8)>> {
        self.chars_reader
            .next()
            .transpose()
            .map_err(Error::ReaderError)
    }

    fn peek(&self, pos: usize) -> Option<char> {
        match self.lookahead.get(pos).copied() {
            Some((c, _)) => Some(c),
            None => None,
        }
    }
    fn seek(&mut self) -> Result<Option<char>> {
        let (c, size) = match self.lookahead.pop_front() {
            Some((c, size)) => (c, size),
            None => return Ok(None),
        };
        self.curr_byte_idx += size as usize;
        if let Some((c, size)) = self.get_next_char_with_size()? {
            self.lookahead.push_back((c, size));
        }
        self.col += 1;
        Ok(Some(c))
    }

    fn consume_if_match(
        &mut self,
        expected: char,
        default: TokenKind,
        matched: TokenKind,
    ) -> Result<TokenKind> {
        match self.peek(0) {
            Some(c) => {
                if c != expected {
                    return Ok(default);
                }
                self.seek()?;
                return Ok(matched);
            }
            None => return Ok(default),
        }
    }

    fn parse_comment(&mut self) -> Result<TokenKind> {
        while let Some(c) = self.peek(0) {
            if c == '\n' {
                break;
            }
            self.seek()?;
        }
        Ok(TokenKind::Comment)
    }

    fn next_token(&mut self) -> Result<Token> {
        use TokenKind::*;
        let start = self.curr_byte_idx;
        let current_char = match self.seek()? {
            Some(c) => c,
            None => {
                return Ok(Token {
                    kind: EOF,
                    span: Span {
                        start: self.curr_byte_idx,
                        end: self.curr_byte_idx + 1,
                    },
                    line: self.line,
                    col: self.col,
                });
            }
        };
        let kind = match current_char {
            _ if current_char.is_whitespace() => {
                if current_char == '\n' {
                    self.col = 1;
                    self.line += 1;
                }
                return self.next_token();
            }
            '(' => LParen,
            ')' => RParen,
            '{' => LBrace,
            '}' => RBrace,
            '[' => LBracket,
            ']' => RBracket,
            ',' => Comma,
            ';' => Semicolon,
            '.' => match self.consume_if_match('.', Dot, Rng)? {
                Rng => self.consume_if_match('=', Rng, RngInc)?,
                _ => Dot,
            },
            ':' => match self.consume_if_match(':', Colon, ColonColon)? {
                Colon => self.consume_if_match('=', Colon, ShortAssign)?,
                _ => ColonColon,
            },
            '+' => match self.consume_if_match('+', Add, Inc)? {
                Add => self.consume_if_match('=', Add, AddAssign)?,
                _ => Inc,
            },
            '-' => match self.consume_if_match('-', Sub, Dec)? {
                Sub => match self.consume_if_match('=', Sub, SubAssign)? {
                    Sub => self.consume_if_match('>', Sub, ArrowRight)?,
                    _ => SubAssign,
                },
                _ => Dec,
            },
            '*' => self.consume_if_match('=', Star, MulAssign)?,
            '/' => {
                let kind = match self.consume_if_match('/', Div, Comment)? {
                    Comment => self.parse_comment()?,
                    _ => self.consume_if_match('=', Div, DivAssign)?,
                };
                kind
            }
            '%' => self.consume_if_match('=', Mod, ModAssign)?,
            '=' => self.consume_if_match('=', Assign, Equal)?,
            '|' => match self.consume_if_match('=', BitOr, BitOrAssign)? {
                BitOr => self.consume_if_match('|', BitOr, Or)?,
                _ => BitOrAssign,
            },
            '&' => match self.consume_if_match('=', Amp, BitAndAssign)? {
                Amp => self.consume_if_match('&', Amp, And)?,
                _ => BitAndAssign,
            },
            '^' => self.consume_if_match('=', Xor, XorAssign)?,
            '~' => TokenKind::BitNot,
            '!' => self.consume_if_match('=', Not, NotEqual)?,
            '<' => match self.consume_if_match('=', Less, LessEq)? {
                Less => match self.consume_if_match('<', Less, ShL)? {
                    ShL => self.consume_if_match('=', ShL, ShLAssign)?,
                    _ => self.consume_if_match('-', Less, ArrowLeft)?,
                },
                _ => LessEq,
            },
            '>' => match self.consume_if_match('=', Great, GreatEq)? {
                Great => match self.consume_if_match('>', Great, ShR)? {
                    ShR => self.consume_if_match('=', ShR, ShRAssign)?,
                    _ => Great,
                },
                _ => GreatEq,
            },
            '"' => return self.parse_string(),
            '\'' => return self.parse_char(),
            '0'..='9' => return self.parse_number(current_char),
            '_' | 'a'..='z' | 'A'..='Z' => self.parse_identifier_or_keyword(current_char)?,
            _ => todo!(),
        };
        Ok(Token {
            kind,
            span: Span {
                start,
                end: self.curr_byte_idx,
            },
            line: self.line,
            col: self.col,
        })
    }
    fn parse_identifier_or_keyword(&mut self, first_char: char) -> Result<TokenKind> {
        let mut result_string = String::from(first_char);
        while let Some(c) = self.peek(0)
            && is_alphanumeric(c)
        {
            result_string.push(c);
            self.seek()?;
        }
        Ok(identifier_or_keyword(&result_string))
    }
    fn parse_number(&mut self, first_char: char) -> Result<Token> {
        let start = self.curr_byte_idx - 1; // decrementing by 1 because first digit was read
        let mut is_float = false;
        let mut num_chars = vec![first_char];
        while let Some(c) = self.peek(0) {
            match c {
                '_' => {
                    if let Some(lc) = num_chars.last()
                        && *lc == '_'
                    {
                        return Err(Error::LexerError {
                            cause: "\"_\" must separate successive digit".into(),
                            line: self.line,
                            column: self.col,
                        });
                    }
                }
                '.' => {
                    if let Some(ch) = self.peek(1)
                        && !ch.is_numeric()
                    {
                        break;
                    }
                    if is_float {
                        break;
                    }
                    num_chars.push(c);
                    is_float = true;
                }
                '0'..='9' => num_chars.push(c),
                _ => break,
            }
            self.seek()?;
        }
        let num_str = num_chars.iter().collect::<String>();
        let kind = if is_float {
            TokenKind::FloatLit(num_str.parse::<f64>().map_err(|e| Error::LexerError {
                cause: format!("float parsing error: {e}"),
                line: self.line,
                column: self.col,
            })?)
        } else {
            TokenKind::IntLit(num_str.parse::<i128>().map_err(|e| Error::LexerError {
                cause: format!("int parsing error: {e}"),
                line: self.line,
                column: self.col,
            })?)
        };
        Ok(Token {
            kind,
            span: Span {
                start,
                end: self.curr_byte_idx + 1,
            },
            line: self.line,
            col: self.col,
        })
    }

    fn parse_char(&mut self) -> Result<Token> {
        let start = self.curr_byte_idx;
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
                    Ok(Token {
                        kind: TokenKind::Char(c),
                        span: Span {
                            start,
                            end: self.curr_byte_idx,
                        },
                        line: self.line,
                        col: self.col,
                    })
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

    fn parse_string(&mut self) -> Result<Token> {
        let start = self.curr_byte_idx - 1; // decrementing by 1 because " was already read
        let start_col = self.col;
        let mut string_chars = Vec::<char>::new();
        while let Some(char) = self.seek()? {
            match char {
                '"' => {
                    return Ok(Token {
                        kind: TokenKind::StringLit,
                        span: Span {
                            start: start,
                            end: self.curr_byte_idx,
                        },
                        line: self.line,
                        col: start_col,
                    });
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

/// retuns next token.
/// NOTE: it always returns Some(Result<Token>),
/// if there are no chars in input anymore it repeatedly returns Some(EOF)
impl<R: io::Read> Iterator for Lexer<R> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_token())
    }
}

#[cfg(test)]
mod test {
    use crate::lexer::{Lexer, TokenKind};
    use crate::utf8_reader::Utf8Reader;
    use std::io::Read;

    const TOKENS_STR_SET: &str = r#"
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
true
false
let
const
if
else
struct
for
return
in
some_identifier
"some_string"
'c'
123
123.123
1_2_3
_
1_2_3.1_2_3
for _ in 0..2
"#;

    fn new_lexer(input: &str) -> Lexer<&[u8]> {
        Lexer::new(Utf8Reader::new(input.as_bytes().bytes()))
            .expect("unexpecetd error while creating lexer")
    }

    #[test]
    fn simple_tokens() {
        use TokenKind::*;
        let expected_token_kinds = vec![
            LParen,
            RParen,
            LBrace,
            RBrace,
            LBracket,
            RBracket,
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
            Struct,
            For,
            Return,
            In,
            Identifier,
            StringLit,
            Char('c'),
            //
            IntLit(123),
            FloatLit(123.123),
            IntLit(123),
            Underscore,
            FloatLit(123.123),
            For,
            Underscore,
            In,
            IntLit(0),
            Rng,
            IntLit(2),
            EOF,
        ];

        let lexer = new_lexer(TOKENS_STR_SET);

        let ident = "some_identifier";
        let string_literal_content = "some_string";

        for (token_result, expected_token_kind) in lexer.zip(expected_token_kinds) {
            let token = token_result
                .map_err(|e| {
                    format!(
                        "unexpeceted error, expected token {:?}, got error: {:?}",
                        expected_token_kind, e
                    )
                })
                .unwrap();
            match token.kind {
                Identifier => assert_eq!(token.span.to_string(TOKENS_STR_SET), ident),
                StringLit => assert_eq!(
                    token.span.as_string_literal(TOKENS_STR_SET),
                    string_literal_content
                ),
                _ => assert_eq!(token.kind, expected_token_kind),
            }
            match (token.kind, expected_token_kind) {
                (Char(g), Char(e)) => assert_eq!(g, e),
                (_, _) => (),
            }
        }
    }

    #[test]
    fn empty_input() {
        use TokenKind::*;
        let mut lexer = new_lexer("");
        if let Some(token_result) = lexer.next() {
            assert_eq!(token_result.unwrap().kind, EOF)
        }
    }

    #[test]
    fn string() {
        use TokenKind::*;
        let src = "\"this is a string\"";
        let mut lexer = new_lexer(src);
        if let Some(token) = lexer
            .next()
            .transpose()
            .expect("expected string literal token")
        {
            assert_eq!(token.kind, StringLit);
            assert_eq!(
                src.get(token.span.start..token.span.end)
                    .expect("string span is wrong"),
                src
            )
        }
    }

    #[test]
    fn numbers() {
        use TokenKind::*;
        let lexer = new_lexer("12 12.12 12..12 12.1_2, 1 2 3 1.a");
        let expected_token_kinds = vec![
            IntLit(12),
            FloatLit(12.12),
            IntLit(12),
            Rng,
            IntLit(12),
            FloatLit(12.12),
            Comma,
            IntLit(1),
            IntLit(2),
            IntLit(3),
            IntLit(1),
            Dot,
            Identifier,
            EOF,
        ];
        for (i, (got, expected)) in lexer.zip(expected_token_kinds).enumerate().into_iter() {
            match got {
                Ok(t) => assert!(
                    t.kind == expected,
                    "tokens don't match at index {i}\nExpected:\n{:?}\nGot:\n{:?}\n",
                    expected,
                    t.kind,
                ),
                Err(e) => panic!("unexpecetd: {e:?}"),
            }
        }
    }

    #[test]
    fn spans() {
        use TokenKind::*;
        let mut lexer = new_lexer(TOKENS_STR_SET);
        let mut idx = 0;
        while let Some(token) = lexer
            .next()
            .transpose()
            .expect("unexpected error while parsing tokens")
            && token.kind != EOF
        {
            match token.kind {
                IntLit(_) | FloatLit(_) | UintType(_) | Char(_) | Identifier | StringLit => (),
                _ => assert_eq!(
                    token.span.to_string(TOKENS_STR_SET),
                    token.kind.to_string().as_str()
                ),
            }
            idx += 1;
        }
    }
}
