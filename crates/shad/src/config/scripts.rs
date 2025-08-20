use crate::compilation::ast::AstNode;
use crate::compilation::FileAst;
use rhai::{Dynamic, Engine, EvalAltResult, Position, Scope, AST};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;

#[allow(clippy::cast_possible_wrap)]
pub(crate) fn compile(
    code: &str,
    asts: &Rc<HashMap<PathBuf, FileAst>>,
    root_path: &Path,
) -> (AST, Engine) {
    let asts = asts.clone();
    let asts2 = asts.clone();
    let asts3 = asts.clone();
    let asts4 = asts.clone();
    let root_path = root_path.to_path_buf();
    let mut engine = Engine::new();
    (
        engine
            .register_iterator::<Vec<Rc<AstNode>>>()
            .register_fn("req", or_stop::<Rc<AstNode>>)
            .register_fn("req", or_stop::<String>)
            .register_fn("or", or::<Rc<AstNode>>)
            .register_fn("is_some", |o: &mut Option<Rc<AstNode>>| o.is_some())
            .register_fn("is_some", |o: &mut Option<f32>| o.is_some())
            .register_fn("is_some", |o: &mut Option<i32>| o.is_some())
            .register_fn("is_some", |o: &mut Option<u32>| o.is_some())
            .register_fn("len", |v: &mut Vec<Rc<AstNode>>| v.len() as i64)
            .register_fn("parse_f32", parse_f32)
            .register_fn("parse_i32", parse_i32)
            .register_fn("parse_u32", parse_u32)
            .register_fn("slice", node_slice)
            .register_fn("kind", node_kind)
            .register_fn("type", move |node: &mut Rc<AstNode>| node_type(node, &asts))
            .register_fn("child", node_child)
            .register_fn("last_child", node_last_child)
            .register_fn("children", node_children)
            .register_fn("id", node_id)
            .register_fn("key", node_key)
            .register_fn("source", move |node: &mut Rc<AstNode>| {
                node_source(node, &asts2)
            })
            .register_fn("source_key", move |node: &mut Rc<AstNode>| {
                node_source_key(node, &asts3)
            })
            .register_fn("import_path", move |node: &mut Rc<AstNode>| {
                node_import_path(node, &root_path)
            })
            .register_fn("exists", move |path: &mut PathBuf| {
                does_path_exist(path, &asts4)
            })
            .register_fn("to_str", path_to_str)
            .compile(code)
            .expect("internal error: invalid script"),
        engine,
    )
}

pub(crate) fn run<T: Clone + 'static>(
    node: &Rc<AstNode>,
    asts: &Rc<HashMap<PathBuf, FileAst>>,
    script_ast: &AST,
    engine: &Engine,
) -> Option<T> {
    let mut scope = Scope::new();
    scope.push("node", node.clone());
    scope.push("root", asts[&node.path].root.clone());
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

pub(crate) fn compile_and_run<T: Clone + 'static>(
    code: &str,
    node: &Rc<AstNode>,
    asts: &Rc<HashMap<PathBuf, FileAst>>,
    root_path: &Path,
) -> Option<T> {
    let (ast, engine) = compile(code, asts, root_path);
    run(node, asts, &ast, &engine)
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

fn node_slice(node: &mut Rc<AstNode>) -> String {
    node.slice.clone()
}

fn node_kind(node: &mut Rc<AstNode>) -> String {
    node.kind_name.clone()
}

fn node_type(node: &Rc<AstNode>, asts: &HashMap<PathBuf, FileAst>) -> Option<String> {
    node.type_(asts)
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

fn node_id(node: &mut Rc<AstNode>) -> i64 {
    node.id.into()
}

fn node_key(node: &mut Rc<AstNode>) -> String {
    node.key()
}

fn node_import_path(node: &Rc<AstNode>, root_path: &Path) -> PathBuf {
    node.import_path(root_path)
}

fn does_path_exist(path: &PathBuf, asts: &HashMap<PathBuf, FileAst>) -> bool {
    asts.contains_key(path)
}

fn node_source(node: &Rc<AstNode>, asts: &HashMap<PathBuf, FileAst>) -> Option<Rc<AstNode>> {
    node.source(asts).cloned()
}

fn node_source_key(node: &Rc<AstNode>, asts: &HashMap<PathBuf, FileAst>) -> Option<String> {
    node.source_key(asts)
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
