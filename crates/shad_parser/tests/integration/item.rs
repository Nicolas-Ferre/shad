use shad_parser::{BufferItem, Error, Expr, Ident, Item, Literal, LiteralType, Program, Span};

#[test]
fn parse_buffer_item2() {
    let Ok(Program { items }) = Program::parse_str("buf my_buffer = 0.;", "") else {
        panic!("invalid item")
    };
    assert_eq!(
        items,
        [Item::Buffer(BufferItem {
            span: Span::new(0, 19),
            name: Ident {
                span: Span::new(4, 13),
                label: "my_buffer".into()
            },
            value: Expr::Literal(Literal {
                span: Span::new(16, 18),
                value: "0.".into(),
                type_: LiteralType::Float,
            }),
        })]
    );
    assert_eq!(items[0].span(), Span::new(0, 19));
}

#[test]
fn parse_multiple_items() {
    assert_eq!(
        Program::parse_str("buf my_buffer = 0.;\nbuf other = 1.2;", ""),
        Ok(Program {
            items: vec![
                Item::Buffer(BufferItem {
                    span: Span::new(0, 19),
                    name: Ident {
                        span: Span::new(4, 13),
                        label: "my_buffer".into()
                    },
                    value: Expr::Literal(Literal {
                        span: Span::new(16, 18),
                        value: "0.".into(),
                        type_: LiteralType::Float,
                    }),
                }),
                Item::Buffer(BufferItem {
                    span: Span::new(20, 36),
                    name: Ident {
                        span: Span::new(24, 29),
                        label: "other".into()
                    },
                    value: Expr::Literal(Literal {
                        span: Span::new(32, 35),
                        value: "1.2".into(),
                        type_: LiteralType::Float,
                    }),
                })
            ]
        })
    );
}

#[test]
fn parse_invalid_item() {
    let Err(Error::Syntax(err)) = Program::parse_str("var b = 0.;", "file") else {
        panic!("incorrect error")
    };
    assert_eq!(err.offset, 0);
    assert_eq!(err.message, "expected item");
}
