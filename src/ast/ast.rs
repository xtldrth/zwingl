use super::{Declaration, Error, Expr, Result, arena};
use crate::ast::TokensReader;
use crate::lexer::{Token, TokenKind};
use std::fmt::Write;
use std::io;
use std::slice::Iter;

#[derive(Clone, Copy, Debug)]
pub enum Statement {
    Declaration(Declaration),
    Expr(arena::ExprsId),
}

impl Statement {
    pub fn parse(
        reader: &mut TokensReader<impl io::Read>,
        arena: &mut arena::Arena,
    ) -> Result<Self> {
        let next_token = reader.peek(0);
        use TokenKind::*;
        Ok(match next_token.kind {
            If | For | Return => Self::Expr({
                let expr = Expr::parse(reader, arena)?;
                arena.alloc_expr(expr)
            }),
            TokenKind::Let => Statement::Declaration(Declaration::parse(reader, arena)?),
            TokenKind::Identifier => {
                if let t = reader.peek(1)
                    && (t.kind == TokenKind::ColonColon || t.kind == TokenKind::ShortAssign)
                {
                    return Ok(Statement::Declaration(Declaration::parse(reader, arena)?));
                }
                return Ok(Statement::Expr({
                    let expr = Expr::parse(reader, arena)?;
                    arena.alloc_expr(expr)
                }));
            }
            _ => Statement::Expr({
                let expr = Expr::parse(reader, arena)?;
                arena.alloc_expr(expr)
            }),
        })
    }

    pub fn to_string(&self, src: &str, arena: &arena::Arena) -> String {
        match self {
            Self::Declaration(d) => d.to_string(src, arena),
            Self::Expr(e) => arena.get_expr(*e).to_string(src, arena),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Block(pub arena::StatementsId);

impl Block {
    pub fn parse(
        reader: &mut TokensReader<impl io::Read>,
        arena: &mut arena::Arena,
    ) -> Result<Self> {
        reader.eat(TokenKind::LBrace)?;
        let mut statements = Vec::<Statement>::new();
        loop {
            if reader.peek(0).kind == TokenKind::RBrace {
                reader.seek()?;
                break;
            }
            statements.push(Statement::parse(reader, arena)?);
        }
        Ok(Self(arena.alloc_statements(&mut statements)))
    }

    pub fn statements_id(&self) -> arena::StatementsId {
        self.0
    }
    pub fn to_string(&self, src: &str, arena: &arena::Arena) -> String {
        let mut result_string = String::new();
        write!(&mut result_string, "{{\n").unwrap();
        for statement in arena.get_statements(self.statements_id()) {
            write!(&mut result_string, "\t{}\n", statement.to_string(src, arena)).unwrap();
        }
        write!(&mut result_string, "}}\n").unwrap();
        result_string
    }
}

pub struct Program(pub Vec<Declaration>);

impl Program {
    pub fn parse(
        reader: &mut TokensReader<impl io::Read>,
        arena: &mut arena::Arena,
    ) -> Result<Self> {
        let mut declarations = Vec::new();
        while reader.peek(0).kind != TokenKind::EOF {
            declarations.push(Declaration::parse(reader, arena)?);
        }
        Ok(Self(declarations))
    }

    pub fn declarations(&self) -> Iter<Declaration> {
        self.0.iter()
    }

    pub fn to_string(&self, src: &str, arena: &arena::Arena) -> String {
        let mut result_string = String::new();
        for declaration in self.declarations() {
            write!(
                &mut result_string,
                "{}\n",
                declaration.to_string(src, arena)
            ).expect("TODO: panic message");
        }
        result_string
    }
}
