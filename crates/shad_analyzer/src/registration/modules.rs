use crate::{errors, Analysis};
use itertools::Itertools;
use shad_parser::AstItem;
use std::iter;

pub(crate) fn register(analysis: &mut Analysis) {
    for (module, ast) in &analysis.asts {
        let mut modules = vec![];
        for item in &ast.items {
            if let AstItem::Import(import) = item {
                let imported_module = import
                    .segments
                    .iter()
                    .map(|segment| &segment.label)
                    .join(".");
                if analysis.asts.contains_key(&imported_module) {
                    modules.push(imported_module);
                } else {
                    analysis
                        .errors
                        .push(errors::modules::not_found(import, &imported_module));
                }
            }
        }
        analysis.visible_modules.insert(
            module.clone(),
            iter::once(module.clone())
                .chain(modules.into_iter().rev())
                .collect(),
        );
    }
}
