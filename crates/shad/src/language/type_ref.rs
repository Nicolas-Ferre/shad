use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, Node, NodeConfig, NodeType, NodeTypeSource, Repeated};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::items::type_;
use crate::language::items::type_::TypeItem;
use crate::language::keywords::{CloseAngleBracketSymbol, CommaSymbol, OpenAngleBracketSymbol};
use crate::language::patterns::Ident;
use crate::language::{sources, validations};
use crate::ValidationError;
use std::iter;

sequence!(
    #[allow(unused_mut)]
    struct Type {
        ident: Ident,
        generics: Repeated<TypeGenericArgs, 0, 1>,
    }
);

impl NodeConfig for Type {
    fn source_key(&self, _index: &NodeIndex) -> Option<String> {
        Some(sources::type_key(&self.ident))
    }

    fn source<'a>(&'a self, index: &'a NodeIndex) -> Option<&'a dyn Node> {
        self.item(index).map(|item| item as _)
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        Some(NodeType::Source(NodeTypeSource {
            item: self.item(index)?,
            generics: self.generics.iter().next().map(|args| &**args),
        }))
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_missing_source(self, ctx);
        if let Some(item) = self.item(ctx.index) {
            let generic_params = item.generic_params();
            let expected_param_count = generic_params.len();
            let actual_param_count = self
                .generics
                .iter()
                .next()
                .map_or(0, |args| 1 + args.other_args.iter().len());
            if expected_param_count != actual_param_count {
                ctx.errors.push(ValidationError::error(
                    ctx,
                    self,
                    "invalid number of generic parameters",
                    Some(&format!("{actual_param_count} parameter(s) specified here")),
                    &[(
                        item,
                        &format!("{expected_param_count} parameter(s) expected"),
                    )],
                ));
            }
        }
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        self.type_(ctx.index)
            .expect("internal error: type not found")
            .transpiled_name(ctx.index)
    }
}

impl Type {
    pub(crate) fn item<'a>(&self, index: &'a NodeIndex) -> Option<&'a dyn TypeItem> {
        let source = index.search(self, &self.source_key(index)?, sources::type_criteria())?;
        Some(type_::to_item(source))
    }
}

sequence!(
    struct TypeGenericArgs {
        start: OpenAngleBracketSymbol,
        #[force_error(true)]
        first_arg: Type,
        other_args: Repeated<TypeOtherGenericArg, 0, { usize::MAX }>,
        final_comma: Repeated<CommaSymbol, 0, 1>,
        end: CloseAngleBracketSymbol,
    }
);

impl NodeConfig for TypeGenericArgs {}

impl TypeGenericArgs {
    pub(crate) fn args(&self) -> impl Iterator<Item = &Type> {
        iter::once(&*self.first_arg).chain(self.other_args.iter().map(|other| &*other.arg))
    }
}

sequence!(
    #[allow(unused_mut)]
    struct TypeOtherGenericArg {
        comma: CommaSymbol,
        arg: Type,
    }
);

impl NodeConfig for TypeOtherGenericArg {}
