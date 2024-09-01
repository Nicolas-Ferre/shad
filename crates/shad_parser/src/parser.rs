use crate::model::{
    Array, AssignmentStatement, BinaryOp, Expr, FnCall, FnItem, FnParam, FnQualifier, ForStatement,
    Ident, LetStatement, Literal, LiteralType, LoopStatement, Program, ReturnStatement, Span,
    Statement, Type, UnaryOp, Value,
};
use crate::UnaryOperator;
use pest_consume::{match_nodes, Error, Parser};
use std::iter;

pub type Result<T> = std::result::Result<T, Error<Rule>>;
pub(crate) type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[derive(Parser)]
#[grammar = "res/shad.pest"]
pub(crate) struct ShadParser;

#[allow(clippy::unnecessary_wraps, clippy::used_underscore_binding)]
#[pest_consume::parser]
impl ShadParser {
    pub(crate) fn program(input: Node<'_>) -> Result<Program> {
        Ok(match_nodes!(input.into_children();
            [fn_item(items).., EOI(())] => Program { items:items.collect() },
        ))
    }

    fn fn_item(input: Node<'_>) -> Result<FnItem> {
        let span = Span::from_node(&input);
        Ok(match_nodes!(input.into_children();
            [
                fn_keyword(()), ident(name),
                parenthesis_open(()), fn_params(params), parenthesis_close(()),
                arrow(()), type_(return_type),
                brace_open(()), statement(statements).., brace_close(())
            ] => FnItem {
                name,
                params,
                return_type: Some(return_type),
                statements: statements.collect(),
                qualifier: FnQualifier::None,
                span,
            },
            [
                fn_keyword(()), ident(name),
                parenthesis_open(()), parenthesis_close(()),
                arrow(()), type_(return_type),
                brace_open(()), statement(statements).., brace_close(())
            ] => FnItem {
                name,
                params: vec![],
                return_type: Some(return_type),
                statements: statements.collect(),
                qualifier: FnQualifier::None,
                span,
            },
            [
                fn_keyword(()), ident(name),
                parenthesis_open(()), fn_params(params), parenthesis_close(()),
                brace_open(()), statement(statements).., brace_close(())
            ] => FnItem {
                name,
                params,
                return_type: None,
                statements: statements.collect(),
                qualifier: FnQualifier::None,
                span,
            },
            [
                fn_keyword(()), ident(name),
                parenthesis_open(()), parenthesis_close(()),
                brace_open(()), statement(statements).., brace_close(())
            ] => FnItem {
                name,
                params: vec![],
                return_type: None,
                statements: statements.collect(),
                qualifier: FnQualifier::None,
                span,
            },
            [
                fn_qualifier(qualifier), fn_keyword(()), ident(name),
                parenthesis_open(()), fn_params(params), parenthesis_close(()),
                arrow(()), type_(return_type), semicolon(())
            ] => FnItem {
                name,
                params,
                return_type: Some(return_type),
                statements: vec![],
                qualifier,
                span,
            },
            [
                fn_qualifier(qualifier), fn_keyword(()), ident(name),
                parenthesis_open(()), parenthesis_close(()),
                arrow(()), type_(return_type), semicolon(())
            ] => FnItem {
                name,
                params: vec![],
                return_type: Some(return_type),
                statements: vec![],
                qualifier,
                span,
            },
            [
                fn_qualifier(qualifier), fn_keyword(()), ident(name),
                parenthesis_open(()), fn_params(params), parenthesis_close(()), semicolon(())
            ] => FnItem {
                name,
                params,
                return_type: None,
                statements: vec![],
                qualifier,
                span,
            },
            [
               fn_qualifier(qualifier), fn_keyword(()), ident(name),
                parenthesis_open(()), parenthesis_close(()), semicolon(())
            ] => FnItem {
                name,
                params: vec![],
                return_type: None,
                statements: vec![],
                qualifier,
                span,
            },
        ))
    }

