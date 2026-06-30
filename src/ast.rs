use crate::tokenizer::TokenKind::EOF;
use crate::tokenizer::{Error as LexerError, Lexer, Token, TokenKind};
use std::fmt::Display;
use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    LexerError(LexerError),
    UnexpectedToken {
        got: TokenKind,
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
            got: token.kind(),
            expected: None,
            col: token.starts_at_column,
            line: token.starts_at_line,
        };
    }

    pub fn unexpected_token_with_expected_kind(token: Token, expected: TokenKind) -> Self {
        return Self::UnexpectedToken {
            got: token.kind(),
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

impl BinOp {
    fn binding_power(&self) -> (u8, u8) {
        use BinOp::*;
        match self {
            ColonColon => (220, 219),
            Dot => (210, 209),
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

impl TryFrom<TokenKind> for BinOp {
    type Error = ();
    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        use crate::tokenizer::TokenKind::*;
        use BinOp as OP;
        Ok(match value {
            Dot => OP::Dot,
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
    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        use crate::tokenizer::TokenKind::*;
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

impl UnPrefOp {
    pub fn binding_power(&self) -> ((), u8) {
        use UnPrefOp::*;
        match self {
            Plus | Minus | Ref | DRef | Not | BNot => ((), 200),
        }
    }
}

impl TryFrom<TokenKind> for UnPrefOp {
    type Error = ();
    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        use crate::tokenizer::TokenKind::*;
        use UnPrefOp as OP;
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

impl UnPostOp {
    pub fn binding_power(&self) -> (u8, ()) {
        use UnPostOp::*;
        match self {
            Inc | Dec => (200, ()),
        }
    }
}

impl TryFrom<TokenKind> for UnPostOp {
    type Error = ();
    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        use crate::tokenizer::TokenKind::*;
        use UnPostOp as OP;
        Ok(match value {
            Inc => OP::Inc,
            Dec => OP::Dec,
            _ => return Err(()),
        })
    }
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

#[derive(Clone)]
pub struct Range {
    ident: Option<String>,
    from: Expr,
    to: Expr,
}

#[derive(Clone)]
pub struct Block {
    statements: Vec<Statement>,
}

impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;
        for statment in self.statements.iter() {
            writeln!(f, "\t{statment}")?;
        }
        writeln!(f, "}}")
    }
}

#[derive(Clone)]
pub enum Params {
    Explicit {
        identifiers: Vec<String>,
        types: Vec<Type>,
    },
    Short(Vec<Type>), // contains only types
}

impl Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        match self {
            Self::Explicit { identifiers, types } => {
                for (i, (ident, type_)) in identifiers.iter().zip(types.iter()).enumerate() {
                    if i == 0 {
                        write!(f, "{ident}: {type_}")?;
                        continue;
                    }
                    write!(f, ", {ident}: {type_}")?;
                }
                write!(f, ")")
            }
            Self::Short(types) => {
                for (i, t) in types.iter().enumerate() {
                    if i == 0 {
                        write!(f, "{t}")?;
                        continue;
                    }
                    write!(f, ", {t}")?;
                }
                write!(f, ")")
            }
        }
    }
}

pub type ReturnTypes = Params;

#[derive(Clone)]
pub enum Type {
    Int(usize),
    Uint(usize),
    Float(usize),
    String,
    Identifier(String),
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String => write!(f, "str"),
            Self::Identifier(i) => write!(f, "{i}"),
            Self::Uint(s) => write!(f, "u{s}"),
            Self::Int(s) => write!(f, "i{s}"),
            Self::Float(s) => write!(f, "f{s}"),
        }
    }
}

impl TryFrom<TokenKind> for Type {
    type Error = ();
    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        use TokenKind::*;
        Ok(match value {
            IntType(size) => Self::Int(size),
            UintType(size) => Self::Uint(size),
            FloatType(size) => Self::Float(size),
            StringType => Self::String,
            Identifier(t) => Self::Identifier(t),
            _ => return Err(()),
        })
    }
}
#[derive(Clone)]
pub struct FnSignature {
    ident: String,
    params: Params,
    return_types: ReturnTypes,
}

impl Display for FnSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} :: {} {}", self.ident, self.params, self.return_types)
    }
}

