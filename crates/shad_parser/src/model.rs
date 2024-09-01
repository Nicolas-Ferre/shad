use crate::parser::{Node, ShadParser};
use crate::{parser, Rule};
use pest_consume::Parser;
use std::vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub items: Vec<FnItem>,
}

impl Program {
    #[allow(clippy::result_large_err)]
    pub fn parse(input: &str) -> parser::Result<Self> {
        // Comments are manually removed to speed up parsing.
        let input = input
            .split('\n')
            .map(|line| line.split_once("//").map_or(line, |line| line.0))
            .collect::<String>();
        let mut inputs = ShadParser::parse(Rule::program, &input)?;
        let input = inputs.next().expect("internal error");
        ShadParser::program(input)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnItem {
    pub name: Ident,
    pub params: Vec<FnParam>,
    pub return_type: Option<Type>,
    pub statements: Vec<Statement>,
    pub qualifier: FnQualifier,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FnQualifier {
    None,
    Cpu,
    Gpu,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnParam {
    pub name: Ident,
    pub type_: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Let(LetStatement),
    Assignment(AssignmentStatement),
    Return(ReturnStatement),
    Expr(Expr),
    For(ForStatement),
    Loop(LoopStatement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetStatement {
    pub name: Ident,
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentStatement {
    pub value: Value,
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStatement {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForStatement {
    pub variable: Ident,
    pub iterable: Expr,
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopStatement {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    UnaryOp(UnaryOp),
    BinaryOp(BinaryOp),
    FnCall(FnCall),
    Array(Array),
    Value(Value),
    Literal(Literal),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Self::UnaryOp(expr) => expr.span,
            Self::BinaryOp(expr) => expr.span,
            Self::FnCall(expr) => expr.span,
            Self::Array(expr) => expr.span,
            Self::Value(expr) => expr.span,
            Self::Literal(expr) => expr.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnaryOp {
    pub operator: UnaryOperator,
    pub expr: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Add,
    Sub,
    Mul,
    Div,
}

impl UnaryOperator {
    pub fn parse(input: &str) -> Self {
        match input {
            "-" => Self::Sub,
            operator => unreachable!("internal error: unrecognized unary operator `{operator}`"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryOp {
    pub left: Box<Expr>,
    pub operator: BinaryOperator,
    pub right: Box<Expr>,
    pub span: Span,
}

impl BinaryOp {
    pub fn from_operands(first: Expr, others: Vec<(String, Expr)>) -> Self {
        let operator_priority = [vec!["+", "-"], vec!["*", "/", "%"]];
        let split_index = operator_priority
            .iter()
            .find_map(|ops| others.iter().position(|(op, _)| ops.contains(&op.as_str())))
            .expect("internal error: unsupported operator");
        let left = if split_index == 0 {
            Box::new(first)
        } else {
            Box::new(Expr::BinaryOp(Self::from_operands(
                first,
                others[..split_index].to_vec(),
            )))
        };
        let right = if split_index == others.len() - 1 {
            Box::new(others[split_index].1.clone())
        } else {
            Box::new(Expr::BinaryOp(Self::from_operands(
                others[split_index].1.clone(),
                others[split_index + 1..].to_vec(),
            )))
        };
        Self {
            span: Span {
                start: left.span().start,
                end: right.span().end,
            },
            left,
            operator: BinaryOperator::parse(&others[split_index].0),
            right,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
}

impl BinaryOperator {
    pub fn parse(input: &str) -> Self {
        match input {
            "+" => Self::Add,
            "-" => Self::Sub,
            "*" => Self::Mul,
            "/" => Self::Div,
            operator => unreachable!("internal error: unrecognized binary operator `{operator}`"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnCall {
    pub name: Ident,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    pub name: Ident,
    pub generic: Option<Box<Type>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Array {
    pub item: Box<Expr>,
    pub size: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    pub path: Vec<Ident>,
    pub index: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub label: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    pub value: String,
    pub type_: LiteralType,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralType {
    Float,
    Int,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Location,
    pub end: Location,
}

impl Span {
    pub fn from_node(node: &Node<'_>) -> Self {
        let (start_line, start_column) = node.as_span().start_pos().line_col();
        let start_offset = node.as_span().start_pos().pos();
        let (end_line, end_column) = node.as_span().end_pos().line_col();
        let end_offset = node.as_span().end_pos().pos();
        Self {
            start: Location {
                column: start_column,
                line: start_line,
                offset: start_offset,
            },
            end: Location {
                column: end_column,
                line: end_line,
                offset: end_offset,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub column: usize,
    pub line: usize,
    pub offset: usize,
}
