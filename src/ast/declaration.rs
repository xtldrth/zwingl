use super::{Error, Expr, Result, TokensReader, arena};
use crate::{
    ast::Block,
    lexer::{Span, Token, TokenKind},
};
use std::fmt::Write;
use std::io;

#[derive(Clone, Copy, Debug)]
pub enum Params {
    Explicit(
        arena::IdentsId, // identifiers id
        arena::TypesId,  // types id
    ),
    Short(arena::TypesId), // types id
}

impl Params {
    pub fn parse(
        reader: &mut TokensReader<impl io::Read>,
        arena: &mut arena::Arena,
    ) -> Result<Self> {
        reader.eat(TokenKind::LParen)?;
        let next_token = reader.peek(0);
        if next_token.kind == TokenKind::RParen {
            reader.seek()?;
            return Ok(Params::Short(arena.alloc_types(Vec::new().as_mut())));
        }

        // parsing short declared params, because only a type were provided
        if next_token.kind == TokenKind::Identifier
            && let t = reader.peek(1)
            && (t.kind == TokenKind::Comma || t.kind == TokenKind::RParen)
        {
            let mut params: Vec<Type> = Vec::new();
            while let next = reader.peek(0)
                && next.kind != TokenKind::RParen
            {
                match next.kind {
                    TokenKind::EOF => return Err(Error::unexpected_token(next)),
                    _ => match Type::try_from(next) {
                        Ok(type_) => params.push(type_),
                        Err(_) => return Err(Error::unexpected_token(next)),
                    },
                }
                reader.seek()?;
            }
            reader.seek()?;
            return Ok(Params::Short(arena.alloc_types(&mut params)));
        }

        // parsing explicit params
        let mut identifiers: Vec<Span> = Vec::new();
        let mut types: Vec<Type> = Vec::new();
        loop {
            let token = reader.peek(0);
            match token.kind {
                TokenKind::Identifier => {
                    reader.seek()?;
                    identifiers.push(token.span);
                }
                TokenKind::RParen => {
                    reader.seek()?;
                    break;
                }
                _ => return Err(Error::unexpected_token(token)),
            }
            let token = reader.seek()?;
            match Type::try_from(token) {
                Ok(type_) => types.push(type_),
                Err(_) => return Err(Error::unexpected_token(token)),
            };
        }
        Ok(Params::Explicit(
            arena.alloc_identifiers(&mut identifiers),
            arena.alloc_types(&mut types),
        ))
    }

