use crate::compilation::constant::{ConstantContext, ConstantValue};
use crate::compilation::index::NodeIndex;
use crate::compilation::node::{
    sequence, GenericArgs, Node, NodeConfig, NodeRef, NodeSource, Repeated,
};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::items::block::Block;
use crate::language::keywords::{
    ArrowSymbol, CloseParenthesisSymbol, ColonSymbol, CommaSymbol, ConstKeyword, EqSymbol,
    FnKeyword, NativeKeyword, OpenParenthesisSymbol, PubKeyword, RefKeyword, SemicolonSymbol,
};
use crate::language::patterns::{Ident, StringLiteral};
use crate::language::type_ref::Type;
use crate::language::{constants, sources, validations};
use crate::ValidationError;
use indoc::indoc;
use itertools::Itertools;
use std::any::Any;
use std::iter;
use std::rc::Rc;

sequence!(
    struct NativeFnItem {
        pub_: Repeated<PubKeyword, 0, 1>,
        const_: Repeated<ConstKeyword, 0, 1>,
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

    fn is_public(&self) -> bool {
        self.pub_.iter().len() > 0
    }

    fn is_ref(&self, index: &NodeIndex) -> Option<bool> {
        self.signature.is_ref(index)
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        self.signature.type_(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        let params = self.signature.params().map(|p| p.ident.slice.as_str());
        validations::check_duplicated_items(self, ctx);
        validations::check_native_code(&self.transpilation, params, ctx);
        if self.const_.iter().len() > 0
            && constants::native_fn_runner(&self.signature.fn_key()).is_none()
        {
            ctx.errors.push(ValidationError::error(
                ctx,
                self,
                "`const` native function without constant implementation",
                Some("this function cannot be qualified with `const`"),
                &[],
            ));
        }
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        let params = self
            .signature
            .params()
            .map(|param| {
                ctx.var_value(param.id)
                    .expect("internal error: not found const fn arg variable")
            })
            .collect::<Vec<_>>();
        let return_type = self.type_(ctx.index)?;
        if return_type.is_no_return() {
            unreachable!("constant expressions always return a value")
        } else {
            Some(ConstantValue {
                transpiled_type_name: return_type.transpiled_type_name(ctx.index),
                data: constants::native_fn_runner(&self.key()?)?(&params),
            })
        }
    }

    fn is_transpilable_dependency(&self, _index: &NodeIndex) -> bool {
        false
    }
}

sequence!(
    struct FnItem {
        pub_: Repeated<PubKeyword, 0, 1>,
        const_: Repeated<ConstKeyword, 0, 1>,
        signature: FnSignature,
        #[force_error(true)]
        body: Block,
    }
);

impl NodeConfig for FnItem {
    fn key(&self) -> Option<String> {
        Some(self.signature.fn_key())
    }

    fn is_public(&self) -> bool {
        self.pub_.iter().len() > 0
    }

    fn is_ref(&self, index: &NodeIndex) -> Option<bool> {
        self.signature.is_ref(index)
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        self.signature.type_(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_duplicated_items(self, ctx);
        validations::check_recursive_items(self, ctx);
        if let Some(const_kw) = self.const_.iter().next() {
            validations::check_invalid_const_scope(&*self.body, &**const_kw, ctx);
        }
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
                    && actual_type.are_same_types(&expected_type) == Some(false)
                {
                    let actual_type_name = actual_type.name_or_no_return();
                    let expected_type_name = expected_type.name_or_no_return();
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

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        self.body.evaluate_constant(ctx)
    }

    fn is_transpilable_dependency(&self, index: &NodeIndex) -> bool {
        !self.is_inlined(index)
    }

    fn transpile(
        &self,
        ctx: &mut TranspilationContext<'_>,
        generic_args: &GenericArgs<'_>,
    ) -> String {
        format!(
            indoc!(
                "{signature} {{
                {param_vars}
                {body}
                }}"
            ),
            signature = self.signature.transpile(ctx, generic_args),
            param_vars = self
                .signature
                .params()
                .map(|param| format!("var _{id} = _p{id};", id = param.id))
                .join("\n"),
            body = self.body.transpile(ctx, generic_args),
        )
    }
}

impl FnItem {
    pub(crate) fn is_inlined(&self, index: &NodeIndex) -> bool {
        self.signature
            .params()
            .any(|param| param.is_ref(index) == Some(true))
            || self.is_ref(index) == Some(true)
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
    fn is_ref(&self, _index: &NodeIndex) -> Option<bool> {
        Some(
            self.return_type
                .iter()
                .next()
                .is_some_and(|return_type| return_type.ref_.iter().len() == 1),
        )
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        if let Some(return_type) = &self.return_type.iter().next() {
            return_type.type_(index)
        } else {
            Some(NodeSource {
                node: NodeRef::NoReturn,
                generic_args: vec![],
            })
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

    fn transpile(
        &self,
        ctx: &mut TranspilationContext<'_>,
        generic_args: &GenericArgs<'_>,
    ) -> String {
        format!(
            "fn _{fn_id}({params}) {return_type}",
            fn_id = self.parent_ids[self.parent_ids.len() - 1],
            params = self
                .params()
                .map(|param| param.transpile(ctx, generic_args))
                .join(", "),
            return_type = self.return_type.transpile(ctx, generic_args),
        )
    }
}

impl FnSignature {
    pub(crate) fn params(&self) -> impl Iterator<Item = &FnParam> + Clone {
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
    pub(crate) fn params(&self) -> impl Iterator<Item = &Rc<FnParam>> + Clone {
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

    fn is_ref(&self, _index: &NodeIndex) -> Option<bool> {
        Some(self.ref_.iter().len() == 1)
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        self.type_.type_(index)
    }

    fn is_transpilable_dependency(&self, _index: &NodeIndex) -> bool {
        false
    }

    fn transpile(
        &self,
        ctx: &mut TranspilationContext<'_>,
        generic_args: &GenericArgs<'_>,
    ) -> String {
        let id = &self.id;
        let type_ = &self.type_.transpile(ctx, generic_args);
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
    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        self.type_.type_(index)
    }

    fn transpile(
        &self,
        ctx: &mut TranspilationContext<'_>,
        generic_args: &GenericArgs<'_>,
    ) -> String {
        let type_ = self.type_.transpile(ctx, generic_args);
        format!("-> {type_}")
    }
}

pub(crate) fn is_const(fn_item_node: &dyn Node) -> bool {
    if let Some(fn_) = (fn_item_node as &dyn Any).downcast_ref::<NativeFnItem>() {
        fn_.const_.iter().len() == 1
    } else if let Some(fn_) = (fn_item_node as &dyn Any).downcast_ref::<FnItem>() {
        fn_.const_.iter().len() == 1
    } else {
        unreachable!("unknown fn item")
    }
}

pub(crate) fn signature(fn_item_node: &dyn Node) -> &FnSignature {
    if let Some(fn_) = (fn_item_node as &dyn Any).downcast_ref::<NativeFnItem>() {
        &fn_.signature
    } else if let Some(fn_) = (fn_item_node as &dyn Any).downcast_ref::<FnItem>() {
        &fn_.signature
    } else {
        unreachable!("unknown fn item")
    }
}
