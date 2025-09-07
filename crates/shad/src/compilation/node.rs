use crate::compilation::index::NodeIndex;
use crate::compilation::parsing;
use crate::compilation::parsing::ParsingContext;
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::ParsingError;
use derive_where::derive_where;
use itertools::Itertools;
use std::any::{type_name, Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::ops::{Deref, Range};
use std::path::PathBuf;
use std::rc::Rc;
use std::slice::Iter;
use std::{iter, mem};

#[derive(Debug, Clone)]
#[derive_where(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NodeProps {
    pub(crate) id: u32,
    #[derive_where(skip)]
    pub(crate) parent_ids: Vec<u32>,
    #[derive_where(skip)]
    pub(crate) slice: String,
    #[derive_where(skip)]
    pub(crate) span: Range<usize>,
    #[derive_where(skip)]
    pub(crate) path: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NodeSourceSearchCriteria {
    pub(crate) node_type: fn() -> TypeId,
    pub(crate) can_be_after: bool,
    pub(crate) common_parent_count: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum NodeType<'a> {
    Source(&'a dyn Node),
    NoReturn,
}

impl<'a> NodeType<'a> {
    pub(crate) fn is_no_return(self) -> bool {
        matches!(self, Self::NoReturn)
    }

    pub(crate) fn source(self) -> Option<&'a dyn Node> {
        match self {
            NodeType::Source(source) => Some(source),
            NodeType::NoReturn => None,
        }
    }
}

// coverage: off (most default implementations are unreachable)
#[allow(unused_variables)]
pub(crate) trait NodeConfig {
    fn key(&self) -> Option<String> {
        None
    }

    fn source_key(&self, index: &NodeIndex) -> Option<String> {
        None
    }

    fn source_search_criteria(&self) -> &'static [NodeSourceSearchCriteria] {
        unreachable!("`{}` node has no source criteria", type_name::<Self>())
    }

    fn is_ref(&self, index: &NodeIndex) -> bool {
        unreachable!("`{}` node has no ref checking", type_name::<Self>())
    }

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        unreachable!("`{}` node has no type", type_name::<Self>())
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {}

    fn is_transpilable_dependency(&self, index: &NodeIndex) -> bool {
        unreachable!("`{}` node has no dependency checking", type_name::<Self>())
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        unreachable!("`{}` node has no transpilation", type_name::<Self>())
    }
}
// coverage: on

pub(crate) trait Node: Any + NodeConfig + Debug + Deref<Target = NodeProps> {
    fn parse(ctx: &mut ParsingContext<'_>) -> Result<Self, ParsingError>
    where
        Self: Sized;

    fn index(&self, index: &mut NodeIndex);

    fn validate_nested(&self, ctx: &mut ValidationContext<'_>);

    fn direct_nested_sources<'a>(&self, index: &'a NodeIndex) -> Vec<&'a dyn Node>;

    fn props(&self) -> &NodeProps {
        self
    }

    fn node_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn source<'a>(&self, index: &'a NodeIndex) -> Option<&'a dyn Node> {
        self.source_from_key(index, &self.source_key(index)?)
    }

    fn source_from_key<'a>(&self, index: &'a NodeIndex, key: &str) -> Option<&'a dyn Node> {
        index.search(
            key,
            self.source_search_criteria(),
            &self.path,
            &self.parent_ids,
        )
    }

    fn nested_sources<'a>(&self, index: &'a NodeIndex) -> Vec<&'a dyn Node>
    where
        Self: Sized,
    {
        let mut sources = vec![];
        let mut registered_source_ids = HashSet::new();
        let mut sources_to_process: HashMap<_, _> =
            iter::once((self.id, self as &dyn Node)).collect();
        while !sources_to_process.is_empty() {
            for node in mem::take(&mut sources_to_process).into_values() {
                if registered_source_ids.contains(&node.id) {
                    continue;
                }
                registered_source_ids.insert(node.id);
                for source in node.direct_nested_sources(index) {
                    sources.push(source);
                    sources_to_process.insert(source.id, source);
                }
            }
        }
        sources
            .into_iter()
            .unique_by(|node| node.id)
            .sorted_by_key(|node| node.id)
            .collect()
    }
}

