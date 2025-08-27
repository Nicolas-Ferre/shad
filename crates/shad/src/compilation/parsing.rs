#![allow(missing_docs)]

use crate::compilation::ast::AstNode;
use crate::compilation::error::ParsingError;
use crate::config::{BinaryTransformationConfig, Config, KindConfig, PatternPartConfig};
use crate::Error;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub(crate) fn parse_files(
    config: &Config,
    files: &HashMap<PathBuf, String>,
) -> Result<HashMap<PathBuf, (String, Rc<AstNode>)>, Error> {
    let mut asts = HashMap::new();
    let mut errors = vec![];
    let mut next_node_id = 0;
    for (path, code) in files {
        match parse_file(config, path, code, next_node_id) {
            Ok((ast, new_next_node_id)) => {
                next_node_id = new_next_node_id;
                asts.insert(path.clone(), (code.clone(), Rc::new(ast)));
            }
            Err(err) => errors.push(err),
        }
    }
    if errors.is_empty() {
        Ok(asts)
    } else {
        Err(Error::Parsing(errors))
    }
}

fn parse_file(
    config: &Config,
    path: &Path,
    raw_code: &str,
    first_node_id: u32,
) -> Result<(AstNode, u32), ParsingError> {
    let code = remove_comments(config, raw_code);
    let mut ctx = Context {
        config,
        kind_name: &config.root_kind,
        kind_config: &config.kinds[&config.root_kind],
        path,
        code: &code,
        offset: 0,
        next_node_id: first_node_id,
        parent_ids: vec![],
    };
    let parsed = parse_node(&mut ctx).map_err(|mut err| {
        err.code = raw_code.into();
        err
    })?;
    clean_code_prefix(&mut ctx);
    if ctx.offset < ctx.code.len() {
        Err(ParsingError {
            expected_tokens: config.root_expected_first_tokens.clone(),
            offset: ctx.offset,
            code: raw_code.into(),
            path: path.into(),
            forced: false,
        })
    } else {
        Ok((parsed, ctx.next_node_id))
    }
}

fn remove_comments(config: &Config, code: &str) -> String {
    code.lines()
        .map(|line| {
            if let Some((left, right)) = line.split_once(&config.comment_prefix) {
                format!("{left}{}", " ".repeat(right.len() + 2))
            } else {
                line.into()
            }
        })
        .join("\n")
}

fn parse_node(ctx: &mut Context<'_>) -> Result<AstNode, ParsingError> {
    if ctx.kind_config.min_repeat == 1 && ctx.kind_config.max_repeat == 1 {
        parse_not_repeated_node(ctx)
    } else {
        parse_repeated_node(ctx)
    }
}

fn parse_repeated_node(ctx: &mut Context<'_>) -> Result<AstNode, ParsingError> {
    let id = ctx.next_node_id();
    ctx.parent_ids.push(id);
    let start = ctx.offset;
    let kind_name = ctx.kind_name.into();
    let kind_config = ctx.kind_config.clone();
    let mut children: Vec<_> = vec![];
    for repeat_index in 0..ctx.kind_config.max_repeat {
        let mut local_ctx = ctx.clone();
        let node = parse_not_repeated_node(&mut local_ctx);
        match node {
            Ok(node) => {
                ctx.apply(&local_ctx);
                children.push(Rc::new(node));
            }
            Err(err) => {
                if err.forced || repeat_index < local_ctx.kind_config.min_repeat {
                    return Err(err);
                }
                break;
            }
        }
    }
    ctx.parent_ids.pop();
    let slice = &ctx.code[start..ctx.offset];
    Ok(AstNode {
        id,
        parent_ids: ctx.parent_ids.clone(),
        children,
        kind_name,
        kind_config,
        slice: slice.into(),
        span: start..start + slice.len(),
        path: ctx.path.into(),
    })
}

fn parse_not_repeated_node(ctx: &mut Context<'_>) -> Result<AstNode, ParsingError> {
    clean_code_prefix(ctx);
    if let Some(string) = &ctx.kind_config.clone().string {
        parse_string(ctx, string)
    } else if !ctx.kind_config.clone().pattern_parts.is_empty() {
        parse_pattern(ctx)
    } else if !ctx.kind_config.sequence.is_empty() {
        parse_sequence(ctx)
    } else if !ctx.kind_config.choice.is_empty() {
        parse_choice(ctx)
    } else {
        unreachable!("kind config should be valid");
    }
}

