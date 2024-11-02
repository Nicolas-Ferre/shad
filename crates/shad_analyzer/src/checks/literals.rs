use crate::{errors, Analysis};
use shad_error::SemanticError;
use shad_parser::{AstLiteral, AstLiteralType, Visit};
use std::str::FromStr;

pub(crate) fn check(analysis: &mut Analysis) {
    let mut checker = LiteralCheck::new(analysis);
    for block in &analysis.init_blocks {
        checker.visit_run_item(&block.ast);
    }
    for block in &analysis.run_blocks {
        let mut checker = LiteralCheck::new(analysis);
        checker.visit_run_item(&block.ast);
    }
    for fn_ in analysis.fns.values() {
        let mut checker = LiteralCheck::new(analysis);
        checker.visit_fn_item(&fn_.ast);
    }
    analysis.errors.extend(checker.errors);
}

struct LiteralCheck<'a> {
    analysis: &'a Analysis,
    errors: Vec<SemanticError>,
}

impl<'a> LiteralCheck<'a> {
    fn new(analysis: &'a Analysis) -> Self {
        Self {
            analysis,
            errors: vec![],
        }
    }

    fn check_f32_literal(&self, literal: &AstLiteral) -> Option<SemanticError> {
        const F32_INT_PART_LIMIT: usize = 38;
        let digit_count = Self::int_part_digit_count(&literal.value);
        (digit_count > F32_INT_PART_LIMIT).then(|| {
            errors::literals::too_many_f32_digits(
                self.analysis,
                literal,
                digit_count,
                F32_INT_PART_LIMIT,
            )
        })
    }

    fn check_int_literal<T>(&self, literal: &AstLiteral, type_name: &str) -> Option<SemanticError>
    where
        T: FromStr,
    {
        let value = if type_name == "u32" {
            &literal.value[..literal.value.len() - 1]
        } else {
            &literal.value
        };
        T::from_str(value)
            .is_err()
            .then(|| errors::literals::invalid_integer(self.analysis, literal, type_name))
    }

    fn int_part_digit_count(float: &str) -> usize {
        float
            .find('.')
            .expect("internal error: `.` not found in `f32` literal")
    }
}

impl Visit for LiteralCheck<'_> {
    fn enter_literal(&mut self, node: &AstLiteral) {
        let error = match node.type_ {
            AstLiteralType::F32 => self.check_f32_literal(node),
            AstLiteralType::U32 => self.check_int_literal::<u32>(node, "u32"),
            AstLiteralType::I32 => self.check_int_literal::<i32>(node, "i32"),
            AstLiteralType::Bool => None,
        };
        self.errors.extend(error);
    }
}
