use crate::compilation::node::{Node, NodeProps, Repeated};
use crate::language::nodes::expressions::{
    AssociatedFnCallSuffix, BinaryOperand, BinaryOperator, Expr, ParsedBinaryOperand, ParsedExpr,
    TransformedBinaryOperand, TransformedExpr,
};
use itertools::Itertools;
use std::iter;
use std::rc::Rc;

const OPERATOR_PRIORITY: &[&[&str]] = &[
    &["||"],
    &["&&"],
    &["<", ">", "<=", ">=", "==", "!="],
    &["+", "-"],
    &["*", "/", "%"],
];

pub(crate) fn transform_expr(expr: ParsedExpr) -> Expr {
    let operators: Vec<_> = expr.right.iter().map(|right| &right.operator).collect();
    if operators.is_empty() {
        return Expr::Parsed(Rc::new(expr));
    }
    let operands: Vec<_> = iter::once(&expr.left)
        .chain(expr.right.iter().map(|right| &right.operand))
        .collect();
    transform_binary_expr(&operators, &operands)
}

fn transform_binary_expr(
    operators: &[&Rc<BinaryOperator>],
    operands: &[&Rc<BinaryOperand>],
) -> Expr {
    let split_index = split_index(operators);
    let operator = operators[split_index];
    let left_operators = &operators[..split_index];
    let right_operators = &operators[split_index + 1..];
    let left_operands = &operands[..=split_index];
    let right_operands = &operands[split_index + 1..];
    let left = if left_operators.is_empty() {
        Expr::Parsed(Rc::new(ParsedExpr {
            right: Rc::new(Repeated::new(operands[0].props().clone())),
            props: operands[0].props().clone(),
            left: operands[0].clone(),
        }))
    } else {
        transform_binary_expr(left_operators, left_operands)
    };
    let right = if right_operators.is_empty() {
        Expr::Parsed(Rc::new(ParsedExpr {
            right: Rc::new(Repeated::new(right_operands[0].props().clone())),
            props: right_operands[0].props().clone(),
            left: right_operands[0].clone(),
        }))
    } else {
        transform_binary_expr(right_operators, right_operands)
    };
    Expr::Transformed(Rc::new(TransformedExpr {
        props: NodeProps {
            id: operator.id,
            parent_ids: operator.parent_ids.clone(),
            slice: format!("{} {} {}", left.slice, operator.slice, right.slice),
            span: left.span.start..right.span.end,
            path: operator.path.clone(),
        },
        left: Rc::new(left),
        operator: operator.clone(),
        right: Rc::new(right),
    }))
}

fn split_index(operators: &[&Rc<BinaryOperator>]) -> usize {
    for checked_operators in OPERATOR_PRIORITY {
        for (index, operator) in operators.iter().enumerate().rev() {
            if checked_operators.contains(&operator.slice.as_str()) {
                return index;
            }
        }
    }
    unreachable!("no operator found in binary node")
}

pub(crate) fn transform_binary_operand(expr: ParsedBinaryOperand) -> BinaryOperand {
    let suffix: Vec<_> = Rc::into_inner(expr.call_suffix)
        .expect("internal error: cannot extract expression suffix")
        .take();
    let prefix = Rc::new(BinaryOperand::Parsed(Rc::new(ParsedBinaryOperand {
        expr: expr.expr,
        call_suffix: Rc::new(Repeated::new(expr.props.clone())),
        props: expr.props,
    })));
    BinaryOperand::Transformed(Rc::new(transform_associated_call(prefix, suffix)))
}

fn transform_associated_call(
    prefix: Rc<BinaryOperand>,
    mut suffixes: Vec<Rc<AssociatedFnCallSuffix>>,
) -> TransformedBinaryOperand {
    if let Some(suffix) = suffixes.pop().and_then(Rc::into_inner) {
        TransformedBinaryOperand {
            props: NodeProps {
                id: prefix.id,
                parent_ids: prefix.parent_ids.clone(),
                slice: format!(
                    "{} {} {}",
                    prefix.slice,
                    suffixes.iter().map(|suffix| &suffix.slice).join(" "),
                    suffix.slice
                ),
                span: prefix.span.start..suffix.span.end,
                path: prefix.path.clone(),
            },
            call_suffix: Rc::new(Repeated::from_node(suffix)),
            expr: {
                let new_expr = transform_associated_call(prefix, suffixes);
                Rc::new(Expr::Parsed(Rc::new(ParsedExpr {
                    right: Rc::new(Repeated::new(new_expr.props().clone())),
                    props: new_expr.props().clone(),
                    left: Rc::new(BinaryOperand::Transformed(Rc::new(new_expr))),
                })))
            },
        }
    } else {
        TransformedBinaryOperand {
            call_suffix: Rc::new(Repeated::new(prefix.props().clone())),
            props: prefix.props().clone(),
            expr: Rc::new(Expr::Parsed(Rc::new(ParsedExpr {
                right: Rc::new(Repeated::new(prefix.props().clone())),
                props: prefix.props().clone(),
                left: prefix,
            }))),
        }
    }
}
