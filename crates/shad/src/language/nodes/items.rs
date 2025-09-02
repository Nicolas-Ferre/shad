use crate::compilation::index::NodeIndex;
use crate::compilation::node::{choice, sequence, EndOfFile, Node, NodeConfig, Repeated};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::compilation::FILE_EXT;
use crate::language::nodes::expressions::TypedExpr;
use crate::language::nodes::statements::Stmt;
use crate::language::nodes::terminals::{
    ArrowSymbol, BufKeyword, CloseCurlyBracketSymbol, CloseParenthesisSymbol, ColonSymbol,
    CommaSymbol, DotSymbol, EqSymbol, FnKeyword, Ident, ImportKeyword, InitKeyword, NativeKeyword,
    OpenCurlyBracketSymbol, OpenParenthesisSymbol, RunKeyword, SemicolonSymbol, StringLiteral,
    TildeSymbol,
};
use crate::language::sources;
use crate::ValidationError;
use indoc::indoc;
use itertools::Itertools;
use std::any::TypeId;
use std::iter;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub(crate) const NO_RETURN_TYPE: &str = "<no return>";

sequence!(
    struct Root {
        items: Repeated<Item, 0, { usize::MAX }>,
        #[force_error(true)]
        eof: EndOfFile,
    }
);

impl NodeConfig for Root {}

choice!(
    enum Item {
        Import(ImportItem),
        Buffer(BufferItem),
        Init(InitItem),
        Run(RunItem),
        NativeFn(NativeFnItem),
        Fn(FnItem),
    }
);

impl NodeConfig for Item {}