#[derive(Debug)]
pub(crate) struct Repeated<T, const MIN: usize, const MAX: usize> {
    nodes: Vec<Rc<T>>,
    props: NodeProps,
}

impl<T: Node, const MIN: usize, const MAX: usize> Repeated<T, MIN, MAX> {
    pub(crate) fn iter(&self) -> Iter<'_, Rc<T>> {
        self.nodes.iter()
    }

    pub(crate) fn take(&mut self) -> Vec<Rc<T>> {
        mem::take(&mut self.nodes)
    }
}

impl<T: Node, const MAX: usize> Repeated<T, 0, MAX> {
    pub(crate) fn new(props: NodeProps) -> Self {
        Self {
            nodes: vec![],
            props,
        }
    }
}

impl<T: Node, const MIN: usize> Repeated<T, MIN, 1> {
    pub(crate) fn from_node(node: T) -> Self {
        Self {
            props: node.clone(),
            nodes: vec![Rc::new(node)],
        }
    }
}

impl<T: Node, const MIN: usize, const MAX: usize> Deref for Repeated<T, MIN, MAX> {
    type Target = NodeProps;

    fn deref(&self) -> &Self::Target {
        &self.props
    }
}

impl<T: Node, const MIN: usize, const MAX: usize> NodeConfig for Repeated<T, MIN, MAX> {
    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        self.nodes.iter().map(|node| node.transpile(ctx)).join("\n")
    }
}

impl<T: Node, const MIN: usize, const MAX: usize> Node for Repeated<T, MIN, MAX> {
    fn parse(ctx: &mut ParsingContext<'_>) -> Result<Self, ParsingError>
    where
        Self: Sized,
    {
        let (nodes, props) = parsing::parse_repeated::<T>(ctx, MIN, MAX)?;
        Ok(Self { nodes, props })
    }

    fn index(&self, index: &mut NodeIndex) {
        for node in &self.nodes {
            debug_assert!(node.key().is_none());
            node.index(index);
        }
    }

    fn validate_nested(&self, ctx: &mut ValidationContext<'_>) {
        for node in &self.nodes {
            node.validate(ctx);
            node.validate_nested(ctx);
        }
    }

    fn direct_nested_sources<'a>(&self, index: &'a NodeIndex) -> Vec<&'a dyn Node> {
        self.nodes
            .iter()
            .flat_map(|node| node.direct_nested_sources(index))
            .collect()
    }
}

#[derive(Debug)]
pub(crate) struct EndOfFile {
    props: NodeProps,
}

impl NodeConfig for EndOfFile {}

impl Deref for EndOfFile {
    type Target = NodeProps;

    // coverage: off (unused by needed by `Node` trait)
    fn deref(&self) -> &Self::Target {
        &self.props
    }
    // coverage: on
}

impl Node for EndOfFile {
    fn parse(ctx: &mut ParsingContext<'_>) -> Result<Self, ParsingError>
    where
        Self: Sized,
    {
        Ok(Self {
            props: parsing::parse_end_of_file(ctx)?,
        })
    }

    fn index(&self, _index: &mut NodeIndex) {}

    fn validate_nested(&self, _ctx: &mut ValidationContext<'_>) {}

    // coverage: off (unused by needed by `Node` trait)
    fn direct_nested_sources<'a>(&self, _index: &'a NodeIndex) -> Vec<&'a dyn Node> {
        vec![]
    }
    // coverage: on
}

