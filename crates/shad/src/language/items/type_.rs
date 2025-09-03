use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, NodeConfig};
use crate::compilation::transpilation::TranspilationContext;
use crate::language::patterns::Ident;

pub(crate) const NO_RETURN_TYPE: &str = "<no return>";

// TODO: validate type
sequence!(
    #[allow(unused_mut)]
    struct Type {
        ident: Ident,
    }
);

impl NodeConfig for Type {
    fn expr_type(&self, _index: &NodeIndex) -> Option<String> {
        Some(self.ident.slice.clone())
    }

    fn transpile(&self, _ctx: &mut TranspilationContext<'_>) -> String {
        let type_name = self.ident.slice.clone();
        transpile_type(type_name)
    }
}

pub(crate) fn transpile_type(type_name: String) -> String {
    if type_name == "bool" {
        "u32".into()
    } else {
        type_name
    }
}
