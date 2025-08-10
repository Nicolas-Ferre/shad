pub(crate) mod idents;
pub(crate) mod imports;
pub(crate) mod literals;
pub(crate) mod types;

use crate::config::ValidationConfig;
use crate::validation::ValidationContext;
use crate::AstNode;

pub(crate) fn run(ctx: &mut ValidationContext<'_>, validation: &ValidationConfig, node: &AstNode) {
    match validation.name.as_str() {
        "check_number_range" => literals::check_number_range(ctx, validation, node),
        "check_ident_uniqueness" => idents::check_ident_uniqueness(ctx, validation, node),
        "check_existing_source" => idents::check_existing_source(ctx, node),
        "check_expr_type" => types::check_expr_type(ctx, validation, node),
        "check_import_path" => imports::check_import_path(ctx, node),
        validation_name => unreachable!("undefined `{validation_name}` validation"),
    }
}
