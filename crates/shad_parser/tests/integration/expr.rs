use shad_parser::{Error, Item, Program, Span};

#[test]
fn parse_literal() {
    let Ok(Program { items }) = Program::parse_str("buf b = 0.;", "") else {
        panic!("invalid item")
    };
    let Item::Buffer(item) = &items[0];
    assert_eq!(item.value.span(), Span::new(8, 10));
}

#[test]
fn parse_invalid_expr() {
    let Err(Error::Syntax(err)) = Program::parse_str("buf b = buf;", "file") else {
        panic!("incorrect error")
    };
    assert_eq!(err.offset, 8);
    assert_eq!(err.message, "expected expression");
}