fn parse_sequence(ctx: &mut Context<'_>) -> Result<AstNode, ParsingError> {
    let id = ctx.next_node_id();
    ctx.parent_ids.push(id);
    let mut local_ctx = ctx.clone();
    let start = local_ctx.offset;
    let kind_name = local_ctx.kind_name.to_string();
    let kind_config = local_ctx.kind_config.clone();
    let mut forced_error = false;
    let children = local_ctx
        .kind_config
        .sequence
        .iter()
        .map(|child_kind_name| {
            local_ctx.kind_name = child_kind_name;
            local_ctx.kind_config = &local_ctx.config.kinds[child_kind_name];
            parse_node(&mut local_ctx).map(|node| {
                let sequence_error_after = kind_config
                    .sequence_error_after
                    .as_deref()
                    .expect("internal error: missing `sequence_error_after` value");
                if child_kind_name == sequence_error_after {
                    forced_error = true;
                }
                Rc::new(node)
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|mut err| {
            err.forced |= forced_error;
            err
        })?;
    *ctx = local_ctx;
    ctx.parent_ids.pop();
    let slice = &ctx.code[start..ctx.offset];
    let node = AstNode {
        id,
        parent_ids: ctx.parent_ids.clone(),
        children,
        kind_name,
        kind_config,
        slice: slice.into(),
        span: start..start + slice.len(),
        path: ctx.path.into(),
    };
    Ok(transform_node(ctx, node))
}

fn transform_node(ctx: &Context<'_>, node: AstNode) -> AstNode {
    if let Some(config) = &node.kind_config.binary_transformation {
        let operators = node.nested_children_except(&config.operator, Some(&config.operand));
        let operands = node.nested_children_except(&config.operand, Some(&config.operator));
        if operators.is_empty() {
            node
        } else {
            transform_binary_node_inner(ctx, &operators, &operands, config)
        }
    } else if let Some(config) = &node.kind_config.repeated_suffix_transformation {
        let mut prefix = node.child(&config.prefix).clone();
        let suffixes = node.child(&config.suffix).children.clone();
        if suffixes.is_empty() {
            node
        } else {
            for suffix in suffixes {
                prefix = Rc::new(AstNode {
                    id: prefix.id,
                    parent_ids: prefix.parent_ids.clone(),
                    kind_name: config.new_child_kind.clone(),
                    kind_config: ctx.config.kinds[&config.new_child_kind].clone(),
                    slice: format!("{} {}", prefix.slice, suffix.slice),
                    span: prefix.span.start..suffix.span.end,
                    children: vec![prefix, suffix],
                    path: node.path.clone(),
                });
            }
            AstNode {
                id: node.id,
                parent_ids: node.parent_ids,
                children: vec![prefix],
                kind_name: config.new_kind.clone(),
                kind_config: ctx.config.kinds[&config.new_kind].clone(),
                slice: node.slice,
                span: node.span,
                path: node.path,
            }
        }
    } else {
        node
    }
}

fn transform_binary_node_inner(
    ctx: &Context<'_>,
    operators: &[&Rc<AstNode>],
    operands: &[&Rc<AstNode>],
    config: &BinaryTransformationConfig,
) -> AstNode {
    let split_index = split_index(operators, config);
    let operator = operators[split_index];
    let left_operators = &operators[..split_index];
    let right_operators = &operators[split_index + 1..];
    let left_operands = &operands[..=split_index];
    let right_operands = &operands[split_index + 1..];
    let left_node = if left_operators.is_empty() {
        operands[0].clone()
    } else {
        Rc::new(transform_binary_node_inner(
            ctx,
            left_operators,
            left_operands,
            config,
        ))
    };
    let right_node = if right_operators.is_empty() {
        right_operands[0].clone()
    } else {
        Rc::new(transform_binary_node_inner(
            ctx,
            right_operators,
            right_operands,
            config,
        ))
    };
    AstNode {
        id: operator.id,
        parent_ids: operator.parent_ids.clone(),
        kind_name: config.new_kind.clone(),
        kind_config: ctx.config.kinds[&config.new_kind].clone(),
        slice: format!(
            "{} {} {}",
            left_node.slice, operator.slice, right_node.slice
        ),
        span: left_node.span.start..right_node.span.end,
        children: vec![left_node, operator.clone(), right_node],
        path: operator.path.clone(),
    }
}

fn split_index(operators: &[&Rc<AstNode>], config: &BinaryTransformationConfig) -> usize {
    for checked_operators in &config.operator_priority {
        for (index, operator) in operators.iter().enumerate().rev() {
            if checked_operators.contains(&operator.slice) {
                return index;
            }
        }
    }
    unreachable!("no operator found in binary node")
}

fn parse_choice(ctx: &mut Context<'_>) -> Result<AstNode, ParsingError> {
    let mut errors = vec![];
    for child_kind_name in &ctx.kind_config.choice {
        let mut local_ctx = ctx.clone();
        let id = local_ctx.next_node_id();
        let kind_name = local_ctx.kind_name.to_string();
        let kind_config = local_ctx.kind_config.clone();
        local_ctx.kind_name = child_kind_name;
        local_ctx.kind_config = local_ctx
            .config
            .kinds
            .get(child_kind_name)
            .unwrap_or_else(|| panic!("internal error: not found `{child_kind_name}` kind"));
        let node = parse_node(&mut local_ctx);
        match node {
            Ok(node) => {
                *ctx = local_ctx;
                return Ok(AstNode {
                    id,
                    parent_ids: ctx.parent_ids.clone(),
                    kind_name,
                    kind_config,
                    slice: node.slice.clone(),
                    span: node.span.clone(),
                    path: node.path.clone(),
                    children: vec![Rc::new(node)],
                });
            }
            Err(err) => {
                if err.forced {
                    return Err(err);
                }
                errors.push(err);
            }
        }
    }
    Err(ParsingError {
        expected_tokens: errors
            .iter()
            .flat_map(|err| err.expected_tokens.iter().cloned())
            .unique()
            .collect(),
        offset: errors[0].offset,
        code: String::new(),
        path: ctx.path.into(),
        forced: false,
    })
}

fn parse_pattern(ctx: &mut Context<'_>) -> Result<AstNode, ParsingError> {
    let length = pattern_length(ctx.code, ctx.offset, &ctx.kind_config.pattern_parts);
    if let Some(length) = length {
        if let Some(value) = parse_slice(ctx, length, true) {
            return Ok(value);
        }
    }
    Err(ParsingError {
        expected_tokens: vec![ctx
            .kind_config
            .display_name
            .clone()
            .expect("internal error: pattern node missing `display_name` property")],
        offset: ctx.offset,
        code: String::new(),
        path: ctx.path.into(),
        forced: false,
    })
}

fn parse_string(ctx: &mut Context<'_>, string: &str) -> Result<AstNode, ParsingError> {
    let length = if ctx.code[ctx.offset..].starts_with(string) {
        Some(string.len())
    } else {
        None
    };
    if let Some(length) = length {
        if let Some(value) = parse_slice(ctx, length, false) {
            return Ok(value);
        }
    }
    Err(ParsingError {
        expected_tokens: vec![ctx
            .kind_config
            .display_name
            .as_deref()
            .unwrap_or(&format!("`{string}`"))
            .into()],
        offset: ctx.offset,
        code: String::new(),
        path: ctx.path.into(),
        forced: false,
    })
}

fn parse_slice(ctx: &mut Context<'_>, length: usize, check_keyword: bool) -> Option<AstNode> {
    let mut local_ctx = ctx.clone();
    let start = local_ctx.offset;
    local_ctx.offset += length;
    let slice = &local_ctx.code[start..local_ctx.offset];
    let node = AstNode {
        id: local_ctx.next_node_id(),
        parent_ids: ctx.parent_ids.clone(),
        children: vec![],
        kind_name: local_ctx.kind_name.into(),
        kind_config: local_ctx.kind_config.clone(),
        slice: slice.into(),
        span: start..start + slice.len(),
        path: ctx.path.into(),
    };
    if is_next_char_valid(&local_ctx) && (!check_keyword || !is_keyword(&local_ctx, slice)) {
        *ctx = local_ctx;
        Some(node)
    } else {
        None
    }
}

fn pattern_length(
    string: &str,
    offset: usize,
    pattern_parts: &[PatternPartConfig],
) -> Option<usize> {
    let mut new_offset = offset;
    for pattern_part in pattern_parts {
        let mut length = 0;
        for _ in 0..pattern_part.max_length {
            let char = string[new_offset..].chars().next().unwrap_or(' ');
            if pattern_part
                .char_ranges
                .iter()
                .any(|range| range.contains(&char))
            {
                new_offset += char.len_utf8();
                length += char.len_utf8();
            } else if length < pattern_part.min_length {
                return None;
            } else {
                break;
            }
        }
    }
    Some(new_offset - offset)
}

fn is_next_char_valid(ctx: &Context<'_>) -> bool {
    let previous_char = ctx.code[..ctx.offset].chars().last().unwrap_or(' ');
    let next_char = ctx.code[ctx.offset..].chars().next().unwrap_or(' ');
    !(is_ident_char(previous_char) && is_ident_char(next_char))
}

fn is_keyword(ctx: &Context<'_>, slice: &str) -> bool {
    ctx.config
        .kinds
        .values()
        .filter_map(|config| config.string.as_ref())
        .any(|keyword| slice == keyword)
}

fn clean_code_prefix(ctx: &mut Context<'_>) {
    let trimmed_code = ctx.code[ctx.offset..].trim_start();
    ctx.offset += ctx.code[ctx.offset..].len() - trimmed_code.len();
}

fn is_ident_char(char: char) -> bool {
    char.is_ascii_alphanumeric() || char == '_'
}

#[derive(Debug, Clone)]
struct Context<'a> {
    config: &'a Config,
    kind_name: &'a str,
    kind_config: &'a Rc<KindConfig>,
    path: &'a Path,
    code: &'a str,
    offset: usize,
    next_node_id: u32,
    parent_ids: Vec<u32>,
}

impl Context<'_> {
    fn apply(&mut self, other: &Self) {
        self.offset = other.offset;
        self.next_node_id = other.next_node_id;
    }

    fn next_node_id(&mut self) -> u32 {
        let index = self.next_node_id;
        self.next_node_id += 1;
        index
    }
}
