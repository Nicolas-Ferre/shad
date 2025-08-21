use crate::compilation::ast::AstNode;
use crate::compilation::transpilation::transpile_node;
use crate::compilation::FileAst;
use crate::config::Config;
use itertools::Itertools;
use rhai::{Dynamic, Engine, EvalAltResult, Position, Scope, AST};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;

#[derive(Clone)]
pub(crate) struct ScriptContext {
    pub(crate) asts: Rc<HashMap<PathBuf, FileAst>>,
    pub(crate) config: Rc<Config>,
    pub(crate) root_path: PathBuf,
    pub(crate) engine: Rc<RefCell<Option<Engine>>>,
    pub(crate) next_binding: Rc<Cell<u32>>,
    pub(crate) cache: Rc<RefCell<HashMap<String, AST>>>,
}

impl ScriptContext {
    pub(crate) fn dummy() -> Self {
        let ctx = Self {
            asts: Rc::new(HashMap::default()),
            config: Rc::new(Config {
                root_kind: String::new(),
                root_expected_first_tokens: vec![],
                comment_prefix: String::new(),
                import_index_key: String::new(),
                type_transpilation: HashMap::default(),
                kinds: HashMap::default(),
            }),
            root_path: PathBuf::default(),
            engine: Rc::default(),
            next_binding: Rc::default(),
            cache: Rc::default(),
        };
        ctx.init();
        ctx
    }

    pub(crate) fn new(
        config: &Rc<Config>,
        asts: &Rc<HashMap<PathBuf, FileAst>>,
        root_path: &Path,
    ) -> Self {
        let ctx = Self {
            asts: asts.clone(),
            config: config.clone(),
            root_path: root_path.to_path_buf(),
            engine: Rc::default(),
            next_binding: Rc::default(),
            cache: Rc::default(),
        };
        ctx.init();
        ctx
    }

    #[allow(clippy::cast_possible_wrap)]
    fn init(&self) {
        let mut engine = Engine::new();
        engine.register_iterator::<Vec<Rc<AstNode>>>();
        self.register_node_functions(&mut engine);
        engine.register_fn("unwrap_or_stop", unwrap_or_stop::<Dynamic>);
        engine.register_fn("unwrap_or", Option::<Dynamic>::unwrap_or);
        engine.register_fn("is_some", |o: &mut Option<Dynamic>| o.is_some());
        engine.register_fn("is_some", |o: &mut Option<f32>| o.is_some());
        engine.register_fn("is_some", |o: &mut Option<i32>| o.is_some());
        engine.register_fn("is_some", |o: &mut Option<u32>| o.is_some());
        engine.register_fn("last", |v: &mut Vec<Dynamic>| v.last().cloned());
        engine.register_fn("join", |v: &mut Vec<Dynamic>, sep: &str| {
            v.iter().map(ToString::to_string).join(sep)
        });
        engine.register_fn("replaced", str::replace::<&str>);
        engine.register_fn("parse_f32", |string: &str| {
            f32::from_str(string)
                .ok()
                .and_then(|value| (!value.is_infinite()).then_some(value))
        });
        engine.register_fn("parse_i32", |string: &str| i32::from_str(string).ok());
        engine.register_fn("parse_u32", |string: &str| u32::from_str(string).ok());
        engine.register_fn("to_str", |path: &mut PathBuf| path.display().to_string());
        let ctx_clone = self.clone();
        engine.register_fn("next_binding", move || {
            let binding = ctx_clone.next_binding.get();
            ctx_clone.next_binding.replace(binding + 1);
            binding
        });
        let ctx_clone = self.clone();
        engine.register_fn("exists", move |path: &mut PathBuf| {
            ctx_clone.asts.contains_key(path)
        });
        let ctx_clone = self.clone();
        engine.register_fn("type_wgsl", move |type_: &str| {
            ctx_clone
                .config
                .type_transpilation
                .get(type_)
                .cloned()
                .unwrap_or_else(|| type_.into())
        });
        for kind in self.config.kinds.keys() {
            let kind_clone = kind.clone();
            engine.register_get(kind, move |node: &mut Rc<AstNode>| {
                node.child(&kind_clone).clone()
            });
        }
        *self.engine.borrow_mut() = Some(engine);
    }

