use super::{Error, Result};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub(crate) fn to_string(self, src: &str) -> &str {
        src.get(self.start..self.end).expect(
            format!(
                "Span[{}..{}] to string unexpected error",
                self.start, self.end
            )
            .as_str(),
        )
    }
    pub(crate) fn as_string_literal(self, src: &str) -> &str {
        src.get(self.start + 1..self.end - 1).expect(
            format!(
                "Span[{}..{}] as string literal unexpected error",
                self.start, self.end
            )
            .as_str(),
        )
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TOKEN: [ {} ]", self.kind)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TokenKind {
    LParen,      // (
    RParen,      // )
    LBrace,      // {
    RBrace,      // }
    LBracket,     // [
    RBracket,     // ]
    Comma,       // ,
    Dot,         // .
    Underscore,  // _
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

    StringLit,
    Char(char), // starts with '  TODO: add special symbols support like '\n' and so on
    IntLit(i128),
    FloatLit(f64), // TODO: add this format .01
    True,          //  true
    False,         //  false

    Let,        //  let
    Const,      // const
    If,         // if
    Else,       //  else
    Struct,     // struct
    For,        //  for
    Return,     //  return
    In,         //  in
    Identifier, //  starts with _ or any letter and can contain any letter or digit or '_'

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

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::LParen => "(",
                Self::RParen => ")",
                Self::LBrace => "{",
                Self::RBrace => "}",
                Self::LBracket => "[",
                Self::RBracket => "]",
                Self::Comma => ",",
                Self::Dot => ".",
                Self::ArrowLeft => "<-",
                Self::ArrowRight => "->",
                Self::Rng => "..",
                Self::RngInc => "..=",
                Self::Colon => ":",
                Self::ColonColon => "::",
                Self::Semicolon => ";",
                Self::Add => "+",
                Self::Inc => "++",
                Self::Sub => "-",
                Self::Dec => "--",
                Self::Star => "*",
                Self::Div => "/",
                Self::Mod => "%",
                Self::Assign => "=",
                Self::ShortAssign => ":=",

                Self::AddAssign => "+=",
                Self::SubAssign => "-=",
                Self::MulAssign => "*=",
                Self::DivAssign => "/=",
                Self::ModAssign => "%=",
                Self::BitOrAssign => "|=",
                Self::BitAndAssign => "&=",
                Self::ShLAssign => "<<=",
                Self::ShRAssign => ">>=",
                Self::XorAssign => "^=",
                Self::Equal => "==",
                Self::BitOr => "|",
                Self::Or => "||",
                Self::Amp => "&",
                Self::And => "&&",
                Self::Less => "<",
                Self::LessEq => "<=",
                Self::Great => ">",
                Self::GreatEq => ">=",
                Self::ShL => "<<",
                Self::ShR => ">>",
                Self::BitNot => "~",
                Self::Xor => "^",
                Self::Not => "!",
                Self::NotEqual => "!=",
                Self::Comment => "//",
                Self::StringLit => "STRING",
                Self::Char(_) => "CHAR",
                Self::IntLit(_) => "INT_LITERAL",
                Self::FloatLit(_) => "FLOAT_LITERAL",
                Self::True => "true",
                Self::False => "false",
                Self::Let => "let",
                Self::Const => "const",
                Self::If => "if",
                Self::Else => "else",
                Self::Struct => "struct",
                Self::For => "for",
                Self::Return => "return",
                Self::In => "in",
                Self::Identifier => "IDENT",
                Self::FloatType(_) => "FLOAT_TYPE",
                Self::IntType(_) => "INT_TYPE",
                Self::UintType(_) => "UINT_TYPE",
                Self::StringType => "STR_TYPE",
                Self::EOF => "EOF",
                Self::Underscore => "_",
            }
        )
    }
}
