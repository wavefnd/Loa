#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Indent,
    Dedent,

    Fun,
    If,
    Else,
    While,
    For,
    Import,
    Return,
    Continue,
    Break,
    Input,
    Print,
    Println,

    LogicalAnd,    // &&
    LogicalOr,     // ||
    NotEqual,      // !=
    Not,           // !
    Xor,           // ^

    Operator(String),

    Identifier(String),
    String(String),
    Number(i64),
    Float(f64),

    Plus,          // +
    Minus,         // -
    Star,          // *
    Div,           // /
    Equal,         // =
    EqualTwo,      // ==
    Comma,         // ,
    Dot,           // .
    SemiColon,     // ;
    Colon,         // :
    Lchevr,        // <
    LchevrEq,      // <=
    Rchevr,        // >
    RchevrEq,      // >=
    Lparen,        // (
    Rparen,        // )
    Lbrack,        // [
    Rbrack,        // ]

    Eof,
    Error,
    Whitespace,
}