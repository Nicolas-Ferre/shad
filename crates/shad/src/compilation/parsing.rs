use crate::compilation::node::{Node, NodeProps};
use crate::language::nodes::items::Root;
use crate::{Error, ParsingError};
use itertools::Itertools;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub(crate) fn parse_files(
    files: &HashMap<PathBuf, String>,
    first_node_id: u32,
) -> Result<HashMap<PathBuf, Root>, Error> {
    let mut roots = HashMap::new();
    let mut errors = vec![];
    let mut next_node_id = first_node_id;
    for (path, code) in files {
        match parse_file(path, code, next_node_id) {
            Ok((mut root, new_next_node_id)) => {
                next_node_id = new_next_node_id;
                root.props.slice.clone_from(code);
                roots.insert(path.clone(), root);
            }
            Err(err) => errors.push(err),
        }
    }
    if errors.is_empty() {
        Ok(roots)
    } else {
        Err(Error::Parsing(errors))
    }
}

pub(crate) fn parse_file(
    path: &Path,
    raw_code: &str,
    first_node_id: u32,
) -> Result<(Root, u32), ParsingError> {
    let code = remove_comments(raw_code);
    let mut ctx = ParsingContext {
        path,
        code: &code,
        offset: 0,
        next_node_id: first_node_id,
        parent_ids: vec![],
    };
    let root = Root::parse(&mut ctx).map_err(|mut err| {
        err.code = raw_code.into();
        err
    })?;
    Ok((root, ctx.next_node_id))
}

pub(crate) fn parse_end_of_file(ctx: &mut ParsingContext<'_>) -> Result<NodeProps, ParsingError> {
    if ctx.offset < ctx.code.len() {
        Err(ParsingError {
            expected_tokens: vec!["end of file".into()],
            offset: ctx.offset,
            code: String::new(), // set only at the end to limit performance impact
            path: ctx.path.into(),
            forced: false,
        })
    } else {
        Ok(NodeProps {
            id: ctx.next_node_id(),
            parent_ids: ctx.parent_ids.clone(),
            slice: String::new(),
            span: ctx.code.len()..ctx.code.len(),
            path: ctx.path.into(),
        })
    }
}

pub(crate) fn parse_keyword(
    ctx: &mut ParsingContext<'_>,
    keyword: &str,
) -> Result<NodeProps, ParsingError> {
    if ctx.code[ctx.offset..].starts_with(keyword)
        && is_next_char_valid(ctx.code, ctx.offset + keyword.len())
    {
        let span = ctx.offset..ctx.offset + keyword.len();
        ctx.offset += keyword.len();
        Ok(NodeProps {
            id: ctx.next_node_id(),
            parent_ids: ctx.parent_ids.clone(),
            slice: ctx.code[span.clone()].into(),
            span,
            path: ctx.path.into(),
        })
    } else {
        Err(ParsingError {
            expected_tokens: vec![format!("`{}`", keyword)],
            offset: ctx.offset,
            code: String::new(), // set only at the end to limit performance impact
            path: ctx.path.into(),
            forced: false,
        })
    }
}

pub(crate) fn parse_pattern(
    ctx: &mut ParsingContext<'_>,
    length: usize,
    display_name: &str,
    reserved_keywords: &[&str],
) -> Result<NodeProps, ParsingError> {
    let span = ctx.offset..ctx.offset + length;
    if length > 0
        && !reserved_keywords.contains(&&ctx.code[span.clone()])
        && is_next_char_valid(ctx.code, ctx.offset + length)
    {
        ctx.offset += length;
        Ok(NodeProps {
            id: ctx.next_node_id(),
            parent_ids: ctx.parent_ids.clone(),
            slice: ctx.code[span.clone()].into(),
            span,
            path: ctx.path.into(),
        })
    } else {
        Err(ParsingError {
            expected_tokens: vec![display_name.into()],
            offset: ctx.offset,
            code: String::new(), // set only at the end to limit performance impact
            path: ctx.path.into(),
            forced: false,
        })
    }
}

pub(crate) fn parse_repeated<T: Node>(
    ctx: &mut ParsingContext<'_>,
    min: usize,
    max: usize,
) -> Result<(Vec<Rc<T>>, NodeProps), ParsingError> {
    let id = ctx.next_node_id();
    let span_start = ctx.offset;
    ctx.parent_ids.push(id);
    let mut nodes: Vec<_> = vec![];
    for repeat_index in 0..max {
        let mut local_ctx = ctx.clone();
        match T::parse(&mut local_ctx) {
            Ok(node) => {
                ctx.apply(&local_ctx);
                nodes.push(Rc::new(node));
            }
            Err(err) => {
                if err.forced || repeat_index < min {
                    return Err(err);
                }
                break;
            }
        }
    }
    ctx.parent_ids.pop();
    let span = span_start..ctx.offset;
    Ok((
        nodes,
        NodeProps {
            id,
            parent_ids: ctx.parent_ids.clone(),
            slice: ctx.code[span.clone()].trim().into(),
            span,
            path: ctx.path.into(),
        },
    ))
}

pub(crate) fn is_next_char_valid(code: &str, offset: usize) -> bool {
    let previous_char = code[..offset].chars().last().unwrap_or(' ');
    let next_char = code[offset..].chars().next().unwrap_or(' ');
    !(is_ident_char(previous_char) && is_ident_char(next_char))
}

fn is_ident_char(char: char) -> bool {
    char.is_ascii_alphanumeric() || char == '_'
}

fn remove_comments(code: &str) -> String {
    const COMMENT_PREFIX: &str = "//";
    code.lines()
        .map(|line| {
            if let Some((left, right)) = line.split_once(COMMENT_PREFIX) {
                format!("{left}{}", " ".repeat(right.len() + 2))
            } else {
                line.into()
            }
        })
        .join("\n")
}

#[derive(Debug, Clone)]
pub(crate) struct ParsingContext<'a> {
    pub(crate) path: &'a Path,
    pub(crate) code: &'a str,
    pub(crate) offset: usize,
    pub(crate) next_node_id: u32,
    pub(crate) parent_ids: Vec<u32>,
}

impl ParsingContext<'_> {
    pub(crate) fn apply(&mut self, other: &Self) {
        self.offset = other.offset;
        self.next_node_id = other.next_node_id;
    }

    pub(crate) fn next_node_id(&mut self) -> u32 {
        let index = self.next_node_id;
        self.next_node_id += 1;
        index
    }

    pub(crate) fn parse_spaces(&mut self) {
        let trimmed_code = self.code[self.offset..].trim_start();
        self.offset += self.code[self.offset..].len() - trimmed_code.len();
    }
}
