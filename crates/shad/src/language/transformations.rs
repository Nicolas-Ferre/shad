use crate::compilation::node::{Node, NodeProps, Repeated};
use crate::language::expressions::binary::{
    BinaryExpr, BinaryOperator, MaybeBinaryExpr, ParsedMaybeBinaryExpr,
};
use crate::language::expressions::chain::{
    ChainExpr, ChainSuffix, ParsedChainExpr, TransformedChainExpr,
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

pub(crate) fn transform_binary_expr(expr: ParsedMaybeBinaryExpr) -> MaybeBinaryExpr {
    let operators: Vec<_> = expr.right.iter().map(|right| &right.operator).collect();
    if operators.is_empty() {
        return MaybeBinaryExpr::Parsed(Rc::new(expr));
    }
    let operands: Vec<_> = iter::once(&expr.left)
        .chain(expr.right.iter().map(|right| &right.operand))
        .collect();
    transform_binary_expr_inner(&operators, &operands)
}

pub(crate) fn transform_chain_expr(expr: ParsedChainExpr) -> ChainExpr {
    let suffix: Vec<_> = Rc::into_inner(expr.suffix)
        .expect("internal error: cannot extract expression suffix")
        .take();
    let prefix = Rc::new(ChainExpr::Parsed(Rc::new(ParsedChainExpr {
        expr: expr.expr,
        suffix: Rc::new(Repeated::new(expr.props.clone())),
        props: expr.props,
    })));
    ChainExpr::Transformed(Rc::new(transform_chain_expr_inner(prefix, suffix)))
}

fn transform_binary_expr_inner(
    operators: &[&Rc<BinaryOperator>],
    operands: &[&Rc<ChainExpr>],
) -> MaybeBinaryExpr {
    let split_index = split_index(operators);
    let operator = operators[split_index];
    let left_operators = &operators[..split_index];
    let right_operators = &operators[split_index + 1..];
    let left_operands = &operands[..=split_index];
    let right_operands = &operands[split_index + 1..];
    let left = if left_operators.is_empty() {
        MaybeBinaryExpr::Parsed(Rc::new(ParsedMaybeBinaryExpr {
            right: Rc::new(Repeated::new(operands[0].props().clone())),
            props: operands[0].props().clone(),
            left: operands[0].clone(),
        }))
    } else {
        transform_binary_expr_inner(left_operators, left_operands)
    };
    let right = if right_operators.is_empty() {
        MaybeBinaryExpr::Parsed(Rc::new(ParsedMaybeBinaryExpr {
            right: Rc::new(Repeated::new(right_operands[0].props().clone())),
            props: right_operands[0].props().clone(),
            left: right_operands[0].clone(),
        }))
    } else {
        transform_binary_expr_inner(right_operators, right_operands)
    };
    MaybeBinaryExpr::Transformed(Rc::new(BinaryExpr {
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

fn transform_chain_expr_inner(
    prefix: Rc<ChainExpr>,
    mut suffixes: Vec<Rc<ChainSuffix>>,
) -> TransformedChainExpr {
    if let Some(suffix) = suffixes.pop().and_then(Rc::into_inner) {
        TransformedChainExpr {
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
            suffix: Rc::new(Repeated::from_node(suffix)),
            expr: {
                let new_expr = transform_chain_expr_inner(prefix, suffixes);
                Rc::new(MaybeBinaryExpr::Parsed(Rc::new(ParsedMaybeBinaryExpr {
                    right: Rc::new(Repeated::new(new_expr.props().clone())),
                    props: new_expr.props().clone(),
                    left: Rc::new(ChainExpr::Transformed(Rc::new(new_expr))),
                })))
            },
        }
    } else {
        TransformedChainExpr {
            suffix: Rc::new(Repeated::new(prefix.props().clone())),
            props: prefix.props().clone(),
            expr: Rc::new(MaybeBinaryExpr::Parsed(Rc::new(ParsedMaybeBinaryExpr {
                right: Rc::new(Repeated::new(prefix.props().clone())),
                props: prefix.props().clone(),
                left: prefix,
            }))),
        }
    }
}
