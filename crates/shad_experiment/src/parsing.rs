#![allow(missing_docs)]

use crate::ast::{AstNode, AstNodeInner};
use crate::config::{Config, KindConfig};
use itertools::Itertools;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

// TODO: improve parsing performance

pub(crate) fn parse_files(
    config: &Config,
    files: &HashMap<PathBuf, String>,
) -> Result<HashMap<PathBuf, (String, Rc<AstNode>)>, Vec<ParsingError>> {
    let mut asts = HashMap::new();
    let mut errors = vec![];
    for (path, code) in files {
        match parse_file(config, path, code) {
            Ok(ast) => {
                asts.insert(path.clone(), (code.clone(), Rc::new(ast)));
            }
            Err(err) => errors.push(err),
        }
    }
    if errors.is_empty() {
        Ok(asts)
    } else {
        Err(errors)
    }
}

fn parse_file(config: &Config, path: &Path, raw_code: &str) -> Result<AstNode, ParsingError> {
    let code = remove_comments(config, raw_code);
    let mut ctx = Context {
        config,
        kind_name: &config.root_kind,
        kind_config: &config.kinds[&config.root_kind],
        path,
        raw_code,
        code: &code,
        offset: 0,
        next_node_id: 0,
        parent_ids: vec![],
    };
    let parsed = parse_node(&mut ctx)?;
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
        Ok(parsed)
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
    let mut nodes: Vec<_> = vec![];
    for repeat_index in 0..ctx.kind_config.max_repeat {
        let mut local_ctx = ctx.clone();
        let node = parse_not_repeated_node(&mut local_ctx);
        match node {
            Ok(node) => {
                ctx.apply(&local_ctx);
                nodes.push(Rc::new(node));
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
    Ok(AstNode {
        id,
        parent_ids: ctx.parent_ids.clone(),
        kind_name,
        kind_config,
        slice: ctx.code[start..ctx.offset].into(),
        offset: start,
        inner: AstNodeInner::Repeated(nodes),
    })
}

fn parse_not_repeated_node(ctx: &mut Context<'_>) -> Result<AstNode, ParsingError> {
    clean_code_prefix(ctx);
    if let Some(pattern) = &ctx.kind_config.clone().pattern {
        parse_pattern(ctx, pattern)
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
    let kind_name = local_ctx.kind_name.into();
    let kind_config = local_ctx.kind_config.clone();
    let mut forced_error = false;
    let children = local_ctx
        .kind_config
        .sequence
        .iter()
        .map(|child_kind_name| {
            local_ctx.kind_name = child_kind_name;
            local_ctx.kind_config = &local_ctx.config.kinds[child_kind_name];
            parse_node(&mut local_ctx)
                .map(|node| (child_kind_name.into(), node))
                .map(|(child_kind_name, node)| {
                    let sequence_error_after = kind_config
                        .sequence_error_after
                        .as_deref()
                        .expect("internal error: missing `sequence_error_after` value");
                    if child_kind_name == sequence_error_after {
                        forced_error = true;
                    }
                    (child_kind_name, Rc::new(node))
                })
        })
        .collect::<Result<HashMap<String, _>, _>>()
        .map_err(|mut err| {
            err.forced = forced_error;
            err
        })?;
    *ctx = local_ctx;
    ctx.parent_ids.pop();
    Ok(AstNode {
        id,
        parent_ids: ctx.parent_ids.clone(),
        kind_name,
        kind_config,
        slice: ctx.code[start..ctx.offset].into(),
        offset: start,
        inner: AstNodeInner::Sequence(children),
    })
}

fn parse_choice(ctx: &mut Context<'_>) -> Result<AstNode, ParsingError> {
    let mut errors = vec![];
    for child_kind_name in &ctx.kind_config.choice {
        let mut local_ctx = ctx.clone();
        local_ctx.kind_name = child_kind_name;
        local_ctx.kind_config = &local_ctx.config.kinds[child_kind_name];
        let node = parse_node(&mut local_ctx);
        match node {
            Ok(node) => {
                *ctx = local_ctx;
                return Ok(node);
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
            .collect(),
        offset: errors[0].offset,
        code: ctx.raw_code.into(),
        path: ctx.path.into(),
        forced: false,
    })
}

fn parse_pattern(ctx: &mut Context<'_>, pattern: &Regex) -> Result<AstNode, ParsingError> {
    let length = if let Some(match_) = pattern.find(&ctx.code[ctx.offset..]) {
        if match_.start() == 0 {
            Some(match_.end())
        } else {
            None
        }
    } else {
        None
    };
    if let Some(length) = length {
        let mut local_ctx = ctx.clone();
        let start = local_ctx.offset;
        local_ctx.offset += length;
        let node = AstNode {
            id: local_ctx.next_node_id(),
            parent_ids: ctx.parent_ids.clone(),
            kind_name: local_ctx.kind_name.into(),
            kind_config: local_ctx.kind_config.clone(),
            slice: local_ctx.code[start..local_ctx.offset].into(),
            offset: start,
            inner: AstNodeInner::Terminal,
        };
        if parse_next_whitespace(&mut local_ctx).is_ok() {
            *ctx = local_ctx;
            return Ok(node);
        }
    }
    Err(ParsingError {
        expected_tokens: vec![ctx
            .kind_config
            .display_name
            .as_deref()
            .unwrap_or(&format!("`{pattern}`"))
            .into()],
        offset: ctx.offset,
        code: ctx.raw_code.into(),
        path: ctx.path.into(),
        forced: false,
    })
}

fn parse_next_whitespace(ctx: &mut Context<'_>) -> Result<(), ()> {
    let previous_char = ctx.code[..ctx.offset].chars().last().unwrap_or(' ');
    let next_char = ctx.code[ctx.offset..].chars().next().unwrap_or(' ');
    if is_ident_char(previous_char) && is_ident_char(next_char) {
        if next_char.is_ascii_whitespace() {
            ctx.offset += next_char.len_utf8();
            Ok(())
        } else {
            Err(())
        }
    } else {
        Ok(())
    }
}

fn clean_code_prefix(ctx: &mut Context<'_>) {
    let trimmed_code = ctx.code[ctx.offset..].trim_start();
    ctx.offset += ctx.code[ctx.offset..].len() - trimmed_code.len();
}

fn is_ident_char(char: char) -> bool {
    char.is_ascii_alphanumeric() || char == '_'
}

#[derive(Debug)]
pub struct ParsingError {
    pub expected_tokens: Vec<String>,
    pub offset: usize,
    pub code: String,
    pub path: PathBuf,
    forced: bool,
}

#[derive(Debug, Clone)]
struct Context<'a> {
    config: &'a Config,
    kind_name: &'a str,
    kind_config: &'a Rc<KindConfig>,
    path: &'a Path,
    raw_code: &'a str,
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
