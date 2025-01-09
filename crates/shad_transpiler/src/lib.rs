//! Transpiler to convert Shad expressions to WGSL.

use shad_analyzer::{Analysis, ComputeShader};

mod atoms;
mod fn_calls;
mod items;
mod statements;
mod wgsl;

const IDENT_UNIT: usize = 4;

/// Generates a WGSL shader from a Shad shader definition.
///
/// # Errors
///
/// An error is returned if the input shader definition is invalid.
#[allow(clippy::result_unit_err)]
pub fn generate_wgsl_compute_shader(analysis: &Analysis, shader: &ComputeShader) -> String {
    format!(
        "{}\n\n@compute @workgroup_size(1, 1, 1)\nfn main() {{\n{}\n}}\n\n{}\n\n{}",
        items::to_buffer_wgsl(analysis, shader),
        statements::to_wgsl(analysis, &shader.statements),
        items::to_struct_wgsl(analysis, shader),
        items::to_fn_wgsl(analysis, shader),
    )
}