    fn fn_qualifier(input: Node<'_>) -> Result<FnQualifier> {
        Ok(match_nodes!(input.into_children();
            [cpu_keyword(())] => FnQualifier::Cpu,
            [gpu_keyword(())] => FnQualifier::Gpu,
        ))
    }

    fn fn_params(input: Node<'_>) -> Result<Vec<FnParam>> {
        Ok(match_nodes!(input.into_children();
            [fn_param(param), next_fn_param(params).., comma(())] =>
                iter::once(param).chain(params).collect(),
            [fn_param(param), next_fn_param(params)..] =>
                iter::once(param).chain(params).collect(),
        ))
    }

    fn fn_param(input: Node<'_>) -> Result<FnParam> {
        let span = Span::from_node(&input);
        Ok(match_nodes!(input.into_children();
            [ident(name), colon(()), type_(type_)] => FnParam { name, type_, span },
        ))
    }

    fn next_fn_param(input: Node<'_>) -> Result<FnParam> {
        Ok(match_nodes!(input.into_children();
            [comma(()), fn_param(param)] => param,
        ))
    }

    fn statement(input: Node<'_>) -> Result<Statement> {
        let span = Span::from_node(&input);
        Ok(match_nodes!(input.into_children();
            [let_keyword(()), ident(name), equal(()), expr(expr), semicolon(())] =>
                Statement::Let(LetStatement { name, expr, span }),
            [return_keyword(()), expr(expr), semicolon(())] =>
                Statement::Return(ReturnStatement { expr, span }),
            [
                for_keyword(()), ident(variable), in_keyword(()), expr(iterable), brace_open(()),
                statement(statements).., brace_close(())
            ] => Statement::For(ForStatement {
                variable,
                iterable,
                statements: statements.collect(),
                span,
            }),
            [loop_keyword(()), brace_open(()), statement(statements).., brace_close(())]
                => Statement::Loop(LoopStatement { statements: statements.collect(), span }),
            [value(value), equal(()), expr(expr), semicolon(())] =>
                Statement::Assignment(AssignmentStatement { value, expr, span }),
            [expr(expr), semicolon(())] => Statement::Expr(expr),
        ))
    }

    fn expr(input: Node<'_>) -> Result<Expr> {
        Ok(match_nodes!(input.into_children();
            [expr_part(expr)] => expr,
            [expr_part(first_expr), right_operand(operands)..] => Expr::BinaryOp(
                BinaryOp::from_operands(first_expr, operands.collect())
            ),
        ))
    }

    fn expr_part(input: Node<'_>) -> Result<Expr> {
        Ok(match_nodes!(input.into_children();
            [parenthesis_open(()), expr(expr), parenthesis_close(())] => expr,
            [expr_terminal(expr)] => expr,
        ))
    }

    fn expr_terminal(input: Node<'_>) -> Result<Expr> {
        let span = Span::from_node(&input);
        Ok(match_nodes!(input.into_children();
            [unary_operator(operator), expr_part(expr)] => Expr::UnaryOp(UnaryOp {
                operator: UnaryOperator::parse(&operator),
                expr: Box::new(expr),
                span,
            }),
            [fn_call(call)] => Expr::FnCall(call),
            [
                square_bracket_open(()), expr(item), semicolon(()),
                expr(size), square_bracket_close(())
            ]
                => Expr::Array(Array { item: Box::new(item), size: Box::new(size), span }),
            [value(value)] => Expr::Value(value),
            [i32_literal(value)] => Expr::Literal(Literal {
                value,
                type_: LiteralType::Int,
                span,
            }),
            [f32_literal(value)] => Expr::Literal(Literal {
                value,
                type_: LiteralType::Float,
                span,
            }),
        ))
    }

