use crate::{
    ast::BinOp::{Add, Sub},
    tokenizer::{
        Error as LexerError, Lexer, Token,
        TokenKind::{self, EOF},
    },
};
use core::fmt;
use std::{
    fmt::{Display, write},
    io,
};

#[derive(Debug)]
enum Error {
    LexerError(LexerError),
    UnexpectedToken {
        kind: TokenKind,
        expected: Option<TokenKind>,
        col: usize,
        line: usize,
    },
    BadToken {
        col: usize,
        line: usize,
        cause: Option<String>,
    },
    TryToGetTokenAfterEOF,
}

impl Error {
    pub fn unexpected_token(token: Token) -> Self {
        return Self::UnexpectedToken {
            kind: token.kind(),
            expected: None,
            col: token.starts_at_column,
            line: token.starts_at_line,
        };
    }
    pub fn unexpected_token_with_expected(token: Token, expected: TokenKind) -> Self {
        return Self::UnexpectedToken {
            kind: token.kind(),
            expected: Some(expected),
            col: token.starts_at_column,
            line: token.starts_at_line,
        };
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Dot,
    Range,
    RangeIncl,
    ColonColon,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    BOr,
    BAnd,
    Shl,
    Shr,
    Xor,

    Assign,
    AddA,
    SubA,
    MulA,
    DivA,
    ModA,
    BOrA,
    BAndA,
    ShlA,
    ShrA,
    XorA,
}

impl Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BinOp::*;
        match self {
            Dot => write!(f, ".",),
            Range => write!(f, "..",),
            RangeIncl => write!(f, "..=",),
            ColonColon => write!(f, "::",),
            Add => write!(f, "+",),
            Sub => write!(f, "-",),
            Mul => write!(f, "*",),
            Div => write!(f, "/",),
            Mod => write!(f, "%",),
            And => write!(f, "&&",),
            Or => write!(f, "||",),
            Eq => write!(f, "==",),
            Ne => write!(f, "!=",),
            Lt => write!(f, "<",),
            Le => write!(f, ">=",),
            Gt => write!(f, ">",),
            Ge => write!(f, ">=",),
            BOr => write!(f, "|",),
            BAnd => write!(f, "&",),
            Shl => write!(f, "<<",),
            Shr => write!(f, ">>",),
            Xor => write!(f, "^",),

            Assign => write!(f, "=",),
            AddA => write!(f, "+=",),
            SubA => write!(f, "-=",),
            MulA => write!(f, "*=",),
            DivA => write!(f, "/=",),
            ModA => write!(f, "%=",),
            BOrA => write!(f, "|=",),
            BAndA => write!(f, "&=",),
            ShlA => write!(f, "<<=",),
            ShrA => write!(f, ">>=",),
            XorA => write!(f, "^=",),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    // prefix OPs
    Plus,
    Minus,
    Ref,
    DRef,
    Not,
    BNot,

    // postfix OPs
    Inc,
    Dec,
}

impl Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plus => write!(f, "+",),
            Self::Minus => write!(f, "-",),
            Self::Ref => write!(f, "&",),
            Self::DRef => write!(f, "*",),
            Self::Not => write!(f, "!",),
            Self::BNot => write!(f, "~",),
            Self::Inc => write!(f, "++",),
            Self::Dec => write!(f, "--",),
        }
    }
}

#[derive(Clone)]
pub enum Expr {
    Binary {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: BinOp,
    },
    Unary {
        op: UnOp,
        operand: Box<Expr>,
    },
    Atom(Atom),
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atom(a) => match a {
                Atom::Ident(s) | Atom::StrLit(s) => write!(f, "{s}",),
                Atom::FloatLit(n) => write!(f, "{n}",),
                Atom::IntLit(n) => write!(f, "{n}",),
                Atom::StructInit {
                    ident: _,
                    fields_values: _,
                } => todo!(),
            },
            Self::Binary { lhs, rhs, op } => write!(f, "({lhs} {op} {rhs})"),
            Self::Unary { operand, op } => {
                use UnOp::*;
                match op {
                    Inc | Dec => write!(f, "({operand}{op})"),
                    _ => write!(f, "({op}{operand})"),
                }
            }
        }
    }
}

