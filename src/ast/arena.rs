use std::{collections::HashMap, ops::Index};

use super::{Expr, Statement, Type};
use crate::{ast::arena::IdVariants::NoItems, lexer::Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IdVariants {
    NoItems,
    OneItem(usize),
    Allocated(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Id(IdVariants);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldsId(Id);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdentsId(Id);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypesId(Id);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatementsId(Id);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExprsId(Id);

type Ends = HashMap<usize, usize>;

const ZERO_ITEMS_ID: Id = Id {
    0: IdVariants::NoItems,
};

#[derive(Default)]
pub(crate) struct Arena {
    fields: Vec<Span>,
    idents: Vec<Span>,
    types: Vec<Type>,
    statements: Vec<Statement>,
    exprs: Vec<Expr>,

    fields_ends: Ends,
    idents_ends: Ends,
    types_ends: Ends,
    statements_ends: Ends,
    exprs_ends: Ends,
}

impl Arena {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.exprs.clear();
        self.fields.clear();
        self.idents.clear();
        self.statements.clear();
        self.types.clear();

        self.fields_ends.clear();
        self.idents.clear();
        self.types_ends.clear();
        self.statements_ends.clear();
        self.exprs_ends.clear();
    }

    fn alloc_item<T>(arena: &mut Vec<T>, item: T) -> Id {
        let id = arena.len();
        arena.push(item);
        Id(IdVariants::OneItem(id))
    }

    pub(crate) fn alloc_expr(&mut self, item: Expr) -> ExprsId {
        ExprsId(Self::alloc_item(&mut self.exprs, item))
    }

    fn alloc_items<T>(arena: &mut Vec<T>, items: &mut Vec<T>, ends: &mut Ends) -> Id {
        if items.is_empty() {
            return ZERO_ITEMS_ID;
        }
        let id = arena.len();
        ends.insert(id, id + items.len());
        arena.append(items);
        Id(IdVariants::Allocated(id))
    }

    pub(crate) fn alloc_fields(&mut self, fields: &mut Vec<Span>) -> FieldsId {
        FieldsId(Self::alloc_items(
            &mut self.fields,
            fields,
            &mut self.fields_ends,
        ))
    }

    pub(crate) fn alloc_identifiers(&mut self, identifiers: &mut Vec<Span>) -> IdentsId {
        IdentsId(Self::alloc_items(
            &mut self.idents,
            identifiers,
            &mut self.idents_ends,
        ))
    }

    pub(crate) fn alloc_types(&mut self, types: &mut Vec<Type>) -> TypesId {
        TypesId(Self::alloc_items(
            &mut self.types,
            types,
            &mut self.types_ends,
        ))
    }

    pub(crate) fn alloc_statements(&mut self, statements: &mut Vec<Statement>) -> StatementsId {
        StatementsId(Self::alloc_items(
            &mut self.statements,
            statements,
            &mut self.statements_ends,
        ))
    }

    pub(crate) fn alloc_exprs(&mut self, exprs: &mut Vec<Expr>) -> ExprsId {
        ExprsId(Self::alloc_items(
            &mut self.exprs,
            exprs,
            &mut self.exprs_ends,
        ))
    }

    fn get_items<'a, T>(arena: &'a Vec<T>, ends: &Ends, items_id: Id) -> &'a [T] {
        match items_id {
            Id(NoItems) => arena.get(0..0).unwrap(),
            Id(IdVariants::Allocated(id)) => arena.get(id..ends[&id]).expect("invalid arena id"),
            _ => {
                panic!("wrong id usage, expected either `NoItems` or `Allocated` key variants")
            }
        }
    }

    pub(crate) fn get_fields(&self, id: FieldsId) -> &[Span] {
        Self::get_items(&self.fields, &self.fields_ends, id.0)
    }
    pub(crate) fn get_identifiers(&self, id: IdentsId) -> &[Span] {
        Self::get_items(&self.idents, &self.idents_ends, id.0)
    }
    pub(crate) fn get_types(&self, id: TypesId) -> &[Type] {
        Self::get_items(&self.types, &self.types_ends, id.0)
    }
    pub(crate) fn get_statements(&self, id: StatementsId) -> &[Statement] {
        Self::get_items(&self.statements, &self.statements_ends, id.0)
    }
    pub(crate) fn get_exprs(&self, id: ExprsId) -> &[Expr] {
        Self::get_items(&self.exprs, &self.exprs_ends, id.0)
    }

    fn get_item<'a, T>(arena: &'a Vec<T>, item_id: Id) -> &'a T {
        match item_id {
            Id(IdVariants::OneItem(id)) => arena.index(id),
            _ => panic!("wrong id usage, expected `OneItem` variant"),
        }
    }

    pub(crate) fn get_expr(&self, id: ExprsId) -> &Expr {
        Self::get_item(&self.exprs, id.0)
    }

    pub(super) fn get_all_exprs(&self) -> &[Expr] {
        self.exprs.as_slice()
    }
}
