use crate::registration::constants::ConstantValue;
use crate::{Analysis, IdentSource};
use shad_parser::{AstExpr, AstExprRoot, AstLiteral, AstLiteralType, AstStatement, VisitMut};
use std::mem;

pub(crate) fn transform(analysis: &mut Analysis) {
    super::transform_statements(analysis, |analysis, statements| {
        *statements = mem::take(statements)
            .into_iter()
            .flat_map(|mut statement| {
                let mut transform = ConstantTransform::new(analysis);
                transform.visit_statement(&mut statement);
                transform.statements.push(statement);
                transform.statements
            })
            .collect();
    });
}

struct ConstantTransform<'a> {
    analysis: &'a mut Analysis,
    statements: Vec<AstStatement>,
}

impl<'a> ConstantTransform<'a> {
    fn new(analysis: &'a mut Analysis) -> Self {
        Self {
            analysis,
            statements: vec![],
        }
    }

    fn literal_str(value: &ConstantValue) -> String {
        match value {
            ConstantValue::U32(value) => format!("{value}u"),
            ConstantValue::I32(value) => value.to_string(),
            ConstantValue::F32(value) => {
                let value = value.to_string();
                if value.contains('.') {
                    value
                } else {
                    format!("{value}.0")
                }
            }
            ConstantValue::Bool(value) => value.to_string(),
        }
    }

    fn literal_type(value: &ConstantValue) -> AstLiteralType {
        match value {
            ConstantValue::U32(_) => AstLiteralType::U32,
            ConstantValue::I32(_) => AstLiteralType::I32,
            ConstantValue::F32(_) => AstLiteralType::F32,
            ConstantValue::Bool(_) => AstLiteralType::Bool,
        }
    }
}

impl VisitMut for ConstantTransform<'_> {
    fn enter_expr(&mut self, node: &mut AstExpr) {
        if let AstExprRoot::Ident(ident) = &node.root {
            if let IdentSource::Constant(constant_id) = &self.analysis.idents[&ident.id].source {
                let value = self.analysis.constants[constant_id]
                    .value
                    .clone()
                    .expect("internal error: not calculated constant");
                node.root = AstExprRoot::Literal(AstLiteral {
                    span: node.root.span().clone(),
                    value: Self::literal_str(&value),
                    type_: Self::literal_type(&value),
                });
            }
        }
    }
}
