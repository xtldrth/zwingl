use std::io;

use std::fmt::Write;
use super::{Block, Declaration, Expr, Result, TokensReader, arena};
use crate::lexer::TokenKind;

#[derive(Clone, Copy, Debug)]
pub enum ForLoop {
    /// for identifier in expression
    ForIn(
        arena::ExprsId, // identifier or _
        arena::ExprsId,
        Block, // statements id
    ),
    ForC(
        Option<Declaration>,
        Option<arena::ExprsId>,
        Option<arena::ExprsId>,
        Block, // statements id
    ),
    ForWhile(Option<arena::ExprsId>, Block),
}

impl ForLoop {
    pub fn parse(
        reader: &mut TokensReader<impl io::Read>,
        arena: &mut arena::Arena,
    ) -> Result<Self> {
        reader.eat(TokenKind::For)?;
        let next_token = reader.peek(0);

        // parsing c style for
        if next_token.kind == TokenKind::Let || next_token.kind == TokenKind::Semicolon || reader.peek(1).kind == TokenKind::ShortAssign {
            let declaration = match reader.peek(0).kind {
                TokenKind::Semicolon => {
                    reader.eat(TokenKind::Semicolon)?;
                    None
                },
                _ => {
                    let e = Some(Declaration::parse(reader, arena)?);
                    reader.eat(TokenKind::Semicolon)?;
                    e
                },
            };
            let condition = match reader.peek(0).kind {
                TokenKind::Semicolon => {
                    reader.eat(TokenKind::Semicolon)?;
                    None
                },
                _ => Some({
                    let expr = Expr::parse(reader, arena)?;
                    let e = arena.alloc_expr(expr);
                    reader.eat(TokenKind::Semicolon)?;
                    e
                }),
            };
            let update = match reader.peek(0).kind {
                TokenKind::LBrace => None,
                _ => Some({
                    let expr = Expr::parse(reader, arena)?;
                    arena.alloc_expr(expr)
                }),
            };
            return Ok(Self::ForC(
                declaration,
                condition,
                update,
                Block::parse(reader, arena)?,
            ));
        }
        if next_token.kind == TokenKind::LBrace {
            return Ok(Self::ForWhile(None, Block::parse(reader, arena)?));
        }
        let expr = Expr::parse(reader, arena)?;
        if reader.peek(0).kind == TokenKind::In {
            reader.seek()?;
            let rhs = Expr::parse(reader, arena)?;
            return Ok(Self::ForIn(
                arena.alloc_expr(expr),
                arena.alloc_expr(rhs),
                Block::parse(reader, arena)?,
            ));
        }
        Ok(Self::ForWhile(
            Some(arena.alloc_expr(expr)),
            Block::parse(reader, arena)?,
        ))
    }

    pub fn to_string(&self, src: &str, arena: &arena::Arena) -> String {
        let mut result_string = String::new();
        match self {
            Self::ForC(d, c, u, b) => {
                write!(
                    &mut result_string,
                    "for {}; {}; {} {}",
                    match d {
                        Some(d) => d.to_string(src, arena),
                        None => "".into(),
                    },
                    match c {
                        Some(e) => arena.get_expr(*e).to_string(src, arena),
                        None => "".into(),
                    },
                    match u {
                        Some(e) => arena.get_expr(*e).to_string(src, arena),
                        None => "".into(),
                    },
                    b.to_string(src, arena),
                ).unwrap()
            },
            Self::ForIn(lhs, rhs, block) => {
                write!(
                    &mut result_string,
                    "for {} in {} {}",
                    arena.get_expr(*lhs).to_string(src, arena),
                    arena.get_expr(*rhs).to_string(src, arena),
                    block.to_string(src, arena),
                ).unwrap()
            }
            Self::ForWhile(c, b)  => {
                write!(
                    &mut result_string,
                    "for {} {}",
                    match c {
                        Some(e) => arena.get_expr(*e).to_string(src, arena),
                        None=> "".into(),
                    },
                    b.to_string(src, arena)
                ).unwrap()
            }
        }
        result_string
    }
}


#[cfg(test)]
mod test {
    use crate::ast::{TokensReader, arena, ForLoop};
    use crate::lexer::{Lexer};
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

    // TODO: this is not the best way to test this, should find another way
    #[test]
    fn for_c_loops() {
        let (start, end) = (0, 0);
        let sources_and_results = vec![
           (r#"
            for i := 0; i < 12; i++ {
                12 + 24
            }
            "#,
             "for i := 0; (< i 12); (i ++) {\n\t(+ 12 24)\n}\n",
           ),
           (r#"
            for ; i < 12; i++ {
                12 + 24
            }
            "#,
             "for ; (< i 12); (i ++) {\n\t(+ 12 24)\n}\n",
           ),
           (r#"
            for ;; i++ {
                12 + 24
            }
            "#,
            "for ; ; (i ++) {\n\t(+ 12 24)\n}\n",
           ),
            (r#"
            for ;; {
                12 + 24
            }
            "#,
             "for ; ;  {\n\t(+ 12 24)\n}\n",
           ),
            (r#"
            for let i: u8 = 12;; i++ {
                12 + 24
            }
            "#,
             "for let i: u8 = 12; ; (i ++) {\n\t(+ 12 24)\n}\n",
           ),
           (r#"
            for let i: u8 = 12;; {
                12 + 24
            }
            "#,
            "for let i: u8 = 12; ;  {\n\t(+ 12 24)\n}\n",
           ),
        ];
        let mut arena = setup_arena();
        sources_and_results.iter().for_each(
            |(source, expected)| {
                let mut reader = setup_reader(source);

                let e = ForLoop::parse(&mut reader, &mut arena).unwrap();
                assert_eq!(e.to_string(source, &arena), *expected);
            }
        )
    }
}
