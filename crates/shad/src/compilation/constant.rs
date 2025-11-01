use crate::compilation::index::NodeIndex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct ConstantValue {
    pub(crate) transpiled_type_name: String,
    pub(crate) data: ConstantData,
}

#[derive(Debug, Clone)]
pub(crate) enum ConstantData {
    F32(f32),
    I32(i32),
    U32(u32),
    Bool(bool),
    StructFields(Vec<ConstantStructFieldData>),
}

impl ConstantData {
    pub(crate) fn fields(&self) -> &[ConstantStructFieldData] {
        if let Self::StructFields(fields) = self {
            fields
        } else {
            unreachable!("constant is not a struct")
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ConstantStructFieldData {
    pub(crate) name: String,
    pub(crate) value: ConstantValue,
    pub(crate) is_alias: bool,
}

#[derive(Debug)]
pub(crate) struct ConstantContext<'a> {
    pub(crate) index: &'a NodeIndex,
    scopes: Vec<Scope>,
}

impl<'a> ConstantContext<'a> {
    pub(crate) fn new(index: &'a NodeIndex) -> Self {
        Self {
            index,
            scopes: vec![],
        }
    }

    pub(crate) fn start_fn(&mut self, params: HashMap<u32, ConstantValue>) {
        self.scopes.push(Scope { vars: params });
    }

    pub(crate) fn end_fn(&mut self) {
        self.scopes.pop();
    }

    pub(crate) fn create_var(&mut self, id: u32, value: ConstantValue) {
        let scope_count = self.scopes.len();
        self.scopes[scope_count - 1].vars.insert(id, value);
    }

    pub(crate) fn var_value(&self, id: u32) -> Option<&ConstantValue> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.vars.get(&id))
    }
}

#[derive(Debug)]
struct Scope {
    vars: HashMap<u32, ConstantValue>,
}