sequence!(
    struct ImportItem {
        import: ImportKeyword,
        #[force_error(true)]
        path_prefix: Repeated<ImportPathPrefix, 0, { usize::MAX }>,
        path_suffix: Ident,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for ImportItem {
    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        let path = self.import_path(ctx.root_path);
        if !path.exists() {
            ctx.errors.push(ValidationError::error(
                ctx,
                self,
                "imported file not found",
                Some(&format!("no file found at `{}`", path.display())),
                &[],
            ));
        }
    }
}

impl ImportItem {
    pub(crate) fn import_path(&self, root_path: &Path) -> PathBuf {
        let segments = self
            .path_prefix
            .iter()
            .map(|prefix| prefix.segment.clone())
            .chain([Rc::new(ImportPathSegment::Ident(self.path_suffix.clone()))])
            .collect::<Vec<_>>();
        let mut path = match &*segments[0] {
            ImportPathSegment::Parent(_) => self.path.clone(),
            ImportPathSegment::Ident(_) => root_path.to_path_buf(),
        };
        for segment in segments {
            match &*segment {
                ImportPathSegment::Parent(_) => path = path.parent().unwrap_or(&path).to_path_buf(),
                ImportPathSegment::Ident(ident) => path.push(&ident.slice),
            }
        }
        path.set_extension(FILE_EXT);
        path
    }
}

sequence!(
    #[allow(unused_mut)]
    struct ImportPathPrefix {
        segment: ImportPathSegment,
        dot: DotSymbol,
    }
);

impl NodeConfig for ImportPathPrefix {}

choice!(
    enum ImportPathSegment {
        Parent(TildeSymbol),
        Ident(Ident),
    }
);

impl NodeConfig for ImportPathSegment {}

sequence!(
    struct BufferItem {
        buf: BufKeyword,
        #[force_error(true)]
        ident: Ident,
        eq: EqSymbol,
        expr: TypedExpr,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for BufferItem {
    fn key(&self) -> Option<String> {
        Some(sources::variable_key(&self.ident))
    }

    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        if is_item_recursive(self, index) {
            None
        } else {
            self.expr.expr_type(index)
        }
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        check_duplicated_items(self, ctx);
        check_recursive_items(self, ctx);
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        format!(
            indoc!(
                "@group(0) @binding({next_binding})
                var<storage, read_write> _{id}: {type_};"
            ),
            id = self.id,
            type_ = transpile_type(
                self.expr_type(ctx.index)
                    .expect("internal error: cannot calculate buffer type")
            ),
            next_binding = ctx.next_binding(),
        )
    }
}

impl BufferItem {
    pub(crate) fn transpile_shader(&self, ctx: &mut TranspilationContext<'_>) -> String {
        format!(
            indoc!(
                "{dependencies}
                {self_}

                @compute
                @workgroup_size(1, 1, 1)
                fn main() {{
                    _{id} = {expr};
                }}"
            ),
            id = self.id,
            expr = self.expr.transpile(ctx),
            dependencies = transpiled_dependencies(ctx, self),
            self_ = self.transpile(ctx),
        )
    }

    pub(crate) fn item_path(&self, root_path: &Path) -> String {
        format!(
            "{}.{}",
            self.path
                .strip_prefix(root_path)
                .expect("internal error: invalid root path")
                .with_extension("")
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .join("."),
            self.ident.slice
        )
    }
}

sequence!(
    struct InitItem {
        init: InitKeyword,
        #[force_error(true)]
        block: NonReturnBlock,
    }
);

impl NodeConfig for InitItem {}

impl InitItem {
    pub(crate) fn transpile_shader(&self, ctx: &mut TranspilationContext<'_>) -> String {
        format!(
            indoc!(
                "{dependencies}

                @compute
                @workgroup_size(1, 1, 1)
                fn main() {{
                {block}
                }}"
            ),
            dependencies = transpiled_dependencies(ctx, self),
            block = self.block.transpile(ctx),
        )
    }
}

sequence!(
    struct RunItem {
        run: RunKeyword,
        #[force_error(true)]
        block: NonReturnBlock,
    }
);

impl NodeConfig for RunItem {}

impl RunItem {
    pub(crate) fn transpile_shader(&self, ctx: &mut TranspilationContext<'_>) -> String {
        format!(
            indoc!(
                "{dependencies}

                @compute
                @workgroup_size(1, 1, 1)
                fn main() {{
                {block}
                }}"
            ),
            dependencies = transpiled_dependencies(ctx, self),
            block = self.block.transpile(ctx),
        )
    }
}

sequence!(
    struct NativeFnItem {
        native: NativeKeyword,
        fn_: FnKeyword,
        #[force_error(true)]
        ident: Ident,
        params_start: OpenParenthesisSymbol,
        params: Repeated<FnParamGroup, 0, 1>,
        params_end: CloseParenthesisSymbol,
        return_type: Repeated<FnReturnType, 0, 1>,
        eq: EqSymbol,
        transpilation: StringLiteral,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for NativeFnItem {
    fn key(&self) -> Option<String> {
        Some(sources::fn_key_from_params(&self.ident, &self.params))
    }

    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        if let Some(return_type) = &self.return_type.iter().next() {
            return_type.expr_type(index)
        } else {
            Some(NO_RETURN_TYPE.into())
        }
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        check_duplicated_items(self, ctx);
        check_duplicated_fn_params(&self.params().collect::<Vec<_>>(), ctx);
    }
}

impl NativeFnItem {
    pub(crate) fn params(&self) -> impl Iterator<Item = &FnParam> {
        self.params
            .iter()
            .flat_map(|params| params.params())
            .map(|param| &**param)
    }
}

// TODO: create FnSignature node common between NativeFnItem and FnItem
sequence!(
    struct FnItem {
        fn_: FnKeyword,
        #[force_error(true)]
        ident: Ident,
        params_start: OpenParenthesisSymbol,
        params: Repeated<FnParamGroup, 0, 1>,
        params_end: CloseParenthesisSymbol,
        return_type: Repeated<FnReturnType, 0, 1>,
        body: Block,
    }
);

impl NodeConfig for FnItem {
    fn key(&self) -> Option<String> {
        Some(sources::fn_key_from_params(&self.ident, &self.params))
    }

    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        if let Some(return_type) = &self.return_type.iter().next() {
            return_type.expr_type(index)
        } else {
            Some(NO_RETURN_TYPE.into())
        }
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        check_duplicated_items(self, ctx);
        check_recursive_items(self, ctx);
        check_duplicated_fn_params(&self.params().collect::<Vec<_>>(), ctx);
        let return_stmt = self.body.last_stmt().and_then(|stmt| stmt.return_());
        let return_type = self.return_type.iter().next();
        if let (None, Some(return_type)) = (return_stmt, return_type) {
            ctx.errors.push(ValidationError::error(
                ctx,
                &*self.body,
                "missing return statement",
                Some("last statement should be a `return` statement"),
                &[(&**return_type, "the function has a return type")],
            ));
        }
        if let (Some(return_stmt), Some(expected_type)) = (return_stmt, self.expr_type(ctx.index)) {
            if let Some(actual_type) = return_stmt.expr_type(ctx.index) {
                if actual_type != NO_RETURN_TYPE && actual_type != expected_type {
                    ctx.errors.push(ValidationError::error(
                        ctx,
                        &*return_stmt.expr,
                        "invalid returned type",
                        Some(&format!("returned type is `{actual_type}`")),
                        &[(
                            &*self.return_type,
                            &format!("expected type is `{expected_type}`"),
                        )],
                    ));
                }
            }
        }
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        format!(
            indoc!(
                "fn _{id}({params}) {return_type} {{
                {param_vars}
                {body}
                }}"
            ),
            id = self.id,
            params = self.params().map(|param| param.transpile(ctx)).join(", "),
            return_type = self.return_type.transpile(ctx),
            param_vars = self
                .params()
                .map(|param| format!("var _{id} = _p{id};", id = param.id))
                .join("\n"),
            body = self.body.transpile(ctx),
        )
    }
}