    pub fn to_string(&self, src: &str, arena: &arena::Arena) -> String {
        let mut result_string = String::new();
        write!(&mut result_string, "(").unwrap();
        match self {
            Self::Explicit(i, t) => arena
                .get_identifiers(*i)
                .iter()
                .zip(arena.get_types(*t))
                .enumerate()
                .for_each(|(i, (ident, type_))| {
                    if i > 0 {
                        write!(
                            &mut result_string,
                            ", {}: {}",
                            ident.to_string(src),
                            type_.to_string(src)
                        )
                        .unwrap()
                    }
                }),
            Self::Short(t) => arena
                .get_types(*t)
                .iter()
                .enumerate()
                .for_each(|(i, type_)| {
                    if i > 0 {
                        write!(&mut result_string, ", {}", type_.to_string(src)).unwrap()
                    } else {
                        write!(&mut result_string, "{}", type_.to_string(src)).unwrap()
                    }
                }),
        }
        write!(&mut result_string, ")").unwrap();
        result_string
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ReturnTypes {
    Explicit(
        arena::IdentsId, // identifiers id
        arena::TypesId,  // types id
    ),
    Short(arena::TypesId), // types id
}

impl ReturnTypes {
    pub fn parse(
        reader: &mut TokensReader<impl io::Read>,
        arena: &mut arena::Arena,
    ) -> Result<Self> {
        if reader.peek(0).kind != TokenKind::ArrowRight {
            return Ok(ReturnTypes::Short(arena.alloc_types(Vec::new().as_mut())));
        }
        reader.seek()?;
        let next_token = reader.peek(0);
        match next_token.kind {
            TokenKind::EOF => return Err(Error::unexpected_token(next_token)),
            TokenKind::LParen => (),
            _ if let Ok(t) = Type::try_from(next_token) => {
                reader.seek()?;
                return Ok(ReturnTypes::Short(arena.alloc_types(vec![t].as_mut())));
            }
            _ => return Err(Error::unexpected_token(next_token)),
        }
        if let second = reader.peek(1)
            && (second.kind == TokenKind::Comma || second.kind == TokenKind::RParen)
        {
            let mut types = Vec::<Type>::new();
            loop {
                let next_token = reader.peek(0);
                if next_token.kind == TokenKind::RParen {
                    reader.seek()?;
                    break;
                }
                match Type::try_from(next_token) {
                    Ok(t) => types.push(t),
                    Err(_) => return Err(Error::unexpected_token(next_token)),
                }
                reader.seek()?;
                let next_token = reader.peek(0);
                if next_token.kind == TokenKind::RParen {
                    reader.seek()?;
                    break;
                }
                reader.eat(TokenKind::Comma)?;
            }
            return Ok(ReturnTypes::Short(arena.alloc_types(&mut types)));
        }
        let mut identifiers = Vec::<Span>::new();
        let mut types = Vec::<Type>::new();
        loop {
            let t = reader.peek(0);
            match t.kind {
                TokenKind::Identifier => identifiers.push(t.span),
                TokenKind::RParen => {
                    reader.seek()?;
                    break;
                }
                _ => return Err(Error::unexpected_token(t)),
            }
            match Type::try_from(next_token) {
                Ok(t) => types.push(t),
                Err(_) => return Err(Error::unexpected_token(next_token)),
            }
            if next_token.kind == TokenKind::RParen {
                reader.seek()?;
                break;
            }
            reader.eat(TokenKind::Comma)?;
        }
        Ok(ReturnTypes::Explicit(
            arena.alloc_identifiers(&mut identifiers),
            arena.alloc_types(&mut types),
        ))
    }
    pub fn to_string(&self, src: &str, arena: &arena::Arena) -> String {
        let mut result_string = String::new();

        match self {
            Self::Explicit(i, t) => {
                write!(&mut result_string, "->").unwrap();
                write!(&mut result_string, "(").unwrap();
                arena
                    .get_identifiers(*i)
                    .iter()
                    .zip(arena.get_types(*t))
                    .enumerate()
                    .for_each(|(i, (ident, type_))| {
                        if i > 0 {
                            write!(
                                &mut result_string,
                                ", {}: {}",
                                ident.to_string(src),
                                type_.to_string(src)
                            )
                            .unwrap()
                        }
                    });
                write!(&mut result_string, "(").unwrap();
            }
            Self::Short(t) => {
                let types = arena.get_types(*t);
                if types.len() > 0 {
                    write!(&mut result_string, "-> ").unwrap();
                }
                if types.len() > 1 {
                    write!(&mut result_string, "(").unwrap();
                    types.iter().enumerate().for_each(|(i, type_)| {
                        if i > 0 {
                            write!(&mut result_string, ", {}", type_.to_string(src)).unwrap()
                        }
                    });
                    write!(&mut result_string, ")").unwrap();
                } else if types.len() == 1 {
                    write!(&mut result_string, "{}", types[0].to_string(src)).unwrap()
                }
            }
        }
        result_string
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Type {
    Int(usize),
    Uint(usize),
    Float(usize),
    String,
    Identifier(Span),
}

impl Type {
    pub fn to_string(&self, src: &str) -> String {
        match self {
            Self::Int(s) => format!("i{}", s),
            Self::Uint(s) => format!("u{}", s),
            Self::Float(s) => format!("f{}", s),
            Self::String => "str".to_string(),
            Self::Identifier(s) => s.to_string(src).into(),
        }
    }
}

impl TryFrom<Token> for Type {
    type Error = ();
    fn try_from(value: Token) -> std::result::Result<Self, Self::Error> {
        Ok(match value.kind {
            TokenKind::IntType(size) => Self::Int(size),
            TokenKind::UintType(size) => Self::Uint(size),
            TokenKind::FloatType(size) => Self::Float(size),
            TokenKind::StringType => Self::String,
            TokenKind::Identifier => Self::Identifier(value.span),
            _ => return Err(()),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Signature(Span, Params, ReturnTypes);

impl Signature {
    fn parse(reader: &mut TokensReader<impl io::Read>, arena: &mut arena::Arena) -> Result<Self> {
        let ident = reader.seek_expected(TokenKind::Identifier)?.span;
        reader.eat(TokenKind::ColonColon)?;
        let params = Params::parse(reader, arena)?;
        let return_types = ReturnTypes::parse(reader, arena)?;
        Ok(Signature(ident, params, return_types))
    }

    pub fn to_string(&self, src: &str, arena: &arena::Arena) -> String {
        let Signature(span, params, return_types) = self;
        format!(
            "{} :: {} {}",
            span.to_string(src),
            params.to_string(src, arena),
            return_types.to_string(src, arena)
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TypeDeclaration {
    Alias(Span, Type),
    Enum(
        Span,
        arena::FieldsId, // fields id
    ),
    Struct(
        Span,
        arena::FieldsId, // fields id
        arena::TypesId,  // types id
    ),

    FnSignature(Signature),
}

#[derive(Clone, Copy, Debug)]
pub enum Declaration {
    Fn(Signature, Block),
    ExplicitVar(
        Span,
        /// expr id
        Option<arena::ExprsId>,
        Type,
    ),
    ShortVar(
        Span,
        /// expr id
        arena::ExprsId,
    ),
    Type(TypeDeclaration),
}

impl Declaration {
    fn type_declaration(
        reader: &mut TokensReader<impl io::Read>,
        arena: &mut arena::Arena,
    ) -> Result<Declaration> {
        todo!()
    }

    fn short_variable_declaration(
        reader: &mut TokensReader<impl io::Read>,
        arena: &mut arena::Arena,
    ) -> Result<Declaration> {
        let ident = reader.seek_expected(TokenKind::Identifier)?.span;
        reader.eat(TokenKind::ShortAssign)?;
        let expr = Expr::parse(reader, arena)?;
        Ok(Declaration::ShortVar(ident, arena.alloc_expr(expr)))
    }
    fn explicit_variable_declaration(
        reader: &mut TokensReader<impl io::Read>,
        arena: &mut arena::Arena,
    ) -> Result<Self> {
        reader.eat(TokenKind::Let)?;
        let ident = reader.seek_expected(TokenKind::Identifier)?.span;
        reader.eat(TokenKind::Colon)?;
        let token = reader.peek(0);
        let type_ = match Type::try_from(token) {
            Ok(t) => t,
            Err(_) => return Err(Error::unexpected_token(token)),
        };
        reader.seek()?;
        if reader.peek(0).kind != TokenKind::Assign {
            return Ok(Declaration::ExplicitVar(ident, None, type_));
        }
        reader.eat(TokenKind::Assign)?;
        Ok(Declaration::ExplicitVar(
            ident,
            Some({
                let expr = Expr::parse(reader, arena)?;
                arena.alloc_expr(expr)
            }),
            type_,
        ))
    }
    pub fn parse(
        reader: &mut TokensReader<impl io::Read>,
        arena: &mut arena::Arena,
    ) -> Result<Self> {
        if reader.peek(0).kind == TokenKind::Let {
            return Self::explicit_variable_declaration(reader, arena);
        }
        let delimiter = reader.peek(1);
        if delimiter.kind == TokenKind::ShortAssign {
            return Self::short_variable_declaration(reader, arena);
        }
        if delimiter.kind != TokenKind::ColonColon {
            return Err(Error::unexpected_token_with_expected_kind(
                delimiter,
                TokenKind::ColonColon,
            ));
        }
        let token = reader.peek(2);
        if token.kind == TokenKind::LParen {
            let signature = Signature::parse(reader, arena)?;
            if reader.peek(0).kind != TokenKind::LBrace {
                return Ok(Declaration::Type(TypeDeclaration::FnSignature(signature)));
            }
            return Ok(Self::Fn(signature, Block::parse(reader, arena)?));
        }
        Self::type_declaration(reader, arena)
    }

    pub fn to_string(&self, src: &str, arena: &arena::Arena) -> String {
        match self {
            Self::Fn(s, b) => format!("{} {}", s.to_string(src, arena), b.to_string(src, arena)),
            Self::ExplicitVar(i, v, t) => {
                format!(
                    "let {}: {} = {}",
                    i.to_string(src),
                    t.to_string(src),
                    match v {
                        Some(e) => arena.get_expr(*e).to_string(src, arena),
                        None => "".to_string(),
                    }
                )
            }
            Self::ShortVar(i, e) => format!(
                "{} := {}",
                i.to_string(src),
                arena.get_expr(*e).to_string(src, arena)
            ),
            _ => todo!(),
        }
    }
}