    fn right_operand(input: Node<'_>) -> Result<(String, Expr)> {
        Ok(match_nodes!(input.into_children();
            [binary_operator(operator), expr_part(expr)] => (operator, expr),
        ))
    }

    fn type_(input: Node<'_>) -> Result<Type> {
        let span = Span::from_node(&input);
        Ok(match_nodes!(input.into_children();
            [
                ident(name), angle_bracket_open(()), type_(generic), angle_bracket_close(()),
            ] => Type {
                name,
                generic: Some(Box::new(generic)),
                span,
            },
            [ident(name)] => Type {
                name,
                generic: None,
                span,
            },
        ))
    }

    fn value(input: Node<'_>) -> Result<Value> {
        let span = Span::from_node(&input);
        Ok(match_nodes!(input.into_children();
            [
                ident(ident), value_attribute(attributes)..,
                square_bracket_open(()), expr(index), square_bracket_close(())
            ] => Value {
                path: iter::once(ident).chain(attributes).collect(),
                index: Some(Box::new(index)),
                span,
            },
            [ident(ident), value_attribute(attributes)..] => Value {
                path: iter::once(ident).chain(attributes).collect(),
                index: None,
                span,
            },
            [
                ident(ident),
                square_bracket_open(()), expr(index), square_bracket_close(())
            ] => Value {
                path: vec![ident],
                index: Some(Box::new(index)),
                span,
            },
            [ident(ident)] => Value {
                path: vec![ident],
                index: None,
                span,
            },
        ))
    }

    fn value_attribute(input: Node<'_>) -> Result<Ident> {
        Ok(match_nodes!(input.into_children();
            [dot(()), ident(ident)] => ident,
        ))
    }

    fn fn_call(input: Node<'_>) -> Result<FnCall> {
        let span = Span::from_node(&input);
        Ok(match_nodes!(input.into_children();
            [ident(name), parenthesis_open(()), fn_call_args(args), parenthesis_close(())] =>
                FnCall { name, args, span },
            [ident(name), parenthesis_open(()), parenthesis_close(())] => FnCall {
                name,
                args: vec![],
                span,
            },
        ))
    }

    fn fn_call_args(input: Node<'_>) -> Result<Vec<Expr>> {
        Ok(match_nodes!(input.into_children();
            [expr(arg), fn_call_next_arg(args).., comma(())] =>
                iter::once(arg).chain(args).collect(),
            [expr(arg), fn_call_next_arg(args)..] =>
                iter::once(arg).chain(args).collect(),
        ))
    }

    fn fn_call_next_arg(input: Node<'_>) -> Result<Expr> {
        Ok(match_nodes!(input.into_children();
            [comma(()), expr(expr)] => expr,
        ))
    }

    fn ident(input: Node<'_>) -> Result<Ident> {
        Ok(Ident {
            label: input.as_str().to_string(),
            span: Span::from_node(&input),
        })
    }

    fn binary_operator(input: Node<'_>) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn unary_operator(input: Node<'_>) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn f32_literal(input: Node<'_>) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn i32_literal(input: Node<'_>) -> Result<String> {
        let mut chars: Vec<_> = input.as_str().chars().collect();
        while chars.first() == Some(&'0') && chars.get(1).is_some() {
            chars.remove(0);
        }
        Ok(chars.iter().collect())
    }

    fn colon(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn semicolon(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn equal(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn comma(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn dot(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn arrow(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn parenthesis_open(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn parenthesis_close(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn brace_open(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn brace_close(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn square_bracket_open(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn square_bracket_close(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn angle_bracket_open(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn angle_bracket_close(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn fn_keyword(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn cpu_keyword(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn gpu_keyword(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn let_keyword(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn return_keyword(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn for_keyword(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn loop_keyword(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    fn in_keyword(_input: Node<'_>) -> Result<()> {
        Ok(())
    }

    #[allow(non_snake_case)]
    fn EOI(_input: Node<'_>) -> Result<()> {
        Ok(())
    }
}