macro_rules! keyword {
    ($typename:ident, $keyword:literal) => {
        #[derive(Debug)]
        pub(crate) struct $typename {
            props: crate::compilation::node::NodeProps,
        }

        impl std::ops::Deref for $typename {
            type Target = crate::compilation::node::NodeProps;

            fn deref(&self) -> &Self::Target {
                &self.props
            }
        }

        impl crate::compilation::node::NodeConfig for $typename {}

        impl crate::compilation::node::Node for $typename {
            fn parse(
                ctx: &mut crate::compilation::parsing::ParsingContext<'_>,
            ) -> Result<Self, crate::compilation::error::ParsingError>
            where
                Self: Sized,
            {
                ctx.parse_spaces();
                Ok(Self {
                    props: crate::compilation::parsing::parse_keyword(ctx, $keyword)?,
                })
            }

            fn index(&self, _index: &mut crate::compilation::index::NodeIndex) {}

            fn validate_nested(
                &self,
                _ctx: &mut crate::compilation::validation::ValidationContext<'_>,
            ) {
            }

            fn direct_nested_sources<'a>(
                &self,
                _index: &'a crate::compilation::index::NodeIndex,
            ) -> Vec<&'a dyn crate::compilation::node::Node> {
                vec![]
            }
        }
    };
}

macro_rules! pattern {
    (
        $typename:ident,
        $display_name:literal,
        $reserved_keywords:ident,
        [$(($min:expr, $max:expr, $fn_:ident($($char_range:expr),+)),)+],
    ) => {
        #[derive(Debug)]
        pub(crate) struct $typename {
            props: crate::compilation::node::NodeProps,
        }

        impl std::ops::Deref for $typename {
            type Target = crate::compilation::node::NodeProps;

            fn deref(&self) -> &Self::Target {
                &self.props
            }
        }

        impl crate::compilation::node::Node for $typename {
            fn parse(
                ctx: &mut crate::compilation::parsing::ParsingContext<'_>,
            ) -> Result<Self, crate::compilation::error::ParsingError>
            where
                Self: Sized,
            {
                ctx.parse_spaces();
                let length = Self::matching_length(&ctx.code[ctx.offset..]);
                Ok(Self {
                    props: crate::compilation::parsing::parse_pattern(
                        ctx,
                        length,
                        $display_name,
                        $reserved_keywords,
                    )?,
                })
            }

            fn index(&self, _index: &mut crate::compilation::index::NodeIndex) {}

            fn validate_nested(
                &self,
                _ctx: &mut crate::compilation::validation::ValidationContext<'_>,
            ) {
            }

            fn direct_nested_sources<'a>(
                &self,
                index: &'a crate::compilation::index::NodeIndex
            ) -> Vec<&'a dyn crate::compilation::node::Node> {
                self.source(index).into_iter().collect()
            }
        }

        impl $typename {
            #[allow(clippy::manual_is_ascii_check, unused_comparisons)]
            fn matching_length(code: &str) -> usize {
                let mut length = 0;
                let mut chars = code.chars();
                $({
                    let mut chars_local = chars.clone();
                    let mut index = 0;
                    while index < $max {
                        if let Some(char) = chars_local.next() {
                            if pattern!(@condition char, $fn_($($char_range),+)) {
                                index += 1;
                                chars.next();
                                continue;
                            }
                        }
                        if index >= $min {
                            break;
                        }
                        return 0;
                    }
                    length += index;
                })+
                length
            }
        }
    };
    (@condition $char:ident, INCLUDE($($char_range:expr),+)) => {
        $($char_range.contains(&$char))||+
    };
    (@condition $char:ident, EXCLUDE($($char_range:expr),+)) => {
        $(!$char_range.contains(&$char))&&+
    };
}

