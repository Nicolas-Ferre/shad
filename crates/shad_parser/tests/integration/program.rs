use shad_parser::{BufferItem, Error, Expr, Ident, Item, Literal, LiteralType, Program, Span};

#[test]
fn parse_existing_file() {
    assert_eq!(
        Program::parse_file(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/res/example.shd"
        )),
        Ok(Program {
            items: vec![Item::Buffer(BufferItem {
                span: Span::new(0, 11),
                name: Ident {
                    span: Span::new(4, 5),
                    label: "b".into()
                },
                value: Expr::Literal(Literal {
                    span: Span::new(8, 10),
                    value: "0.".into(),
                    type_: LiteralType::Float,
                }),
            })]
        })
    );
}

#[test]
fn parse_missing_file() {
    assert!(matches!(
        Program::parse_file(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/res/missing.shd"
        )),
        Err(Error::Io(_))
    ));
}