    fn register_node_functions(&self, engine: &mut Engine) {
        engine.register_fn("id", |node: &mut Rc<AstNode>| node.id);
        engine.register_fn("slice", |node: &mut Rc<AstNode>| node.slice.clone());
        engine.register_fn("kind", |node: &mut Rc<AstNode>| node.kind_name.clone());
        let ctx_clone = self.clone();
        engine.register_fn("type", move |node: &mut Rc<AstNode>| {
            unwrap_or_stop(node.type_(&ctx_clone.asts))
        });
        engine.register_fn("children", |node: &mut Rc<AstNode>| {
            node.children()
                .cloned()
                .map(Dynamic::from)
                .collect::<Vec<_>>()
        });
        engine.register_fn("nested_children", |node: &mut Rc<AstNode>, kind: &str| {
            let mut children: Vec<Dynamic> = vec![];
            node.scan(&mut |scanned| {
                if scanned.kind_name == kind {
                    children.push(Dynamic::from(scanned.clone()));
                    return true;
                }
                false
            });
            children
        });
        engine.register_fn("key", |node: &mut Rc<AstNode>| node.key());
        let ctx_clone = self.clone();
        engine.register_fn("source", move |node: &mut Rc<AstNode>| {
            node.source(&ctx_clone.asts)
                .expect("internal error: source not found")
                .clone()
        });
        let ctx_clone = self.clone();
        engine.register_fn("has_source", move |node: &mut Rc<AstNode>| {
            node.source(&ctx_clone.asts).is_some()
        });
        let ctx_clone = self.clone();
        engine.register_fn("source_key", move |node: &mut Rc<AstNode>| {
            unwrap_or_stop(node.source_key(&ctx_clone.asts))
        });
        let ctx_clone = self.clone();
        engine.register_fn("import_path", move |node: &mut Rc<AstNode>| {
            node.import_path(&ctx_clone.root_path)
        });
        let ctx_clone = self.clone();
        engine.register_fn("nested_sources", move |node: &mut Rc<AstNode>| {
            node.nested_sources(&ctx_clone.asts)
                .into_iter()
                .cloned()
                .map(Dynamic::from)
                .collect::<Vec<_>>()
        });
        let ctx_clone = self.clone();
        engine.register_fn("wgsl", move |node: &mut Rc<AstNode>| {
            transpile_node(&ctx_clone, node)
        });
    }
}

pub(crate) fn compile_and_run<T: Clone + 'static>(
    code: &str,
    node: &Rc<AstNode>,
    ctx: &ScriptContext,
) -> Option<T> {
    let engine = ctx.engine.borrow();
    let engine = engine
        .as_ref()
        .expect("internal error: script engine not initialized");
    let ast = ctx
        .cache
        .borrow_mut()
        .entry(code.into())
        .or_insert_with(|| {
            engine
                .compile(code)
                .expect("internal error: invalid script")
        })
        .clone();
    run(node, ctx, &ast, engine)
}

fn run<T: Clone + 'static>(
    node: &Rc<AstNode>,
    ctx: &ScriptContext,
    script_ast: &AST,
    engine: &Engine,
) -> Option<T> {
    let mut scope = Scope::new();
    scope.push("node", node.clone());
    scope.push("root", ctx.asts[&node.path].root.clone());
    match engine.eval_ast_with_scope::<T>(&mut scope, script_ast) {
        Ok(result) => Some(result),
        Err(err) => {
            if let EvalAltResult::ErrorMismatchOutputType(_, _, _) = *err {
                None
            } else {
                unreachable!("{err}")
            }
        }
    }
}

#[allow(clippy::unnecessary_box_returns)]
fn early_stop_error() -> Box<EvalAltResult> {
    Box::new(EvalAltResult::Exit(Dynamic::from(()), Position::NONE))
}

fn unwrap_or_stop<T>(value: Option<T>) -> Result<T, Box<EvalAltResult>> {
    value.ok_or_else(early_stop_error)
}
