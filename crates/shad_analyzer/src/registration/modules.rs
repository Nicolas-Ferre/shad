use crate::{errors, Analysis};
use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools;
use shad_parser::AstItem;
use std::iter;

pub(crate) fn register(analysis: &mut Analysis) {
    let imported_modules = imported_modules(analysis);
    register_visible(analysis, &imported_modules);
}

fn imported_modules(analysis: &mut Analysis) -> FxHashMap<String, Vec<Module>> {
    let mut imported_modules = FxHashMap::default();
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
                    modules.push(Module {
                        path: imported_module,
                        is_pub: import.is_pub,
                    });
                } else {
                    analysis
                        .errors
                        .push(errors::modules::not_found(import, &imported_module));
                }
            }
        }
        imported_modules.insert(module.clone(), modules);
    }
    imported_modules
}

fn register_visible(analysis: &mut Analysis, imported_modules: &FxHashMap<String, Vec<Module>>) {
    for module in imported_modules.keys() {
        analysis.visible_modules.insert(
            module.clone(),
            find_visible_modules(module, imported_modules, 0, &mut FxHashSet::default()),
        );
    }
}

fn find_visible_modules(
    module: &str,
    imported_modules: &FxHashMap<String, Vec<Module>>,
    module_level: u32,
    already_found_modules: &mut FxHashSet<String>,
) -> Vec<String> {
    iter::once(module.to_string())
        .chain({
            let child_modules = imported_modules[module]
                .iter()
                .rev()
                .filter(|module| !already_found_modules.contains(&module.path))
                .filter(|module| {
                    if module_level == 0 {
                        true
                    } else {
                        module.is_pub
                    }
                })
                .collect::<Vec<_>>();
            for module in &child_modules {
                already_found_modules.insert(module.path.clone());
            }
            child_modules.into_iter().flat_map(|module| {
                find_visible_modules(
                    &module.path,
                    imported_modules,
                    module_level + 1,
                    already_found_modules,
                )
            })
        })
        .collect::<Vec<_>>()
}

struct Module {
    path: String,
    is_pub: bool,
}
