use shad_parser::{BufferItem, Error, Expr, Ident, Item, Literal, LiteralType, Program, Span};

#[test]
fn parse_ident() {
    assert_valid_ident("ident");
    assert_valid_ident("_id_ent_");
    assert_valid_ident("_1de4t");
    assert_valid_ident("_");
    assert_invalid_ident("1dent");
}

#[test]
fn parse_float_literal() {
    assert_valid_literal("0.", LiteralType::Float);
    assert_valid_literal("0.0", LiteralType::Float);
    assert_valid_literal("0.01234", LiteralType::Float);
    assert_valid_literal("0123.", LiteralType::Float);
    assert_valid_literal("0_123_.0_1", LiteralType::Float);
    assert_invalid_literal("0123._01", 5);
}

fn assert_valid_ident(ident: &str) {
    assert_eq!(
        Program::parse_str(&format!("buf {ident} = 0.;"), ""),
        Ok(Program {
            items: vec![Item::Buffer(BufferItem {
                span: Span::new(0, 10 + ident.len()),
                name: Ident {
                    span: Span::new(4, 4 + ident.len()),
                    label: ident.into()
                },
                value: Expr::Literal(Literal {
                    span: Span::new(7 + ident.len(), 9 + ident.len()),
                    value: "0.".to_string(),
                    type_: LiteralType::Float,
                }),
            })]
        })
    );
}

fn assert_invalid_ident(ident: &str) {
    let parsed = Program::parse_str(&format!("buf {ident} = 0.;"), "filename");
    let Error::Syntax(err) = parsed.expect_err("valid ident") else {
        panic!("incorrect error")
    };
    assert_eq!(err.offset, 4);
    assert_eq!(err.message, "unexpected token");
    assert!(err.pretty_message.contains("filename:1:5"));
    assert!(err.pretty_message.contains("unexpected token"));
}

fn assert_valid_literal(literal: &str, type_: LiteralType) {
    assert_eq!(
        Program::parse_str(&format!("buf b = {literal};"), ""),
        Ok(Program {
            items: vec![Item::Buffer(BufferItem {
                span: Span::new(0, 9 + literal.len()),
                name: Ident {
                    span: Span::new(4, 5),
                    label: "b".into()
                },
                value: Expr::Literal(Literal {
                    span: Span::new(8, 8 + literal.len()),
                    value: literal.to_string(),
                    type_,
                }),
            })]
        })
    );
}

fn assert_invalid_literal(literal: &str, offset: usize) {
    let parsed = Program::parse_str(&format!("buf b = {literal};"), "filename");
    let Error::Syntax(err) = parsed.expect_err("valid ident") else {
        panic!("incorrect error")
    };
    assert_eq!(err.offset, 8 + offset);
    assert_eq!(err.message, "expected `;`");
    assert!(err
        .pretty_message
        .contains(&format!("filename:1:{}", 8 + offset + 1)));
    assert!(err.pretty_message.contains("expected `;`"));
}