#[derive(Clone)]
pub enum TypeDeclaration {
    Alias {
        ident: String,
        parent: Type,
    },
    Enum {
        ident: String,
        fields: Vec<String>,
    },
    Struct {
        ident: String,
        fields: Vec<String>,
        types: Vec<String>,
    },

    FnSignature(FnSignature),
}

impl Display for TypeDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Clone)]
pub enum Declaration {
    Fn {
        signature: FnSignature,
        block: Block,
    },
    ExplicitVar {
        ident: String,
        value: Option<Expr>,
        type_: Type,
    },
    ShortVar {
        ident: String,
        value: Expr,
    },
    Type(TypeDeclaration),
}

impl Display for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fn { signature, block } => {
                write!(f, "{signature} {block}")
            }
            Self::ExplicitVar {
                ident,
                value,
                type_,
            } => {
                write!(f, "let {ident}: {type_}")?;
                if let Some(value) = value {
                    write!(f, "{value}")?;
                }
                Ok(())
            }
            Self::ShortVar { ident, value } => write!(f, "{ident} := {value}"),
            Self::Type(t) => write!(f, "{t}"),
        }
    }
}

#[derive(Clone)]
pub enum Statement {
    If {
        condition: Expr,
        block: Block,
    },
    ForIn {
        lhs: Expr,
        rhs: Expr,
        block: Block,
    },
    ForC {
        ident: Option<String>,
        condition: Option<Expr>,
        update: Option<Expr>,
        block: Block,
    },
    ForWhile {
        condition: Option<Expr>,
        block: Block,
    },
    Return(Vec<Expr>),
    Declaration(Declaration),
    Expr(Expr),
}

impl Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::If { condition, block } => write!(f, "if {condition} {block}"),
            Self::ForIn { lhs, rhs, block } => write!(f, "for {lhs} in {rhs} {block}"),
            Self::ForC {
                ident,
                condition,
                update,
                block,
            } => {
                todo!()
            }
            Self::ForWhile { condition, block } => {
                write!(
                    f,
                    "for {} {block}",
                    if let Some(condition) = condition {
                        condition.to_string()
                    } else {
                        "".to_string()
                    }
                )
            }
            Self::Return(exprs) => {
                for (i, expr) in exprs.iter().enumerate() {
                    if i == 0 {
                        write!(f, "{expr}")?;
                        continue;
                    }
                    write!(f, ", {expr}")?;
                }
                Ok(())
            }
            Self::Declaration(d) => write!(f, "{d}"),
            Self::Expr(e) => write!(f, "{e}"),
        }
    }
}

pub struct Program {
    declarations: Vec<Declaration>,
}

impl Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for declaration in self.declarations.iter() {
            write!(f, "{declaration}")?;
        }
        Ok(())
    }
}

pub struct AstBuilder<L> {
    lexer: Lexer<L>,
    next_token: Option<Token>,
    second_next: Option<Token>,
    third_next: Option<Token>,
}