impl FnItem {
    pub(crate) fn params(&self) -> impl Iterator<Item = &FnParam> {
        self.params
            .iter()
            .flat_map(|params| params.params())
            .map(|param| &**param)
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
        type_: Type,
    }
);

impl NodeConfig for FnParam {
    fn key(&self) -> Option<String> {
        Some(sources::variable_key(&self.ident))
    }

    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        self.type_.expr_type(index)
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
        type_: Type,
    }
);

impl NodeConfig for FnReturnType {
    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        self.type_.expr_type(index)
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let type_ = self.type_.transpile(ctx);
        format!("-> {type_}")
    }
}

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

sequence!(
    #[allow(unused_mut)]
    struct NonReturnBlock {
        inner: Block,
    }
);

impl NodeConfig for NonReturnBlock {
    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if let Some(return_stmt) = self.inner.statements.iter().find_map(|stmt| stmt.return_()) {
            ctx.errors.push(ValidationError::error(
                ctx,
                return_stmt,
                "`return` statement used outside a function",
                Some("not allowed statement"),
                &[],
            ));
        }
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        self.inner.transpile(ctx)
    }
}

sequence!(
    struct Block {
        open: OpenCurlyBracketSymbol,
        #[force_error(true)]
        statements: Repeated<Stmt, 0, { usize::MAX }>,
        close: CloseCurlyBracketSymbol,
    }
);

impl NodeConfig for Block {
    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        let last_stmt_id = self.last_stmt().map_or(0, |stmt| stmt.inner().id);
        for stmt in self.statements.iter() {
            if let Some(return_stmt) = stmt.return_() {
                if return_stmt.id != last_stmt_id {
                    ctx.errors.push(ValidationError::error(
                        ctx,
                        return_stmt,
                        "`return` statement before end of the block",
                        Some("only allowed at the end of a block"),
                        &[],
                    ));
                }
            }
        }
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        self.statements.transpile(ctx)
    }
}

impl Block {
    fn last_stmt(&self) -> Option<&Stmt> {
        self.statements.iter().last().map(|stmt| &**stmt)
    }
}

fn check_duplicated_items(item: &impl Node, ctx: &mut ValidationContext<'_>) {
    let key = item
        .key()
        .expect("internal error: cannot calculate item key");
    for other_item in ctx.roots[&item.path].items.iter() {
        let other_item = other_item.inner();
        if other_item.id < item.id && other_item.key().as_ref() == Some(&key) {
            ctx.errors.push(ValidationError::error(
                ctx,
                item,
                &format!("{key} defined multiple times"),
                Some("duplicated item"),
                &[(other_item, "same item defined here")],
            ));
        }
    }
}

fn is_item_recursive(item: &impl Node, index: &NodeIndex) -> bool {
    item.nested_sources(index)
        .iter()
        .any(|source| source.id == item.id)
}

fn check_recursive_items(item: &impl Node, ctx: &mut ValidationContext<'_>) {
    if is_item_recursive(item, ctx.index) {
        ctx.errors.push(ValidationError::error(
            ctx,
            item,
            "item definition with circular dependency",
            Some("this item is directly or indirectly referring to itself"),
            &[],
        ));
    }
}

fn check_duplicated_fn_params(params: &[&FnParam], ctx: &mut ValidationContext<'_>) {
    for &param1 in params {
        for &param2 in params {
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

fn transpiled_dependencies(ctx: &mut TranspilationContext<'_>, item: &impl Node) -> String {
    item.nested_sources(ctx.index)
        .into_iter()
        .filter(|source| {
            [TypeId::of::<BufferItem>(), TypeId::of::<FnItem>()].contains(&(*source).node_type_id())
        })
        .map(|source| source.transpile(ctx))
        .join("\n")
}

fn transpile_type(type_name: String) -> String {
    if type_name == "bool" {
        "u32".into()
    } else {
        type_name
    }
}