#[derive(Clone)]
pub enum Atom {
    Ident(String),
    IntLit(i128),
    FloatLit(f64),
    StrLit(String),
    StructInit {
        ident: Option<String>,
        fields_values: Vec<(String, Box<Atom>)>,
    },
}

pub struct Ast<'a, L> {
    lexer: &'a mut Lexer<'a, L>,
    next_token: Option<Token>,
}

impl<'a, L: io::Read> Ast<'a, L> {
    pub fn new(lexer: &'a mut Lexer<'a, L>) -> Result<Self, Error> {
        let next_token = lexer.next();
        Ok(Self {
            lexer: lexer,
            next_token: match next_token {
                Some(res) => match res {
                    Ok(t) => Some(t),
                    Err(e) => return Err(Error::LexerError(e)),
                },
                None => unreachable!(),
            },
        })
    }

    fn peek(&mut self) -> Result<Token, Error> {
        match self.next_token.clone() {
            Some(t) => Ok(t),
            None => Err(Error::TryToGetTokenAfterEOF),
        }
    }

    fn seek(&mut self) -> Result<Token, Error> {
        let token = if let Some(token) = self.next_token.clone() {
            token
        } else {
            return Err(Error::TryToGetTokenAfterEOF);
        };
        self.next_token = match self.lexer.next() {
            Some(res) => match res {
                Err(e) => return Err(Error::LexerError(e)),
                Ok(t) => Some(t),
            },
            // None => Err(Error::TryToGetTokenAfterEOF),
            None => panic!("trying to get a token after EOF reached"),
        };
        Ok(token)
    }

    fn expr(&mut self) -> Result<Expr, Error> {
        self.expr_bp(0)
    }

    fn token_to_prefix_un_op(token: Token) -> Option<UnOp> {
        use crate::tokenizer::TokenKind::*;
        use UnOp as OP;
        Some(match token.kind() {
            Add => OP::Plus,
            Sub => OP::Minus,
            Amp => OP::Ref,
            Star => OP::DRef,
            BitNot => OP::BNot,
            Not => OP::Not,
            _ => return None,
        })
    }

    fn token_to_postfix_un_op(kind: TokenKind) -> Option<UnOp> {
        use crate::tokenizer::TokenKind::*;
        use UnOp as OP;
        Some(match kind {
            Inc => OP::Inc,
            Dec => OP::Dec,
            _ => return None,
        })
    }

    fn token_to_infix_bin_op(kind: TokenKind) -> Option<BinOp> {
        use crate::tokenizer::TokenKind::*;
        use BinOp as OP;
        Some(match kind {
            Dot => OP::Dot,
            Range => OP::Range,
            RangeIncl => OP::RangeIncl,
            ColonColon => OP::ColonColon,
            Add => OP::Add,
            Sub => OP::Sub,
            Star => OP::Mul,
            Div => OP::Div,
            Mod => OP::Mod,
            And => OP::And,
            Or => OP::Or,
            Equal => OP::Eq,
            NotEqual => OP::Ne,
            Less => OP::Lt,
            LessEq => OP::Le,
            Great => OP::Gt,
            GreatEq => OP::Ge,
            BitOr => OP::BOr,
            Amp => OP::BAnd,
            ShL => OP::Shl,
            ShR => OP::Shr,
            Xor => OP::Xor,

            Assign => OP::Assign,
            AddAssign => OP::AddA,
            SubAssign => OP::SubA,
            MulAssign => OP::MulA,
            DivAssign => OP::DivA,
            ModAssign => OP::ModA,
            BitOrAssign => OP::BOrA,
            BitAndAssign => OP::BAndA,
            ShLAssign => OP::ShlA,
            ShRAssign => OP::ShrA,
            XorAssign => OP::XorA,
            _ => return None,
        })
    }

    fn prefix_binding_power(op: UnOp) -> Option<((), u8)> {
        use UnOp::*;
        match op {
            Plus | Minus | Ref | DRef | Not | BNot => Some(((), 200)),
            Inc | Dec => None,
        }
    }

