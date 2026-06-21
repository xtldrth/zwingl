use crate::tokenizer::{Error as LexerError, Lexer, Token, TokenKind};
use core::fmt;
use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
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
    RepeatedNonassociativeOP,
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
    pub fn unexpected_token_with_expected_kind(token: Token, expected: TokenKind) -> Self {
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
    Rng,
    RngInc,
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
}

#[derive(Debug, Clone, Copy)]
pub enum AssignOp {
    Common,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BOr,
    BAnd,
    Shl,
    Shr,
    Xor,
}

impl Display for AssignOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AssignOp::*;
        match self {
            Common => write!(f, "=",),
            Add => write!(f, "+=",),
            Sub => write!(f, "-=",),
            Mul => write!(f, "*=",),
            Div => write!(f, "/=",),
            Mod => write!(f, "%=",),
            BOr => write!(f, "|=",),
            BAnd => write!(f, "&=",),
            Shl => write!(f, "<<=",),
            Shr => write!(f, ">>=",),
            Xor => write!(f, "^=",),
        }
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BinOp::*;
        match self {
            Dot => write!(f, ".",),
            Rng => write!(f, "..",),
            RngInc => write!(f, "..=",),
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
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UnPrefOp {
    Plus,
    Minus,
    Ref,
    DRef,
    Not,
    BNot,
}

impl Display for UnPrefOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plus => write!(f, "+",),
            Self::Minus => write!(f, "-",),
            Self::Ref => write!(f, "&",),
            Self::DRef => write!(f, "*",),
            Self::Not => write!(f, "!",),
            Self::BNot => write!(f, "~",),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UnPostOp {
    Inc,
    Dec,
}

impl Display for UnPostOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
    UnaryPrefix {
        op: UnPrefOp,
        rhs: Box<Expr>,
    },
    UnaryPostfix {
        op: UnPostOp,
        lhs: Box<Expr>,
    },
    Assign {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: AssignOp,
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
            Self::Binary { lhs, rhs, op } => write!(f, "({op} {lhs} {rhs})"),
            Self::Assign { lhs, rhs, op } => write!(f, "({op} {lhs} {rhs})"),
            Self::UnaryPrefix { rhs, op } => write!(f, "({op} {rhs})"),
            Self::UnaryPostfix { lhs, op } => write!(f, "({lhs} {op})"),
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

pub struct Ast<L> {
    lexer: Lexer<L>,
    next_token: Option<Token>,
}

impl<L: io::Read> Ast<L> {
    pub fn new(lexer: Lexer<L>) -> Result<Self, Error> {
        let mut a = Self {
            lexer: lexer,
            next_token: None,
        };
        a.next_token = match a.lexer.next() {
            Some(res) => match res {
                Ok(t) => Some(t),
                Err(e) => return Err(Error::LexerError(e)),
            },
            None => unreachable!(),
        };
        Ok(a)
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

    fn token_to_prefix_un_op(token: Token) -> Option<UnPrefOp> {
        use crate::tokenizer::TokenKind::*;
        use UnPrefOp as OP;
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

    fn token_to_postfix_un_op(kind: TokenKind) -> Option<UnPostOp> {
        use crate::tokenizer::TokenKind::*;
        use UnPostOp as OP;
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
            Range => OP::Rng,
            RangeIncl => OP::RngInc,
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
            _ => return None,
        })
    }

    fn token_to_assign_op(kind: TokenKind) -> Option<AssignOp> {
        use crate::tokenizer::TokenKind::*;
        use AssignOp as OP;
        Some(match kind {
            Assign => OP::Common,
            AddAssign => OP::Add,
            SubAssign => OP::Sub,
            MulAssign => OP::Mul,
            DivAssign => OP::Div,
            ModAssign => OP::Mod,
            BitOrAssign => OP::BOr,
            BitAndAssign => OP::BAnd,
            ShLAssign => OP::Shl,
            ShRAssign => OP::Shr,
            XorAssign => OP::Xor,
            _ => return None,
        })
    }

    fn prefix_binding_power(op: UnPrefOp) -> ((), u8) {
        use UnPrefOp::*;
        match op {
            Plus | Minus | Ref | DRef | Not | BNot => ((), 200),
        }
    }

    fn postfix_binding_power(op: UnPostOp) -> (u8, ()) {
        use UnPostOp::*;
        match op {
            Inc | Dec => (200, ()),
        }
    }
    fn infix_binding_power(op: BinOp) -> (u8, u8) {
        use BinOp::*;
        match op {
            ColonColon => (220, 219),
            Dot => (210, 209),
            Mul | Div | Mod => (149, 150),
            Add | Sub => (129, 130),
            Shl | Shr => (119, 120),
            BAnd => (109, 110),
            Xor => (99, 100),
            BOr => (89, 90),

            // nonassociative
            Eq | Ne | Lt | Le | Gt | Ge => (80, 80),

            And => (69, 70),
            Or => (59, 60),
            Rng | RngInc => (50, 50),
        }
    }

    fn assign_binding_power(op: AssignOp) -> (u8, u8) {
        use AssignOp::*;
        match op {
            Common | Add | Sub | Mul | Div | Mod | BOr | BAnd | Shl | Shr | Xor => (40, 40),
        }
    }

    fn eat(&mut self, kind: TokenKind) -> Result<(), Error> {
        let token = self.seek()?;
        if token.kind() == kind {
            return Ok(());
        }
        Err(Error::unexpected_token_with_expected_kind(token, kind))
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
                let ((), rbp) = Self::prefix_binding_power(op);
                self.expr_bp(rbp)?
            }
            _ => todo!(),
        };
        loop {
            let token = self.peek()?;
            if token.kind() == EOF {
                break;
            }
            if let Some(op) = Self::token_to_postfix_un_op(token.kind()) {
                let (l_bp, ()) = Self::postfix_binding_power(op);
                if l_bp < min_bp {
                    break;
                }
                self.seek()?;
                lhs = Expr::UnaryPostfix {
                    op,
                    lhs: Box::new(lhs),
                };
                continue;
            }
            if let Some(op) = Self::token_to_infix_bin_op(token.kind()) {
                let (l_bp, r_bp) = Self::infix_binding_power(op);
                if l_bp == min_bp {
                    return Err(Error::RepeatedNonassociativeOP);
                }
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
            if let Some(op) = Self::token_to_assign_op(token.kind()) {
                let (l_bp, r_bp) = Self::assign_binding_power(op);
                if l_bp == min_bp {
                    return Err(Error::RepeatedNonassociativeOP);
                }
                if l_bp < min_bp {
                    break;
                }
                self.seek()?;
                lhs = Expr::Assign {
                    lhs: Box::new(lhs),
                    rhs: Box::new(self.expr_bp(r_bp)?),
                    op,
                };
                continue;
            }
            break;
        }
        Ok(lhs)
    }

    pub fn build(&mut self) -> Result<(), Error> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use crate::utf8_reader::Utf8Reader;
    use crate::{ast::Ast, tokenizer::Lexer};
    use std::io::{Cursor, Read};

    fn new_ast_builder(input: &str) -> Ast<Cursor<Vec<u8>>> {
        let cursor = Cursor::new(input.as_bytes().to_vec());
        let bytes_iter = cursor.bytes();
        let reader = Utf8Reader::new(bytes_iter);
        let lexer = Lexer::new(reader).expect("unexpected error while creating lexer");
        let ast = Ast::new(lexer).expect("unexpected error while creating ast builder");
        ast
    }

    #[test]
    fn basic_expression() {
        let expr = "1 + (2 + a) * 12 + 2";
        let mut ast_builder = new_ast_builder(expr);
        let expr = ast_builder
            .expr()
            .map_err(|e| format!("unexpeceted error: {:?}", e))
            .unwrap();

        assert_eq!(format!("{expr}"), "(+ (+ 1 (* (+ 2 a) 12)) 2)");
    }

    #[test]
    fn expressions() {
        let expr = "a  .  b   +   12   *   12  +   n   |   12";
        let mut ast_builder = new_ast_builder(expr);
        let expr = ast_builder
            .expr()
            .map_err(|e| format!("unexpeceted error: {:?}", e))
            .unwrap();

        assert_eq!(format!("{expr}"), "(| (+ (+ (. a b) (* 12 12)) n) 12)");
    }
}
