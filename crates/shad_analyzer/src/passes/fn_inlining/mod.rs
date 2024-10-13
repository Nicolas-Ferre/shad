use crate::passes::fn_inlining::replacement::StatementInline;
use crate::passes::fn_inlining::split::StatementSplitContext;
use crate::{Asg, AsgFn, AsgStatement, FunctionListing};
use shad_parser::AstFnQualifier;
use split::StatementSplit;
use std::mem;

mod replacement;
mod split;

pub(crate) fn inline_fns(asg: &mut Asg) {
    let mut are_functions_inlined: Vec<_> = asg
        .function_bodies
        .iter()
        .map(|body| body.fn_.ast.qualifier == AstFnQualifier::Gpu)
        .collect();
    while !are_functions_inlined.iter().all(|&is_inlined| is_inlined) {
        let fns = asg.functions.values().cloned().collect::<Vec<_>>();
        for fn_ in fns {
            if !are_functions_inlined[fn_.index]
                && are_all_dependent_fns_inlined(asg, &are_functions_inlined, &fn_)
            {
                let statements = mem::take(&mut asg.function_bodies[fn_.index].statements);
                asg.function_bodies[fn_.index].statements = inline(asg, statements);
                are_functions_inlined[fn_.index] = true;
            }
        }
    }
    asg.buffer_inits = mem::take(&mut asg.buffer_inits)
        .into_iter()
        .map(|statements| inline(asg, statements))
        .collect();
    asg.run_blocks = mem::take(&mut asg.run_blocks)
        .into_iter()
        .map(|statements| inline(asg, statements))
        .collect();
}

fn are_all_dependent_fns_inlined(asg: &Asg, are_fns_inlined: &[bool], fn_: &AsgFn) -> bool {
    FunctionListing::slice_functions(&asg.function_bodies[fn_.index].statements, asg)
        .iter()
        .all(|fn_| are_fns_inlined[fn_.index])
}

fn inline(asg: &mut Asg, statements: Vec<AsgStatement>) -> Vec<AsgStatement> {
    statements
        .into_iter()
        .flat_map(|statement| {
            statement
                .split(asg, &mut StatementSplitContext::default())
                .statements(Clone::clone)
        })
        .collect::<Vec<_>>()
        .into_iter()
        .flat_map(|statement| statement.inline(asg))
        .collect()
}
