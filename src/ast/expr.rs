use super::{
    Block,
    Error,
    Result,
    // Statement,
    TokensReader,
    arena,
};
use crate::ast::ForLoop;
use crate::lexer::{self, TokenKind};

use std::fmt::{self, format};
use std::io;

#[derive(Debug, Clone, Copy)]
pub enum InfixOp {
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

impl InfixOp {
    fn binding_power(&self) -> (u8, u8) {
        use InfixOp::*;
        match self {
            ColonColon => (230, 229),
            Dot => (220, 219),
            Mul | Div | Mod => (149, 150),
            Add | Sub => (129, 130),
            Shl | Shr => (119, 120),
            BAnd => (109, 110),
            Xor => (99, 100),
            BOr => (89, 90),
            And => (69, 70),
            Or => (59, 60),

            // nonassociative
            Eq | Ne | Lt | Le | Gt | Ge => (80, 80),
            Rng | RngInc => (50, 50),
        }
    }
}

impl TryFrom<TokenKind> for InfixOp {
    type Error = ();
    fn try_from(value: TokenKind) -> std::result::Result<Self, Self::Error> {
        use crate::lexer::TokenKind::*;
        use InfixOp as OP;
        Ok(match value {
            Dot => OP::Dot,
            ColonColon => OP::ColonColon,
            Rng => OP::Rng,
            RngInc => OP::RngInc,
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
            _ => return Err(()),
        })
    }
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

impl AssignOp {
    pub fn binding_power(&self) -> (u8, u8) {
        use AssignOp::*;
        match self {
            Common | Add | Sub | Mul | Div | Mod | BOr | BAnd | Shl | Shr | Xor => (40, 40),
        }
    }
}

impl TryFrom<TokenKind> for AssignOp {
    type Error = ();
    fn try_from(value: TokenKind) -> std::result::Result<Self, Self::Error> {
        use crate::lexer::TokenKind::*;
        use AssignOp as OP;
        Ok(match value {
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
            _ => return Err(()),
        })
    }
}

impl fmt::Display for AssignOp {
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

impl fmt::Display for InfixOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InfixOp::*;
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
pub enum PrefixOp {
    Plus,
    Minus,
    Ref,
    DRef,
    Not,
    BNot,
}

impl PrefixOp {
    pub fn binding_power(&self) -> ((), u8) {
        use PrefixOp::*;
        match self {
            Plus | Minus | Ref | DRef | Not | BNot => ((), 200),
        }
    }
}

impl TryFrom<TokenKind> for PrefixOp {
    type Error = ();
    fn try_from(value: TokenKind) -> std::result::Result<Self, Self::Error> {
        use crate::lexer::TokenKind::*;
        use PrefixOp as OP;
        Ok(match value {
            Add => OP::Plus,
            Sub => OP::Minus,
            Amp => OP::Ref,
            Star => OP::DRef,
            BitNot => OP::BNot,
            Not => OP::Not,
            _ => return Err(()),
        })
    }
}

impl fmt::Display for PrefixOp {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PostfixOp {
    Inc,
    Dec,
}

impl PostfixOp {
    pub fn binding_power(&self) -> (u8, ()) {
        use PostfixOp::*;
        match self {
            Inc | Dec => (200, ()),
        }
    }
}

impl TryFrom<TokenKind> for PostfixOp {
    type Error = ();
    fn try_from(value: TokenKind) -> std::result::Result<Self, Self::Error> {
        use crate::lexer::TokenKind::*;
        use PostfixOp as OP;
        Ok(match value {
            Inc => OP::Inc,
            Dec => OP::Dec,
            _ => return Err(()),
        })
    }
}

impl fmt::Display for PostfixOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inc => write!(f, "++",),
            Self::Dec => write!(f, "--",),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Expr {
    /// expression + expression
    InfixOp(InfixOp, arena::ExprsId, arena::ExprsId),

    /// -expression
    PrefixOp(
        PrefixOp,
        /// expr id
        arena::ExprsId,
    ),

    /// expression++
    PostfixOp(PostfixOp, arena::ExprsId),

    Call(arena::ExprsId, arena::ExprsId), // TODO: test
    Idx(arena::ExprsId, arena::ExprsId), // TODO: test

    /// expression = expression
    /// op is +=, -= and so on
    Assign(AssignOp, arena::ExprsId, arena::ExprsId), // TODO: test

    /// if condition { statements } else { statements }
    If(arena::ExprsId, Block, Option<arena::ExprsId>), // TODO: test

    For(ForLoop), // TODO: test

    Return(arena::ExprsId), // TODO: test

    /// identifiers or literals
    Atom(Atom), // TODO: test

    /// {
    ///     statements
    ///     ...
    /// }
    Block(Block), // TODO: test
    Sequence(arena::ExprsId), // TODO: test
}

impl Expr {
    pub fn parse(
        reader: &mut TokensReader<impl io::Read>,
        arena: &mut arena::Arena,
    ) -> Result<Self> {
        Self::expr_bp(reader, arena, 0)
    }

