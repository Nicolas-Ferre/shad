pub(crate) fn resolve_placeholders(
    code_template: impl Into<String>,
    params: impl Iterator<Item = impl AsRef<str>>,
    args: impl Iterator<Item = impl AsRef<str>>,
) -> String {
    let mut transpilation = code_template.into();
    for (arg, param) in args.zip(params) {
        let placeholder = format!("${{{}}}", param.as_ref());
        transpilation = transpilation.replace(&placeholder, arg.as_ref());
    }
    transpilation
}
