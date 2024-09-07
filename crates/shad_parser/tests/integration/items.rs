use shad_parser::{BufferItem, Expr, Ident, Item, Literal, LiteralType, Program, Span};

#[test]
fn parse_buffer_item() {
    assert_eq!(
        Program::parse_str("buf my_buffer = 0.;", ""),
        Ok(Program {
            items: vec![Item::Buffer(BufferItem {
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
        })
    );
}