    fn expr_bp(
        reader: &mut TokensReader<impl io::Read>,
        arena: &mut arena::Arena,
        min_bp: u8,
    ) -> Result<Expr> {
        use Atom::*;
        let token = reader.seek()?;
        let mut lhs = match token.kind {
            TokenKind::StringLit => Expr::Atom(StrLit(token.span)),
            TokenKind::IntLit(s) => Expr::Atom(Atom::IntLit(s)),
            TokenKind::FloatLit(s) => Expr::Atom(Atom::FloatLit(s)),
            TokenKind::Identifier => Expr::Atom(Ident(token.span)),
            TokenKind::Underscore => Expr::Atom(Underscore),
            TokenKind::Return => Expr::Return({
                let s = Self::sequence(reader, arena, None)?;
                arena.alloc_expr(s)
            }),
            TokenKind::LParen => {
                let mut lhs = Self::expr_bp(reader, arena, 0)?;
                if reader.peek(0).kind == TokenKind::Comma {
                    reader.seek()?;
                    lhs = Self::sequence(reader, arena, Some(lhs))?;
                }
                reader.eat(TokenKind::RParen)?;
                lhs
            }
            _ if let Ok(op) = PrefixOp::try_from(token.kind) => {
                let ((), rbp) = op.binding_power();
                let rhs = Self::expr_bp(reader, arena, rbp)?;
                Expr::PrefixOp(op, arena.alloc_expr(rhs))
            }

            TokenKind::If => {
                reader.eat(TokenKind::If)?;
                let condition = Self::parse(reader, arena)?;
                let block = Block::parse(reader, arena)?;
                if reader.peek(0).kind == TokenKind::Else {
                    reader.seek()?;
                    todo!()
                }
                return Ok(Self::If(arena.alloc_expr(condition), block, None));
            }

            TokenKind::For => return Ok(Expr::For(ForLoop::parse(reader, arena)?)),

            _ => return Err(Error::unexpected_token(token)),
        };
        loop {
            let token = reader.peek(0);
            if token.kind == TokenKind::EOF {
                break;
            }

            if let Ok(op) = PostfixOp::try_from(token.kind) {
                let (l_bp, ()) = op.binding_power();
                if l_bp < min_bp {
                    break;
                }
                reader.seek()?;
                lhs = Expr::PostfixOp(op, arena.alloc_expr(lhs));
                continue;
            }

            if let Ok(op) = InfixOp::try_from(token.kind) {
                let (l_bp, r_bp) = op.binding_power();
                if l_bp == min_bp {
                    return Err(Error::RepeatedNonassociativeOP);
                }
                if l_bp < min_bp {
                    break;
                }
                reader.seek()?;
                let rhs = Self::expr_bp(reader, arena, r_bp)?;
                lhs = Expr::InfixOp(op, arena.alloc_expr(lhs), arena.alloc_expr(rhs));
                continue;
            }

            if let Ok(op) = AssignOp::try_from(token.kind) {
                let (l_bp, r_bp) = op.binding_power();
                if l_bp == min_bp {
                    return Err(Error::RepeatedNonassociativeOP);
                }
                if l_bp < min_bp {
                    break;
                }
                reader.seek()?;
                let rhs = Self::expr_bp(reader, arena, r_bp)?;
                lhs = Expr::Assign(op, arena.alloc_expr(lhs), arena.alloc_expr(rhs));
                continue;
            }
            match token.kind {
                TokenKind::LParen => {
                    let rhs = Self::sequence(reader, arena, None)?;
                    lhs = Expr::Call(arena.alloc_expr(lhs), arena.alloc_expr(rhs));
                    reader.eat(TokenKind::RParen)?;
                    continue;
                }
                TokenKind::LBracket => {
                    let rhs = Expr::parse(reader, arena)?;
                    lhs = Expr::Idx(arena.alloc_expr(lhs), arena.alloc_expr(rhs));
                    reader.eat(TokenKind::RBracket)?;
                    continue;
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn sequence(
        reader: &mut TokensReader<impl io::Read>,
        arena: &mut arena::Arena,
        first: Option<Expr>,
    ) -> Result<Self> {
        let mut sequence = match first {
            Some(e) => vec![e],
            None => vec![],
        };
        loop {
            use TokenKind::*;
            match reader.peek(0).kind {
                RParen | RBrace | RBracket => break,
                _ => sequence.push(Self::parse(reader, arena)?),
            }
            if reader.peek(0).kind != Comma {
                break;
            }
            reader.seek()?;
        }
        Ok(Self::Sequence(arena.alloc_exprs(&mut sequence)))
    }

    pub fn to_string(&self, src: &str, arena: &arena::Arena) -> String {
        match self {
            Self::Atom(a) => match a {
                Atom::Ident(s) => s.to_string(src).into(),
                Atom::StrLit(s) => s.as_string_literal(src).into(),
                Atom::FloatLit(n) => format!("{n}"),
                Atom::IntLit(n) => format!("{n}"),
                Atom::Underscore => "_".into(),
            },
            Self::InfixOp(op, lhs, rhs) => {
                format!(
                    "({} {} {})",
                    op,
                    arena.get_expr(*lhs).to_string(src, arena),
                    arena.get_expr(*rhs).to_string(src, arena),
                )
            }
            Self::Assign(op, lhs, rhs) => {
                format!(
                    "({} {} {})",
                    op,
                    arena.get_expr(*lhs).to_string(src, arena),
                    arena.get_expr(*rhs).to_string(src, arena)
                )
            }
            Self::PrefixOp(op, rhs) => {
                format!("({op} {})", arena.get_expr(*rhs).to_string(src, arena))
            }
            Self::PostfixOp(op, lhs) => {
                format!("({} {op})", arena.get_expr(*lhs).to_string(src, arena))
            }
            Self::Sequence(s) => {
                let exrps = arena.get_exprs(*s);
                let mut result = String::new();
                for (i, e) in exrps.iter().enumerate() {
                    if i > 0 {
                        result += ", "
                    }
                    result += e.to_string(src, arena).as_str();
                }
                result
            }
            Self::Call(e, s) => {
                format!(
                    "{}({})",
                    arena.get_expr(*e).to_string(src, arena),
                    arena.get_expr(*s).to_string(src, arena)
                )
            }
            Self::Idx(e, i) => {
                format!(
                    "{}({})",
                    arena.get_expr(*e).to_string(src, arena),
                    arena.get_expr(*i).to_string(src, arena)
                )
            }
            Self::If(c, b, e) => {
                format!(
                    "if {} {} {}",
                    arena.get_expr(*c).to_string(src, arena),
                    b.to_string(src, arena),
                    match e {
                        Some(e) => format!("else {}", arena.get_expr(*e).to_string(src, arena)),
                        None => "".into(),
                    }
                )
            }
            Self::For(f) => f.to_string(src, arena),
            Self::Return(s) => format!("return {}", arena.get_expr(*s).to_string(src, arena)),
            Self::Block(b) => b.to_string(src, arena),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Atom {
    Ident(lexer::Span),
    IntLit(i128),
    FloatLit(f64),
    StrLit(lexer::Span),
    Underscore,
}

#[cfg(test)]
mod test {

    use crate::ast::{Expr, TokensReader, arena};
    use crate::lexer::Lexer;
    use crate::utf8_reader::Utf8Reader;
    use std::io::Read;

    fn setup_arena() -> arena::Arena {
        arena::Arena::new()
    }

    fn setup_reader(src: &str) -> TokensReader<&[u8]> {
        TokensReader::new(
            Lexer::new(
                Utf8Reader::new(src.as_bytes().bytes())
            ).expect("unexpected error while creating lexer")
        ).expect("unexpected error while creating ast builder")
    }
    fn setup(src: &str) -> (TokensReader<&[u8]>, arena::Arena) {
        (setup_reader(src), setup_arena())
    }

    #[test]
    fn basic_expressions() {
        let sources_and_results = vec![
            (
                "1 + (2 + a) * 12 + 2",
                "(+ (+ 1 (* (+ 2 a) 12)) 2)",
            ),
            (
                "(+-a  .  b   +   12   *   12  +   n   |   12)   +   \"some_string\"",
                "(+ (| (+ (+ (+ (- (. a b))) (* 12 12)) n) 12) some_string)"
            ),
        ];
        let mut arena = setup_arena();
        sources_and_results.iter().for_each(
            |(source, expected_output)| {
                let mut reader = setup_reader(source);
                let expr = Expr::parse(&mut reader, &mut arena).unwrap();
                assert_eq!(
                    format!("{}", expr.to_string(source, &arena)),
                    *expected_output
                );
                arena.clear()
            }
        )
    }
}
