use crate::transpilation::{node_code, Context, KindPlaceholder};
use crate::AstNode;
use itertools::Itertools;

pub(crate) fn run(ctx: &mut Context<'_>, node: &AstNode, placeholder: &KindPlaceholder) -> String {
    match placeholder.name.as_str() {
        "static" => static_(placeholder),
        "binding" => binding(ctx),
        "number_slice" => number_slice(node),
        "self" => self_(ctx, node),
        "child" => child(ctx, node, placeholder),
        "nested_sources" => nested_sources(ctx, node, placeholder),
        "self_id" => self_id(node),
        "source_id" => source_id(ctx, node),
        "expr_type" => expr_type(ctx, node, placeholder),
        _ => unreachable!("undefined `{}` transpilation step", placeholder.name),
    }
}

fn static_(placeholder: &KindPlaceholder) -> String {
    placeholder.params[0].clone()
}

fn binding(ctx: &mut Context<'_>) -> String {
    ctx.generate_binding().to_string()
}

fn number_slice(node: &AstNode) -> String {
    node.slice.replace("_", "")
}

fn self_(ctx: &mut Context<'_>, node: &AstNode) -> String {
    node_code(ctx, node)
}

fn child(ctx: &mut Context<'_>, node: &AstNode, placeholder: &KindPlaceholder) -> String {
    let child_name = &placeholder.params[0];
    node_code(ctx, node.child(child_name))
}

fn nested_sources(ctx: &mut Context<'_>, node: &AstNode, placeholder: &KindPlaceholder) -> String {
    let accepted_kinds: Vec<_> = placeholder.params[0].split(';').collect();
    node.nested_sources(ctx.asts, ctx.path)
        .into_iter()
        .filter(|source| accepted_kinds.contains(&source.kind_name.as_str()))
        .map(|source| node_code(ctx, source))
        .join("\n")
}

fn self_id(node: &AstNode) -> String {
    node.id.to_string()
}

fn source_id(ctx: &Context<'_>, node: &AstNode) -> String {
    node.source(ctx.asts, ctx.path)
        .expect("internal error: source not found")
        .id
        .to_string()
}

fn expr_type(ctx: &Context<'_>, node: &AstNode, placeholder: &KindPlaceholder) -> String {
    node.child(&placeholder.params[0])
        .type_(ctx.asts, ctx.path)
        .expect("internal error: cannot transpile expression type")
}
