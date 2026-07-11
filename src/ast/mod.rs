use std::{collections::VecDeque, fmt};

mod arena;
mod ast;
mod declaration;
mod expr;
mod for_loops;

pub(super) use arena::*;
pub use ast::*;
pub use declaration::*;
pub use expr::*;
pub use for_loops::*;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    LexerError(crate::lexer::Error),
    UnexpectedToken {
        got: crate::lexer::TokenKind,
        expected: Option<crate::lexer::TokenKind>,
        col: usize,
        line: usize,
    },
    RepeatedNonassociativeOP,
}

impl Error {
    pub fn unexpected_token(token: crate::lexer::Token) -> Self {
        return Self::UnexpectedToken {
            got: token.kind,
            expected: None,
            col: token.col,
            line: token.line,
        };
    }

    pub fn unexpected_token_with_expected_kind(
        token: crate::lexer::Token,
        expected: crate::lexer::TokenKind,
    ) -> Self {
        return Self::UnexpectedToken {
            got: token.kind,
            expected: Some(expected),
            col: token.col,
            line: token.line,
        };
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LexerError(e) => write!(f, "{e}"),
            Self::UnexpectedToken {
                got,
                expected: _,
                col,
                line,
            } => write!(f, "{line}:{col}: unexpected token {}", got),
            Self::RepeatedNonassociativeOP => write!(f, "repeated nonassociative op usage"),
        }
    }
}

pub(super) struct TokensReader<R> {
    lexer: crate::lexer::Lexer<R>,
    lookahead: VecDeque<crate::lexer::Token>,
}

impl<R: std::io::Read> TokensReader<R> {
    pub fn new(lexer: crate::lexer::Lexer<R>) -> Result<Self> {
        const LOOKAHEAD_SIZE: usize = 3;
        let lookahead = VecDeque::with_capacity(LOOKAHEAD_SIZE);
        let mut r = Self { lexer, lookahead };
        for _ in 0..LOOKAHEAD_SIZE {
            let token = r.get_next_token()?;
            r.lookahead.push_back(token);
        }
        Ok(r)
    }

    fn get_next_token(&mut self) -> Result<crate::lexer::Token> {
        self.lexer
            .next()
            .expect("lexer should always return Some(token)")
            .map_err(Error::LexerError)
    }

    pub fn peek(&mut self, pos: usize) -> crate::lexer::Token {
        if let Some(token) = self.lookahead.get(pos) {
            return *token;
        }
        unreachable!()
    }
    pub fn seek(&mut self) -> Result<crate::lexer::Token> {
        let token = self
            .lookahead
            .pop_front()
            .expect("there must be always tokens in queue");
        let new_token = self.get_next_token()?;
        self.lookahead.push_back(new_token);
        Ok(token)
    }

    pub fn eat(&mut self, kind: crate::lexer::TokenKind) -> Result<()> {
        let token = self.peek(0);
        if token.kind != kind {
            return Err(Error::unexpected_token_with_expected_kind(token, kind));
        }
        self.seek()?;
        Ok(())
    }

    pub fn seek_expected(
        &mut self,
        expected_kind: crate::lexer::TokenKind,
    ) -> Result<crate::lexer::Token> {
        let token = self.peek(0);
        if token.kind != expected_kind {
            return Err(Error::unexpected_token_with_expected_kind(
                token,
                expected_kind,
            ));
        }
        self.seek()
    }
}

struct Parser<'a, T> {
    pub(super) arena: &'a Arena,
    pub(super) reader: &'a TokensReader<T>,
}




#[cfg(test)]
mod test {

    use crate::{lexer::Lexer, lexer::Span, utf8_reader::Utf8Reader};
    use std::io::{self, Read};
    use std::mem::discriminant;
    use crate::ast::Atom::Ident;
    use super::{Declaration, Expr, Program, Statement, TokensReader, arena, Atom, InfixOp};

    fn setup(input: &str) -> (TokensReader<impl io::Read>, arena::Arena) {
        (
            TokensReader::new(Lexer::new(Utf8Reader::new(input.as_bytes().bytes())).unwrap())
                .unwrap(),
            arena::Arena::new(),
        )
    }

    #[test]
    fn simple_program() {
        let expected_program = r#"foo :: () -> i8 {
    let a: i8 = 12
    b := a + 4
    return b * b
}
"#;
        let (mut reader, mut arena) = setup(expected_program);

        let got_program = Program::parse(&mut reader, &mut arena).unwrap();

        let (start, end) = (0, 0); // for span

        use Atom::*;
        assert_eq!(got_program.0.len(), 1);
        let mut expected_arena = arena::Arena::new();
        // let a: i8 = 12
        expected_arena.alloc_expr(Expr::Atom(IntLit(12)));

        // a
        let lhs = expected_arena.alloc_expr(Expr::Atom(Ident(Span{start, end})));

        // 4
        let rhs = expected_arena.alloc_expr(Expr::Atom(IntLit(4)));

        // a + 4
        expected_arena.alloc_expr(Expr::InfixOp(InfixOp::Add, lhs, rhs));

        // b
        let lhs = expected_arena.alloc_expr(Expr::Atom(Ident(Span{start, end})));
        // b
        let rhs = expected_arena.alloc_expr(Expr::Atom(Ident(Span{start, end})));

        // b * b
        let seq = Expr::Sequence(expected_arena.alloc_exprs(&mut vec![Expr::InfixOp(InfixOp::Mul, lhs, rhs)]));

        let expr = expected_arena.alloc_expr(seq);

        expected_arena.alloc_expr(Expr::Return(expr));

        arena.get_all_exprs()
            .iter()
            .zip(expected_arena.get_all_exprs().into_iter())
            .enumerate()
            .for_each(|(i, (g, e))| {
                let (expected, got) = (discriminant(e), discriminant(g));
                assert!(
                    expected == got,
                    "unexpected expression at index {i}, expected: \n{:?}\ngot: \n{:?}",
                    expected,
                    got
                )
            });
    }
}
