use crate::compilation::ast::AstNode;
use crate::compilation::transpilation::transpile_node;
use crate::compilation::FileAst;
use crate::config::Config;
use rhai::{Dynamic, Engine, EvalAltResult, Position, Scope, AST};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::atomic::{AtomicU32, Ordering};

pub(crate) static NEXT_BINDING: AtomicU32 = AtomicU32::new(0);

#[derive(Clone)]
pub(crate) struct ScriptContext {
    pub(crate) asts: Rc<HashMap<PathBuf, FileAst>>,
    pub(crate) config: Rc<Config>,
    pub(crate) root_path: PathBuf,
    pub(crate) engine: Rc<RefCell<Option<Engine>>>,
    pub(crate) cache: Rc<RefCell<HashMap<String, AST>>>,
}

impl ScriptContext {
    #[allow(clippy::cast_possible_wrap)]
    pub(crate) fn init(&self) {
        let mut engine = Engine::new();
        engine.register_iterator::<Vec<Rc<AstNode>>>();
        engine.register_fn("req", or_stop::<Rc<AstNode>>);
        engine.register_fn("req", or_stop::<String>);
        engine.register_fn("or", or::<Rc<AstNode>>);
        engine.register_fn("is_some", |o: &mut Option<Rc<AstNode>>| o.is_some());
        engine.register_fn("is_some", |o: &mut Option<f32>| o.is_some());
        engine.register_fn("is_some", |o: &mut Option<i32>| o.is_some());
        engine.register_fn("is_some", |o: &mut Option<u32>| o.is_some());
        engine.register_fn("len", |v: &mut Vec<Rc<AstNode>>| v.len() as i64);
        engine.register_fn("replaced", str::replace::<&str>);
        engine.register_fn("parse_f32", parse_f32);
        engine.register_fn("parse_i32", parse_i32);
        engine.register_fn("parse_u32", parse_u32);
        engine.register_fn("to_str", path_to_str);
        engine.register_fn("next_binding", next_binding);
        engine.register_fn("id", node_id);
        engine.register_fn("slice", node_slice);
        engine.register_fn("kind", node_kind);
        let ctx_clone = self.clone();
        engine.register_fn("type", move |node: &mut Rc<AstNode>| {
            node_type(node, &ctx_clone)
        });
        engine.register_fn("child", node_child);
        engine.register_fn("last_child", node_last_child);
        engine.register_fn("children", node_children);
        engine.register_fn("key", node_key);
        let ctx_clone = self.clone();
        engine.register_fn("source", move |node: &mut Rc<AstNode>| {
            node_source(node, &ctx_clone)
        });
        let ctx_clone = self.clone();
        engine.register_fn("source_key", move |node: &mut Rc<AstNode>| {
            node_source_key(node, &ctx_clone)
        });
        let ctx_clone = self.clone();
        engine.register_fn("import_path", move |node: &mut Rc<AstNode>| {
            node_import_path(node, &ctx_clone)
        });
        let ctx_clone = self.clone();
        engine.register_fn("nested_sources", move |node: &mut Rc<AstNode>| {
            node_nested_sources(node, &ctx_clone)
        });
        let ctx_clone = self.clone();
        engine.register_fn("exists", move |path: &mut PathBuf| {
            does_path_exist(path, &ctx_clone)
        });
        let ctx_clone = self.clone();
        engine.register_fn("transpile", move |node: &mut Rc<AstNode>| {
            transpile_node(&ctx_clone, node)
        });
        let ctx_clone = self.clone();
        engine.register_fn("transpile_type", move |type_: &str| {
            transpile_type(type_, &ctx_clone)
        });
        *self.engine.borrow_mut() = Some(engine);
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

fn or_stop<T>(value: Option<T>) -> Result<T, Box<EvalAltResult>> {
    value.ok_or_else(early_stop_error)
}

fn or<T>(value: Option<T>, other: Option<T>) -> Option<T> {
    value.or(other)
}

fn parse_f32(string: &str) -> Option<f32> {
    f32::from_str(string)
        .ok()
        .and_then(|value| (!value.is_infinite()).then_some(value))
}

fn parse_i32(string: &str) -> Option<i32> {
    i32::from_str(string).ok()
}

fn parse_u32(string: &str) -> Option<u32> {
    u32::from_str(string).ok()
}

#[allow(clippy::ptr_arg)]
fn path_to_str(path: &mut PathBuf) -> String {
    path.display().to_string()
}

fn next_binding() -> u32 {
    NEXT_BINDING.fetch_add(1, Ordering::Relaxed)
}

fn node_id(node: &mut Rc<AstNode>) -> u32 {
    node.id
}

fn node_slice(node: &mut Rc<AstNode>) -> String {
    node.slice.clone()
}

fn node_kind(node: &mut Rc<AstNode>) -> String {
    node.kind_name.clone()
}

fn node_type(node: &Rc<AstNode>, ctx: &ScriptContext) -> Option<String> {
    node.type_(&ctx.asts)
}

fn node_child(node: &mut Rc<AstNode>, child_kind: &str) -> Option<Rc<AstNode>> {
    node.child_option(child_kind).cloned()
}

fn node_last_child(node: &mut Rc<AstNode>) -> Option<Rc<AstNode>> {
    node.children().last().cloned()
}

fn node_children(node: &mut Rc<AstNode>) -> Vec<Rc<AstNode>> {
    node.children().cloned().collect()
}

fn node_key(node: &mut Rc<AstNode>) -> String {
    node.key()
}

fn node_import_path(node: &Rc<AstNode>, ctx: &ScriptContext) -> PathBuf {
    node.import_path(&ctx.root_path)
}

fn does_path_exist(path: &PathBuf, ctx: &ScriptContext) -> bool {
    ctx.asts.contains_key(path)
}

fn node_source(node: &Rc<AstNode>, ctx: &ScriptContext) -> Option<Rc<AstNode>> {
    node.source(&ctx.asts).cloned()
}

fn node_source_key(node: &Rc<AstNode>, ctx: &ScriptContext) -> Option<String> {
    node.source_key(&ctx.asts)
}

fn node_nested_sources(node: &Rc<AstNode>, ctx: &ScriptContext) -> Vec<Rc<AstNode>> {
    node.nested_sources(&ctx.asts)
        .into_iter()
        .cloned()
        .collect()
}

fn transpile_type(type_: &str, ctx: &ScriptContext) -> String {
    ctx.config
        .type_transpilation
        .get(type_)
        .cloned()
        .unwrap_or_else(|| type_.into())
}
