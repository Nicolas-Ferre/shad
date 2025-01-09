// based on https://www.w3.org/TR/WGSL/#identifiers
pub(crate) fn is_ident_name_accepted(ident: &str) -> bool {
    ident != "_"
        && !ident.starts_with("__")
        && !wgpu::naga::keywords::wgsl::RESERVED.contains(&ident)
}