    fn postfix_binding_power(op: UnOp) -> Option<(u8, ())> {
        use UnOp::*;
        match op {
            Inc | Dec => Some((200, ())),
            _ => None,
        }
    }
    fn infix_binding_power(op: BinOp) -> (u8, u8) {
        use BinOp::*;
        match op {
            ColonColon => (220, 219),
            Dot => (210, 209),
            Mul | Div | Mod => (150, 149),
            Add | Sub => (130, 129),
            Shl | Shr => (120, 119),
            BAnd => (110, 109),
            Xor => (100, 99),
            BOr => (90, 89),

            // non associative
            Eq | Ne | Lt | Le | Gt | Ge => (80, 80),

            And => (70, 69),
            Or => (60, 59),
            Range | RangeIncl => (50, 49),
            // TODO: think about assign later
            Assign | AddA | SubA | MulA | DivA | ModA | BOrA | BAndA | ShlA | ShrA | XorA => {
                (40, 39)
            }
        }
    }

    fn eat(&mut self, kind: TokenKind) -> Result<(), Error> {
        let token = self.seek()?;
        if token.kind() == kind {
            return Ok(());
        }
        Err(Error::unexpected_token_with_expected(token, kind))
    }

    fn expr_bp(&mut self, min_bp: u8) -> Result<Expr, Error> {
        use crate::tokenizer::TokenKind::*;
        use Atom::*;
        let token = self.seek()?;
        let mut lhs = match token.kind() {
            String(s) => Expr::Atom(StrLit(s)),
            Int(s) => Expr::Atom(IntLit(s)),
            Float(s) => Expr::Atom(FloatLit(s)),
            Identifier(s) => Expr::Atom(Ident(s)),
            LParen => {
                let lhs = self.expr_bp(0)?;
                self.eat(RParen)?;
                lhs
            }
            _ if let Some(op) = Self::token_to_prefix_un_op(token.clone()) => {
                let lhs = if let Some(((), rbp)) = Self::prefix_binding_power(op) {
                    self.expr_bp(rbp)?
                } else {
                    return Err(Error::BadToken {
                        col: token.starts_at_column,
                        line: token.starts_at_line,
                        cause: Some(format!("expected prefix unary op, got {:?}", token.kind())),
                    });
                };
                lhs
            }
            _ => todo!(),
        };
        loop {
            let token = self.peek()?;
            match token.kind() {
                EOF => break,
                _ if let Some(op) = Self::token_to_postfix_un_op(token.kind()) => {
                    if let Some((l_bp, ())) = Self::postfix_binding_power(op) {
                        if l_bp < min_bp {
                            break;
                        }
                        self.seek()?;
                        lhs = Expr::Unary {
                            op,
                            operand: Box::new(lhs),
                        };
                        continue;
                    }
                }
                _ if let Some(op) = Self::token_to_infix_bin_op(token.kind()) => {
                    let (l_bp, r_bp) = Self::infix_binding_power(op);
                    if l_bp < min_bp {
                        break;
                    }
                    self.seek()?;
                    lhs = Expr::Binary {
                        lhs: Box::new(lhs),
                        rhs: Box::new(self.expr_bp(r_bp)?),
                        op,
                    };
                    continue;
                }
                _ => {
                    break;
                    // return Err(Error::BadToken {
                    //     col: token.clone().starts_at_column,
                    //     line: token.clone().starts_at_line,
                    //     cause: Some(format!("expected infix binary op, got {:?}", token.kind())),
                    // });
                }
            }
        }
        Ok(lhs)
    }

    pub fn build(&mut self) -> Result<Expr, Error> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use crate::utf8_reader::Utf8Reader;
    use crate::{ast::Ast, tokenizer::Lexer};
    use std::io::{Cursor, Read};

    #[test]
    fn basic_expression() {
        let expr = "1 + (2 + a) * 12".to_string();
        let mut reader = Utf8Reader::new(Cursor::new(expr.clone().into_bytes()).bytes());
        let mut lexer = Lexer::new(&mut reader).expect("unexpecetd error while creating lexer");
        let mut ast_builder =
            Ast::new(&mut lexer).expect("unexpecetd error while creating ast builder");
        let expr = ast_builder
            .expr()
            .map_err(|e| format!("unexpeceted error: {:?}", e))
            .unwrap();

        assert_eq!(format!("{expr}"), "(1 + ((2 + a) * 12))");
    }
}
