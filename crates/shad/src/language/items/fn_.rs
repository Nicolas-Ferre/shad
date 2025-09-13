use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, NodeConfig, NodeType, Repeated};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::items::block::Block;
use crate::language::items::type_;
use crate::language::items::type_::Type;
use crate::language::keywords::{
    ArrowSymbol, CloseParenthesisSymbol, ColonSymbol, CommaSymbol, EqSymbol, FnKeyword,
    NativeKeyword, OpenParenthesisSymbol, RefKeyword, SemicolonSymbol,
};
use crate::language::patterns::{Ident, StringLiteral};
use crate::language::{sources, validations};
use crate::ValidationError;
use indoc::indoc;
use itertools::Itertools;
use std::iter;
use std::rc::Rc;

sequence!(
    struct NativeFnItem {
        native: NativeKeyword,
        signature: FnSignature,
        #[force_error(true)]
        eq: EqSymbol,
        transpilation: StringLiteral,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for NativeFnItem {
    fn key(&self) -> Option<String> {
        Some(self.signature.fn_key())
    }

    fn is_ref(&self, index: &NodeIndex) -> bool {
        self.signature.is_ref(index)
    }

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.signature.type_(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_duplicated_items(self, ctx);
    }

    fn is_transpilable_dependency(&self, _index: &NodeIndex) -> bool {
        false
    }
}

sequence!(
    struct FnItem {
        signature: FnSignature,
        #[force_error(true)]
        body: Block,
    }
);

impl NodeConfig for FnItem {
    fn key(&self) -> Option<String> {
        Some(self.signature.fn_key())
    }

    fn is_ref(&self, index: &NodeIndex) -> bool {
        self.signature.is_ref(index)
    }

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.signature.type_(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_duplicated_items(self, ctx);
        validations::check_recursive_items(self, ctx);
        let return_stmt = self.body.last_stmt().and_then(|stmt| stmt.as_return());
        let return_type = self.signature.return_type.iter().next();
        if let (None, Some(return_type)) = (return_stmt, return_type) {
            ctx.errors.push(ValidationError::error(
                ctx,
                &*self.body,
                "missing return statement",
                Some("last statement should be a `return` statement"),
                &[(&**return_type, "the function has a return type")],
            ));
        }
        if let (Some(return_stmt), Some(expected_type)) = (return_stmt, self.type_(ctx.index)) {
            if let Some(actual_type) = return_stmt.type_(ctx.index) {
                if !actual_type.is_no_return()
                    && actual_type.source().map(|s| s.id) != expected_type.source().map(|s| s.id)
                {
                    let actual_type_name = type_::name_or_no_return(actual_type);
                    let expected_type_name = type_::name_or_no_return(expected_type);
                    ctx.errors.push(ValidationError::error(
                        ctx,
                        &*return_stmt.expr,
                        "invalid returned type",
                        Some(&format!("returned type is `{actual_type_name}`",)),
                        &[(
                            &*self.signature.return_type,
                            &format!("expected type is `{expected_type_name}`"),
                        )],
                    ));
                }
            }
        }
    }

    fn is_transpilable_dependency(&self, index: &NodeIndex) -> bool {
        !self.is_inlined(index)
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        format!(
            indoc!(
                "{signature} {{
                {param_vars}
                {body}
                }}"
            ),
            signature = self.signature.transpile(ctx),
            param_vars = self
                .signature
                .params()
                .map(|param| format!("var _{id} = _p{id};", id = param.id))
                .join("\n"),
            body = self.body.transpile(ctx),
        )
    }
}

impl FnItem {
    pub(crate) fn is_inlined(&self, index: &NodeIndex) -> bool {
        self.signature.params().any(|param| param.is_ref(index)) || self.is_ref(index)
    }
}

sequence!(
    struct FnSignature {
        fn_: FnKeyword,
        #[force_error(true)]
        ident: Ident,
        params_start: OpenParenthesisSymbol,
        params: Repeated<FnParamGroup, 0, 1>,
        params_end: CloseParenthesisSymbol,
        return_type: Repeated<FnReturnType, 0, 1>,
    }
);

impl NodeConfig for FnSignature {
    fn is_ref(&self, _index: &NodeIndex) -> bool {
        self.return_type
            .iter()
            .next()
            .is_some_and(|return_type| return_type.ref_.iter().len() == 1)
    }

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        if let Some(return_type) = &self.return_type.iter().next() {
            return_type.type_(index)
        } else {
            Some(NodeType::NoReturn)
        }
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        for param1 in self.params() {
            for param2 in self.params() {
                if param1.id < param2.id && param1.ident.slice == param2.ident.slice {
                    ctx.errors.push(ValidationError::error(
                        ctx,
                        param2,
                        "function parameter defined multiple times",
                        Some("duplicated parameter name"),
                        &[(param1, "same parameter name defined here")],
                    ));
                }
            }
        }
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        format!(
            "fn _{fn_id}({params}) {return_type}",
            fn_id = self.parent_ids[self.parent_ids.len() - 1],
            params = self.params().map(|param| param.transpile(ctx)).join(", "),
            return_type = self.return_type.transpile(ctx),
        )
    }
}

impl FnSignature {
    pub(crate) fn params(&self) -> impl Iterator<Item = &FnParam> {
        self.params
            .iter()
            .flat_map(|params| params.params())
            .map(|param| &**param)
    }

    fn fn_key(&self) -> String {
        sources::fn_key_from_params(&self.ident, &self.params)
    }
}

sequence!(
    struct FnParamGroup {
        first_param: FnParam,
        #[force_error(true)]
        other_params: Repeated<FnOtherParam, 0, { usize::MAX }>,
        final_comma: Repeated<CommaSymbol, 0, 1>,
    }
);

impl NodeConfig for FnParamGroup {}

impl FnParamGroup {
    pub(crate) fn params(&self) -> impl Iterator<Item = &Rc<FnParam>> {
        iter::once(&self.first_param).chain(self.other_params.iter().map(|other| &other.param))
    }
}

sequence!(
    struct FnParam {
        ident: Ident,
        #[force_error(true)]
        colon: ColonSymbol,
        ref_: Repeated<RefKeyword, 0, 1>,
        type_: Type,
    }
);

impl NodeConfig for FnParam {
    fn key(&self) -> Option<String> {
        Some(sources::variable_key(&self.ident))
    }

    fn is_ref(&self, _index: &NodeIndex) -> bool {
        self.ref_.iter().len() == 1
    }

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.type_.type_(index)
    }

    fn is_transpilable_dependency(&self, _index: &NodeIndex) -> bool {
        false
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let id = &self.id;
        let type_ = &self.type_.transpile(ctx);
        format!("_p{id}: {type_}")
    }
}

sequence!(
    #[allow(unused_mut)]
    struct FnOtherParam {
        commas: CommaSymbol,
        param: FnParam,
    }
);

impl NodeConfig for FnOtherParam {}

sequence!(
    struct FnReturnType {
        arrow: ArrowSymbol,
        #[force_error(true)]
        ref_: Repeated<RefKeyword, 0, 1>,
        type_: Type,
    }
);

impl NodeConfig for FnReturnType {
    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.type_.type_(index)
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let type_ = self.type_.transpile(ctx);
        format!("-> {type_}")
    }
}
