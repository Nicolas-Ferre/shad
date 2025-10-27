use crate::compilation::node::keyword;

pub(crate) const RESERVED_KEYWORDS: &[&str] = &[
    "buf", "const", "false", "fn", "import", "init", "native", "pub", "ref", "run", "return",
    "struct", "var", "true",
];

keyword!(BufKeyword, "buf");
keyword!(ConstKeyword, "const");
keyword!(FalseKeyword, "false");
keyword!(FnKeyword, "fn");
keyword!(ImportKeyword, "import");
keyword!(InitKeyword, "init");
keyword!(NativeKeyword, "native");
keyword!(PubKeyword, "pub");
keyword!(RefKeyword, "ref");
keyword!(RunKeyword, "run");
keyword!(ReturnKeyword, "return");
keyword!(StructKeyword, "struct");
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