impl<L: io::Read> AstBuilder<L> {
    pub fn new(lexer: Lexer<L>) -> Result<Self, Error> {
        let mut a = Self {
            lexer: lexer,
            next_token: None,
            second_next: None,
            third_next: None,
        };
        a.next_token = match a.lexer.next() {
            Some(res) => match res {
                Ok(t) => Some(t),
                Err(e) => return Err(Error::LexerError(e)),
            },
            None => unreachable!(),
        };
        a.second_next = match a.lexer.next() {
            Some(res) => match res {
                Ok(t) => Some(t),
                Err(e) => return Err(Error::LexerError(e)),
            },
            None => unreachable!(),
        };
        a.third_next = match a.lexer.next() {
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

    fn peek_2nd(&mut self) -> Option<Token> {
        self.second_next.clone()
    }
    fn peek_3nd(&mut self) -> Option<Token> {
        self.third_next.clone()
    }

    fn seek(&mut self) -> Result<Token, Error> {
        let token = if let Some(token) = self.next_token.clone() {
            token
        } else {
            return Err(Error::TryToGetTokenAfterEOF);
        };
        self.next_token = self.second_next.clone();
        self.second_next = self.third_next.clone();
        self.third_next = match self.lexer.next() {
            Some(res) => match res {
                Err(e) => return Err(Error::LexerError(e)),
                Ok(t) => Some(t),
            },
            None => None,
        };
        Ok(token)
    }

    fn seek_n(&mut self, n: usize) -> Result<Token, Error> {
        for _ in 0..n - 1 {
            self.seek()?;
        }
        self.seek()
    }
    fn eat(&mut self, kind: TokenKind) -> Result<(), Error> {
        let token = self.seek()?;
        if token.kind() == kind {
            return Ok(());
        }
        Err(Error::unexpected_token_with_expected_kind(token, kind))
    }

    fn get_identifier(&mut self) -> Result<String, Error> {
        let token = self.peek()?;
        let identifier = match token.kind() {
            TokenKind::Identifier(i) => i,
            _ => {
                return Err(Error::unexpected_token_with_expected_kind(
                    token,
                    TokenKind::Identifier("".into()),
                ));
            }
        };
        self.seek()?;
        Ok(identifier)
    }

    fn explicit_variable_declaration(&mut self) -> Result<Declaration, Error> {
        self.eat(TokenKind::Let)?;
        let ident = self.get_identifier()?;
        self.eat(TokenKind::Colon)?;
        let token = self.peek()?;
        let type_ = match Type::try_from(token.kind()) {
            Ok(t) => t,
            Err(_) => return Err(Error::unexpected_token(token)),
        };
        self.seek()?;
        if self.peek()?.kind() != TokenKind::Assign {
            return Ok(Declaration::ExplicitVar {
                ident,
                value: None,
                type_,
            });
        }
        self.eat(TokenKind::Assign)?;
        Ok(Declaration::ExplicitVar {
            ident,
            value: Some(self.expr()?),
            type_,
        })
    }

    fn short_variable_declaration(&mut self) -> Result<Declaration, Error> {
        let ident = self.get_identifier()?;
        self.eat(TokenKind::ShortAssign)?;
        let expr = self.expr()?;
        Ok(Declaration::ShortVar { ident, value: expr })
    }

    fn type_declaration(&mut self) -> Result<Declaration, Error> {
        todo!()
    }

    fn declaration(&mut self) -> Result<Declaration, Error> {
        if self.peek()?.kind() == TokenKind::Let {
            return self.explicit_variable_declaration();
        }
        if let Some(delimiter) = self.peek_2nd() {
            if delimiter.kind() == TokenKind::ShortAssign {
                return self.short_variable_declaration();
            }
            if delimiter.kind() != TokenKind::ColonColon {
                return Err(Error::unexpected_token_with_expected_kind(
                    delimiter,
                    TokenKind::ColonColon,
                ));
            }
        }
        if let Some(t) = self.peek_3nd()
            && t.kind() == TokenKind::LParen
        {
            return self.func_or_signature_declaration();
        }
        self.type_declaration()
    }

    fn block(&mut self) -> Result<Block, Error> {
        self.eat(TokenKind::LBrace)?;
        let mut statements = Vec::<Statement>::new();
        loop {
            let next_token = self.peek()?;
            match next_token.kind() {
                TokenKind::EOF => {
                    return Err(Error::unexpected_token(next_token));
                }
                TokenKind::RBrace => {
                    self.seek()?;
                    return Ok(Block { statements });
                }
                _ => {
                    statements.push(self.statement()?);
                }
            }
        }
    }

    fn fn_params(&mut self) -> Result<Params, Error> {
        if let token = self.peek()?
            && token.kind() != TokenKind::LParen
        {
            return Err(Error::unexpected_token_with_expected_kind(
                token,
                TokenKind::LParen,
            ));
        }
        self.seek()?;
        let next_token = self.peek()?;
        if next_token.kind() == TokenKind::RParen {
            self.seek()?;
            return Ok(Params::Short(Vec::new()));
        }

        // parsing short declared params, because only a type were provided
        if next_token.kind() == TokenKind::Identifier("".into())
            && let Some(t) = self.peek_2nd()
            && t.kind() == TokenKind::Colon
        {
            let mut params: Vec<Type> = Vec::new();
            while let next = self.peek()?
                && next.kind() != TokenKind::RParen
            {
                match next.kind() {
                    TokenKind::EOF => return Err(Error::unexpected_token(next)),
                    _ => match Type::try_from(next.kind()) {
                        Ok(type_) => params.push(type_),
                        Err(_) => return Err(Error::unexpected_token(next)),
                    },
                }
                self.seek()?;
            }
            self.seek()?;
            return Ok(Params::Short(params));
        }

        // parsing explicit params
        let mut identifiers: Vec<String> = Vec::new();
        let mut types: Vec<Type> = Vec::new();
        loop {
            match self.peek() {
                Ok(t) => match t.kind() {
                    TokenKind::Identifier(i) => {
                        self.seek()?;
                        identifiers.push(i);
                    }
                    TokenKind::RParen => {
                        self.seek()?;
                        break;
                    }
                    _ => return Err(Error::unexpected_token(t)),
                },
                Err(e) => return Err(e),
            }
            let token = self.peek()?;
            match Type::try_from(token.kind()) {
                Ok(type_) => types.push(type_),
                Err(_) => return Err(Error::unexpected_token(token)),
            };
        }
        Ok(Params::Explicit { identifiers, types })
    }

    fn fn_return_types(&mut self) -> Result<ReturnTypes, Error> {
        if self.peek()?.kind() != TokenKind::ArrowRight {
            return Ok(ReturnTypes::Short(Vec::new()));
        }
        self.seek()?;
        let next_token = self.peek()?;
        match next_token.kind() {
            TokenKind::EOF => return Err(Error::unexpected_token(next_token)),
            TokenKind::LParen => (),
            _ if let Ok(t) = Type::try_from(next_token.kind()) => {
                self.seek()?;
                return Ok(ReturnTypes::Short(vec![t]));
            }
            _ => return Err(Error::unexpected_token(next_token)),
        }
        if let Ok(_) = self.peek()
            && let Some(second) = self.peek_2nd()
            && (second.kind() == TokenKind::Comma || second.kind() == TokenKind::RParen)
        {
            let mut types = Vec::<Type>::new();
            while let Ok(next_token) = self.peek() {
                if next_token.kind() == TokenKind::RParen {
                    self.seek()?;
                    break;
                }
                match Type::try_from(next_token.kind()) {
                    Ok(t) => types.push(t),
                    Err(_) => return Err(Error::unexpected_token(next_token)),
                }
                self.seek()?;
                let next_token = self.peek()?;
                if next_token.kind() == TokenKind::RParen {
                    self.seek()?;
                    break;
                }
                self.eat(TokenKind::Comma)?;
            }
            return Ok(ReturnTypes::Short(types));
        }
        let mut identifiers = Vec::<String>::new();
        let mut types = Vec::<Type>::new();
        loop {
            match self.peek() {
                Ok(t) => match t.kind() {
                    TokenKind::Identifier(i) => identifiers.push(i),
                    TokenKind::RParen => {
                        self.seek()?;
                        break;
                    }
                    _ => return Err(Error::unexpected_token(t)),
                },
                Err(e) => return Err(e),
            }
            match Type::try_from(next_token.kind()) {
                Ok(t) => types.push(t),
                Err(_) => return Err(Error::unexpected_token(next_token)),
            }
            if next_token.kind() == TokenKind::RParen {
                self.seek()?;
                break;
            }
            self.eat(TokenKind::Comma)?;
        }
        Ok(ReturnTypes::Explicit { identifiers, types })
    }

    fn func_or_signature_declaration(&mut self) -> Result<Declaration, Error> {
        let ident = self.get_identifier()?;
        self.eat(TokenKind::ColonColon)?;
        let params = self.fn_params()?;
        let return_types = self.fn_return_types()?;
        let signature = FnSignature {
            ident,
            params,
            return_types,
        };
        if self.peek()?.kind() != TokenKind::LBrace {
            return Ok(Declaration::Type(TypeDeclaration::FnSignature(signature)));
        }
        let block = self.block()?;
        Ok(Declaration::Fn { signature, block })
    }

    fn if_statement(&mut self) -> Result<Statement, Error> {
        self.eat(TokenKind::If)?;
        let condition = self.expr()?;
        let block = self.block()?;
        Ok(Statement::If { condition, block })
    }

    fn for_in_statement(&mut self) -> Result<Statement, Error> {
        let ident = self.get_identifier()?;
        self.eat(TokenKind::In)?;
        let expr = self.expr()?;
        todo!();
        // Ok(Statement::ForIn {
        //     lhs: ident,
        //     expr: expr,
        //     block: self.block()?,
        // })
    }
    fn for_c_statement(&mut self) -> Result<Statement, Error> {
        todo!()
    }
    fn for_while_statement(&mut self) -> Result<Statement, Error> {
        if self.peek()?.kind() == TokenKind::LBrace {
            return Ok(Statement::ForWhile {
                condition: None,
                block: self.block()?,
            });
        }
        let condition = self.expr()?;
        let block = self.block()?;
        Ok(Statement::ForWhile {
            condition: Some(condition),
            block,
        })
    }
    fn for_statement(&mut self) -> Result<Statement, Error> {
        self.eat(TokenKind::For)?;
        let next_token = self.peek()?;
        if next_token.kind() == TokenKind::Let || next_token.kind() == TokenKind::Semicolon {
            return self.for_c_statement();
        }
        if next_token.kind() == TokenKind::LBrace {
            return Ok(Statement::ForWhile {
                condition: None,
                block: self.block()?,
            });
        }
        let expr = self.expr()?;
        if self.peek()?.kind() == TokenKind::In {
            self.seek()?;
            let rhs = self.expr()?;
            return Ok(Statement::ForIn {
                lhs: expr,
                rhs,
                block: self.block()?,
            });
        }
        Ok(Statement::ForWhile {
            condition: Some(expr),
            block: self.block()?,
        })
    }
    fn return_statement(&mut self) -> Result<Statement, Error> {
        self.eat(TokenKind::Return)?;
        let mut return_exprs = Vec::<Expr>::new();
        loop {
            let next_token = self.peek()?;
            match next_token.kind() {
                TokenKind::RBrace => return Ok(Statement::Return(return_exprs)),
                TokenKind::EOF => return Err(Error::unexpected_token(next_token)),
                TokenKind::Comma => {}
                _ => {
                    return_exprs.push(self.expr()?);
                    let next_token = self.peek()?;
                    match next_token.kind() {
                        TokenKind::Comma => {
                            self.seek()?;
                        }
                        _ => continue,
                    }
                }
            }
        }
    }

    fn statement(&mut self) -> Result<Statement, Error> {
        let next_token = self.peek()?;
        Ok(match next_token.kind() {
            TokenKind::If => self.if_statement()?,
            TokenKind::For => self.for_statement()?,
            TokenKind::Return => self.return_statement()?,
            TokenKind::Let => Statement::Declaration(self.declaration()?),
            TokenKind::Identifier(_) => {
                if let Some(t) = self.peek_2nd()
                    && (t.kind() == TokenKind::ColonColon || t.kind() == TokenKind::ShortAssign)
                {
                    return Ok(Statement::Declaration(self.declaration()?));
                }
                return Ok(Statement::Expr(self.expr()?));
            }
            _ => Statement::Expr(self.expr()?),
        })
    }

    fn expr(&mut self) -> Result<Expr, Error> {
        self.expr_bp(0)
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
            _ if let Ok(op) = UnPrefOp::try_from(token.kind()) => {
                let ((), rbp) = op.binding_power();
                self.expr_bp(rbp)?
            }
            _ => todo!(),
        };
        loop {
            let token = self.peek()?;
            if token.kind() == EOF {
                break;
            }

            // postfix unary op
            if let Ok(op) = UnPostOp::try_from(token.kind()) {
                let (l_bp, ()) = op.binding_power();
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

            // infix binary op
            if let Ok(op) = BinOp::try_from(token.kind()) {
                let (l_bp, r_bp) = op.binding_power();
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

            // assign op
            if let Ok(op) = AssignOp::try_from(token.kind()) {
                let (l_bp, r_bp) = op.binding_power();
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

    fn program(&mut self) -> Result<Program, Error> {
        let mut declarations = Vec::<Declaration>::new();
        loop {
            if self.peek()?.kind() == EOF {
                return Ok(Program { declarations });
            }
            declarations.push(self.declaration()?);
        }
    }

    pub fn build(&mut self) -> Result<Program, Error> {
        self.program()
    }
}

#[cfg(test)]
mod test {
    use super::{
        Atom, BinOp, Block, Declaration, Expr, FnSignature, Params, Program, ReturnTypes,
        Statement, Type,
    };
    use crate::utf8_reader::Utf8Reader;
    use crate::{ast::AstBuilder, tokenizer::Lexer};
    use std::io::{Cursor, Read};

    fn new_ast_builder(input: &str) -> AstBuilder<Cursor<Vec<u8>>> {
        let cursor = Cursor::new(input.as_bytes().to_vec());
        let bytes_iter = cursor.bytes();
        let reader = Utf8Reader::new(bytes_iter);
        let lexer = Lexer::new(reader).expect("unexpected error while creating lexer");
        let ast = AstBuilder::new(lexer).expect("unexpected error while creating ast builder");
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

    #[test]
    fn program() {
        let expr = r#"
foo :: () -> i8 {
    let a: i8 = 12
    b := a + 4
    return b * b
}
"#;
        let mut ast_builder = new_ast_builder(expr);
        let result = ast_builder
            .build()
            .map_err(|e| format!("unexpeceted error: {:?}", e))
            .unwrap()
            .to_string();
        let declarations = vec![Declaration::Fn {
            signature: FnSignature {
                ident: "foo".into(),
                params: Params::Short(Vec::new()),
                return_types: ReturnTypes::Short(vec![Type::Int(8)]),
            },
            block: Block {
                statements: vec![
                    Statement::Declaration(Declaration::ExplicitVar {
                        ident: "a".into(),
                        value: Some(Expr::Atom(Atom::IntLit(12))),
                        type_: Type::Int(8),
                    }),
                    Statement::Declaration(Declaration::ShortVar {
                        ident: "b".into(),
                        value: Expr::Binary {
                            lhs: Box::new(Expr::Atom(Atom::Ident("a".into()))),
                            rhs: Box::new(Expr::Atom(Atom::IntLit(4))),
                            op: BinOp::Add,
                        },
                    }),
                    Statement::Return(vec![Expr::Binary {
                        lhs: Box::new(Expr::Atom(Atom::Ident("b".to_string()))),
                        rhs: Box::new(Expr::Atom(Atom::Ident("b".to_string()))),
                        op: BinOp::Mul,
                    }]),
                ],
            },
        }];
        let expected_program = Program {
            declarations: declarations,
        };
        assert_eq!(result, expected_program.to_string())
    }
}
