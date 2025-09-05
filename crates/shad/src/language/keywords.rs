use crate::compilation::node::keyword;

pub(crate) const RESERVED_KEYWORDS: &[&str] = &[
    "buf", "false", "fn", "import", "init", "native", "run", "return", "var", "true",
];

keyword!(BufKeyword, "buf");
keyword!(FalseKeyword, "false");
keyword!(FnKeyword, "fn");
keyword!(ImportKeyword, "import");
keyword!(InitKeyword, "init");
keyword!(NativeKeyword, "native");
keyword!(RefKeyword, "ref");
keyword!(RunKeyword, "run");
keyword!(ReturnKeyword, "return");
keyword!(VarKeyword, "var");
keyword!(TrueKeyword, "true");

keyword!(AndSymbol, "&&");
keyword!(ArrowSymbol, "->");
keyword!(CommaSymbol, ",");
keyword!(CloseAngleBracketSymbol, ">");
keyword!(CloseCurlyBracketSymbol, "}");
keyword!(CloseParenthesisSymbol, ")");
keyword!(ColonSymbol, ":");
keyword!(DoubleEqSymbol, "==");
keyword!(DotSymbol, ".");
keyword!(EqSymbol, "=");
keyword!(ExclamationSymbol, "!");
keyword!(GreaterEqSymbol, ">=");
keyword!(HyphenSymbol, "-");
keyword!(LessEqSymbol, "<=");
keyword!(NotEqSymbol, "!=");
keyword!(OpenAngleBracketSymbol, "<");
keyword!(OpenCurlyBracketSymbol, "{");
keyword!(OpenParenthesisSymbol, "(");
keyword!(OrSymbol, "||");
keyword!(PercentSymbol, "%");
keyword!(PlusSymbol, "+");
keyword!(SlashSymbol, "/");
keyword!(StarSymbol, "*");
keyword!(SemicolonSymbol, ";");
keyword!(TildeSymbol, "~");