macro_rules! sequence {
    (
        $(#[allow($($attr_token:tt)*)])*
        struct $typename:ident {
            $(
                $(#[force_error($value:literal)])?
                $child:ident: $child_type:ty,
            )*
        }
    ) => {
        $(#[allow($($attr_token)*)])*
        #[derive(Debug)]
        #[derive_where::derive_where(PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub(crate) struct $typename {
            $(#[derive_where(skip)] pub(crate) $child: std::rc::Rc<$child_type>,)*
            pub(crate) props: crate::compilation::node::NodeProps,
        }

        impl std::ops::Deref for $typename {
            type Target = crate::compilation::node::NodeProps;

            fn deref(&self) -> &Self::Target {
                &self.props
            }
        }

        $(#[allow($($attr_token)*)])*
        impl crate::compilation::node::Node for $typename {
            fn parse(
                ctx: &mut crate::compilation::parsing::ParsingContext<'_>
            ) -> Result<Self, crate::compilation::error::ParsingError>
            where
                Self: Sized,
            {
                ctx.parse_spaces();
                let mut local_ctx = ctx.clone();
                let id = local_ctx.next_node_id();
                let span_start = local_ctx.offset;
                let mut forced_error = false;
                local_ctx.parent_ids.push(id);
                $(
                    $(if $value {
                        forced_error = true;
                    })?
                    let $child = <$child_type as crate::compilation::node::Node>::parse(&mut local_ctx)
                        .map_err(|mut err| {
                            err.forced = forced_error;
                            err
                        })?;
                )*
                *ctx = local_ctx;
                ctx.parent_ids.pop();
                let span = span_start..ctx.offset;
                Ok(Self {
                    $($child: std::rc::Rc::new($child),)*
                    props: crate::compilation::node::NodeProps {
                        id,
                        parent_ids: ctx.parent_ids.clone(),
                        slice: ctx.code[span.clone()].trim().into(),
                        span,
                        path: ctx.path.into(),
                    },
                })
            }

            fn index(&self, index: &mut crate::compilation::index::NodeIndex) {
                $(
                    if let Some(key) = self.$child.key() {
                        index.register(key, &self.$child);
                    }
                    self.$child.index(index);
                )*
            }

            fn validate_nested(
                &self,
                ctx: &mut crate::compilation::validation::ValidationContext<'_>,
            ) {
                $(
                    self.$child.validate(ctx);
                    self.$child.validate_nested(ctx);
                )*
            }

            fn direct_nested_sources<'a>(
                &self,
                index: &'a crate::compilation::index::NodeIndex
            ) -> Vec<&'a dyn crate::compilation::node::Node> {
                let mut sources = vec![];
                $(
                    sources.extend(self.source(index));
                    sources.extend(self.$child.direct_nested_sources(index));
                )*
                sources
            }
        }
    };
}

macro_rules! choice {
    (
        $(#[$($attr_token:tt)*])*
        enum $typename:ident {
            $($child:ident($child_type:ty),)*
        }
    ) => {
        $(#[$($attr_token)*])*
        #[derive(Debug)]
        pub(crate) enum $typename {
            $($child(std::rc::Rc<$child_type>),)*
        }

        #[automatically_derived]
        impl std::ops::Deref for $typename {
            type Target = crate::compilation::node::NodeProps;

            fn deref(&self) -> &Self::Target {
                match self {
                    $(Self::$child(child) => &*child,)*
                }
            }
        }

        impl crate::compilation::node::Node for $typename {
            fn parse(
                ctx: &mut crate::compilation::parsing::ParsingContext<'_>
            ) -> Result<Self, crate::compilation::error::ParsingError>
            where
                Self: Sized,
            {
                use itertools::Itertools;
                ctx.parse_spaces();
                let mut errors = vec![];
                $(match <$child_type as crate::compilation::node::Node>::parse(ctx) {
                    Ok(child) => {
                        return Ok(Self::$child(std::rc::Rc::new(child)));
                    },
                    Err(err) => {
                        if err.forced {
                            return Err(err); // no-coverage (false positive)
                        }
                        errors.push(err);
                    }
                })*
                Err(crate::compilation::error::ParsingError {
                    expected_tokens: errors
                        .iter()
                        .flat_map(|err| err.expected_tokens.iter().cloned())
                        .unique()
                        .collect(),
                    offset: ctx.offset,
                    code: String::new(), // set only at the end to limit performance impact
                    path: ctx.path.into(),
                    forced: false,
                })
            }

            fn index(&self, index: &mut crate::compilation::index::NodeIndex) {
                match self {
                    $(Self::$child(child) => {
                        if let Some(key) = child.key() {
                            index.register(key, child);
                        }
                        child.index(index);
                    })*
                }
            }

            fn validate_nested(
                &self,
                ctx: &mut crate::compilation::validation::ValidationContext<'_>,
            ) {
                match self {
                    $(Self::$child(child) => {
                        child.validate(ctx);
                        child.validate_nested(ctx);
                    })*
                }
            }

            fn direct_nested_sources<'a>(
                &self,
                index: &'a crate::compilation::index::NodeIndex
            ) -> Vec<&'a dyn crate::compilation::node::Node> {
                match self {
                    $(Self::$child(child) => child.direct_nested_sources(index),)*
                }
            }
        }

        #[allow(unused)]
        impl $typename {
            pub(crate) fn inner(&self) -> &dyn crate::compilation::node::Node {
                match self {
                    $(Self::$child(child) => child.as_ref(),)*
                }
            }

            pastey::paste! {
                $(pub(crate) fn [<as_ $child:snake>](&self) -> Option<&$child_type> {
                    if let Self::$child(item) = self {
                        Some(item)
                    } else {
                        None
                    }
                })*
            }
        }
    };
}

macro_rules! transform {
    (
        $typename:ident,
        $parsed_type:ty,
        $transformed_type:ty,
        $transform_fn:path
    ) => {
        #[derive(Debug)]
        #[allow(clippy::large_enum_variant)]
        pub(crate) enum $typename {
            Parsed(std::rc::Rc<$parsed_type>),
            Transformed(std::rc::Rc<$transformed_type>),
        }

        impl std::ops::Deref for $typename {
            type Target = crate::compilation::node::NodeProps;

            fn deref(&self) -> &Self::Target {
                match self {
                    Self::Parsed(child) => &*child,
                    Self::Transformed(child) => &*child,
                }
            }
        }

        impl crate::compilation::node::NodeConfig for $typename {
            fn is_ref(&self, index: &crate::compilation::index::NodeIndex) -> bool {
                match self {
                    Self::Parsed(child) => child.is_ref(index),
                    Self::Transformed(child) => child.is_ref(index),
                }
            }

            fn type_<'a>(
                &self,
                index: &'a crate::compilation::index::NodeIndex,
            ) -> Option<crate::compilation::node::NodeType<'a>> {
                match self {
                    Self::Parsed(child) => child.type_(index),
                    Self::Transformed(child) => child.type_(index),
                }
            }

            fn transpile(
                &self,
                ctx: &mut crate::compilation::transpilation::TranspilationContext<'_>,
            ) -> std::string::String {
                match self {
                    Self::Parsed(child) => child.transpile(ctx),
                    Self::Transformed(child) => child.transpile(ctx),
                }
            }
        }

        impl crate::compilation::node::Node for $typename {
            fn parse(
                ctx: &mut crate::compilation::parsing::ParsingContext<'_>,
            ) -> Result<Self, crate::compilation::error::ParsingError>
            where
                Self: Sized,
            {
                Ok($transform_fn(
                    <$parsed_type as crate::compilation::node::Node>::parse(ctx)?,
                ))
            }

            fn index(&self, index: &mut crate::compilation::index::NodeIndex) {
                match self {
                    Self::Parsed(child) => {
                        debug_assert!(child.key().is_none());
                        child.index(index);
                    }
                    Self::Transformed(child) => {
                        debug_assert!(child.key().is_none());
                        child.index(index);
                    }
                }
            }

            fn validate_nested(
                &self,
                ctx: &mut crate::compilation::validation::ValidationContext<'_>,
            ) {
                match self {
                    Self::Parsed(child) => {
                        child.validate(ctx);
                        child.validate_nested(ctx);
                    }
                    Self::Transformed(child) => {
                        child.validate(ctx);
                        child.validate_nested(ctx);
                    }
                }
            }

            fn direct_nested_sources<'a>(
                &self,
                index: &'a crate::compilation::index::NodeIndex,
            ) -> Vec<&'a dyn crate::compilation::node::Node> {
                match self {
                    Self::Parsed(child) => child.direct_nested_sources(index),
                    Self::Transformed(child) => child.direct_nested_sources(index),
                }
            }
        }
    };
}

pub(crate) use choice;
pub(crate) use keyword;
pub(crate) use pattern;
pub(crate) use sequence;
pub(crate) use transform;
