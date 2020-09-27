#![cfg(test)]
#![allow(unused_mut, unused_variables, unused_assignments)]

use crate::Lexer;
use quickcheck_macros::quickcheck;

// Assert the result of lexing a piece of source code,
// and make sure the tokens yielded are fully lossless and the source can be reconstructed from only the tokens
macro_rules! assert_lex {
    ($src:expr, $($kind:ident:$len:expr $(,)?)*) => {{
        let mut lexer = Lexer::from_str($src, 0);
        let mut tokens = lexer.collect::<Vec<_>>();
        let mut idx = 0;
        let mut tok_idx = 0;

        let mut new_str = String::with_capacity($src.len());
        // remove eof
        tokens.pop();

        $(
            assert_eq!(
                tokens[idx].0.kind,
                rslint_syntax::SyntaxKind::$kind,
                "expected token kind {}, but found {:?}",
                stringify!($kind),
                tokens[idx].0.kind,
            );

            assert_eq!(
                tokens[idx].0.len,
                $len,
                "expected token length of {}, but found {} for token {:?}",
                $len,
                tokens[idx].0.len,
                tokens[idx].0.kind,
            );

            new_str.push_str($src.get(tok_idx..(tok_idx + tokens[idx].0.len)).unwrap());
            tok_idx += tokens[idx].0.len;

            idx += 1;
        )*

        assert_eq!($src, new_str, "Failed to reconstruct input");
        assert_eq!(idx, tokens.len());
    }};
}

// This is for testing if the lexer is truly lossless
// It parses random strings and puts them back together with the produced tokens and compares
#[quickcheck]
fn losslessness(string: String) -> bool {
    let tokens = Lexer::from_str(&string, 0).map(|x| x.0).collect::<Vec<_>>();
    let mut new_str = String::with_capacity(string.len());
    let mut idx = 0;

    for token in tokens {
        new_str.push_str(string.get(idx..(idx + token.len)).unwrap());
        idx += token.len;
    }

    string == new_str
}

#[test]
fn strip_shebang() {
    let mut lex = Lexer::from_str("#! /bin/node \n\n", 0);
    lex.strip_shebang();
    assert_eq!(lex.cur, 13);
}

#[test]
fn empty() {
    assert_lex! {
        "",
    }
}

#[test]
fn identifier() {
    assert_lex! {
        "Abcdefg",
        IDENT:7
    }
}

#[test]
fn punctuators() {
    assert_lex! {
        "!%%&()*+,-.:;<=>?[]^{}|~",
        BANG:1,
        PERCENT:1,
        PERCENT:1,
        AMP:1,
        L_PAREN:1,
        R_PAREN:1,
        STAR:1,
        PLUS:1,
        COMMA:1,
        MINUS:1,
        DOT:1,
        COLON:1,
        SEMICOLON:1,
        LTEQ:2,
        R_ANGLE:1,
        QUESTION:1,
        L_BRACK:1,
        R_BRACK:1,
        CARET:1,
        L_CURLY:1,
        R_CURLY:1,
        PIPE:1,
        TILDE:1,
    }
}

#[test]
fn consecutive_punctuators() {
    assert_lex! {
        "&&&&^^^||",
        AMP2:2,
        AMP2:2,
        CARET:1,
        CARET:1,
        CARET:1,
        PIPE2:2,
    }
}

#[test]
fn unicode_whitespace() {
    assert_lex! {
        " \u{00a0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200A}\u{202F}\u{205F}\u{3000}",
        WHITESPACE:48
    }
}

#[test]
fn unicode_whitespace_ident_part() {
    assert_lex! {
        "Abcd\u{2006}",
        IDENT:4,
        WHITESPACE:3 // length is in bytes
    }
}

#[test]
fn all_whitespace() {
    assert_lex! {
        "
        ",
        WHITESPACE:9
    }
}

#[test]
fn empty_string() {
    assert_lex! {
        r#""""#,
        STRING:2
    }

    assert_lex! {
        "''",
        STRING:2
    }
}

#[test]
fn template_literals() {
    assert_lex! {
        "`abcdefg` `abc",
        BACKTICK:1,
        TEMPLATE_CHUNK:7,
        BACKTICK:1,
        WHITESPACE:1,
        BACKTICK:1,
        TEMPLATE_CHUNK:3,
    }

    assert_lex! {
        "`${a} a`",
        BACKTICK:1,
        DOLLARCURLY:2,
        IDENT:1,
        R_CURLY:1,
        TEMPLATE_CHUNK:2,
        BACKTICK:1
    }

    assert_lex! {
        "`${a} b ${b}`",
        BACKTICK:1,
        DOLLARCURLY:2,
        IDENT:1,
        R_CURLY:1,
        TEMPLATE_CHUNK:3,
        DOLLARCURLY:2,
        IDENT:1,
        R_CURLY:1,
        BACKTICK:1
    }
}

#[test]
fn simple_string() {
    assert_lex! {
        r#"'abcdefghijklmnopqrstuvwxyz123456789\'10🦀'"#,
        STRING:45
    }

    assert_lex! {
        r#""abcdefghijklmnopqrstuvwxyz123456789\"10🦀""#,
        STRING:45
    }
}

#[test]
fn string_unicode_escape_invalid() {
    assert_lex! {
        r#""abcd\u21""#,
        ERROR_TOKEN:10
    }

    assert_lex! {
        r#"'abcd\u21'"#,
        ERROR_TOKEN:10
    }
}

#[test]
fn string_unicode_escape_valid() {
    assert_lex! {
        r#""abcd\u2000a""#,
        STRING:13
    }

    assert_lex! {
        r#"'abcd\u2000a'"#,
        STRING:13
    }
}

#[test]
fn string_unicode_escape_valid_resolving_to_endquote() {
    assert_lex! {
        r#""abcd\u0022a""#,
        STRING:13
    }

    assert_lex! {
        r#"'abcd\u0027a'"#,
        STRING:13
    }
}

#[test]
fn string_hex_escape_invalid() {
    assert_lex! {
        r#""abcd \xZ0 \xGH""#,
        ERROR_TOKEN:16
    }

    assert_lex! {
        r#"'abcd \xZ0 \xGH'"#,
        ERROR_TOKEN:16
    }
}

#[test]
fn string_hex_escape_valid() {
    assert_lex! {
        r#""abcd \x00 \xAB""#,
        STRING:16
    }

    assert_lex! {
        r#"'abcd \x00 \xAB'"#,
        STRING:16
    }
}

#[test]
fn unterminated_string() {
    assert_lex! {
        r#""abcd"#,
        ERROR_TOKEN:5
    }

    assert_lex! {
        r#"'abcd"#,
        ERROR_TOKEN:5
    }
}

#[test]
fn string_all_escapes() {
    assert_lex! {
        r#""\x\u2004\u20\ux\xNN""#,
        ERROR_TOKEN:21
    }

    assert_lex! {
        r#"'\x\u2004\u20\ux\xNN'"#,
        ERROR_TOKEN:21
    }
}

#[test]
fn complex_string_1() {
    assert_lex! {
        r#" _this += "str'n\u200bg";"#,
        WHITESPACE:1,
        IDENT:5,
        WHITESPACE:1,
        PLUSEQ:2,
        WHITESPACE:1,
        STRING:14,
        SEMICOLON:1
    }

    assert_lex! {
        r#" _this += 'str"n\u200bg';"#,
        WHITESPACE:1,
        IDENT:5,
        WHITESPACE:1,
        PLUSEQ:2,
        WHITESPACE:1,
        STRING:14,
        SEMICOLON:1
    }
}

#[test]
fn unterminated_string_length() {
    assert_lex! {
        "'abc",
        ERROR_TOKEN:4
    }
}

#[test]
fn unterminated_string_with_escape_len() {
    assert_lex! {
        "'abc\\",
        ERROR_TOKEN:5
    }

    assert_lex! {
        r#"'abc\x"#,
        ERROR_TOKEN:6
    }

    assert_lex! {
        r#"'abc\x4"#,
        ERROR_TOKEN:7
    }

    assert_lex! {
        r#"'abc\x45"#,
        ERROR_TOKEN:8
    }

    assert_lex! {
        r#"'abc\u"#,
        ERROR_TOKEN:6
    }

    assert_lex! {
        r#"'abc\u20"#,
        ERROR_TOKEN:8
    }
}

#[test]
fn dollarsign_underscore_idents() {
    assert_lex! {
        "$a",
        IDENT:2
    }
}

#[test]
fn labels_a() {
    assert_lex! {
        "await",
        AWAIT_KW:5
    }

    assert_lex! {
        "awaited",
        IDENT:7
    }
}

#[test]
fn labels_b() {
    assert_lex! {
        "break",
        BREAK_KW:5
    }

    assert_lex! {
        "breaking speed records",
        IDENT:8,
        WHITESPACE:1,
        IDENT:5,
        WHITESPACE:1,
        IDENT:7
    }
}

#[test]
fn labels_c() {
    assert_lex! {
        "continue, const, class, catch, case",
        CONTINUE_KW:8,
        COMMA:1,
        WHITESPACE:1,
        CONST_KW:5,
        COMMA:1,
        WHITESPACE:1,
        CLASS_KW:5,
        COMMA:1,
        WHITESPACE:1,
        CATCH_KW:5,
        COMMA:1,
        WHITESPACE:1,
        CASE_KW:4
    }

    assert_lex! {
        "classy crabs",
        IDENT:6,
        WHITESPACE:1,
        IDENT:5
    }
}

#[test]
fn labels_d() {
    assert_lex! {
        "debugger default delete do",
        DEBUGGER_KW:8,
        WHITESPACE:1,
        DEFAULT_KW:7,
        WHITESPACE:1,
        DELETE_KW:6,
        WHITESPACE:1,
        DO_KW:2
    }

    assert_lex! {
        "derive doot d",
        IDENT:6,
        WHITESPACE:1,
        IDENT:4,
        WHITESPACE:1,
        IDENT:1
    }
}

#[test]
fn labels_e() {
    assert_lex! {
        "else enum export extends",
        ELSE_KW:4,
        WHITESPACE:1,
        ENUM_KW:4,
        WHITESPACE:1,
        EXPORT_KW:6,
        WHITESPACE:1,
        EXTENDS_KW:7
    }

    assert_lex! {
        "e exports elsey",
        IDENT:1,
        WHITESPACE:1,
        IDENT:7,
        WHITESPACE:1,
        IDENT:5
    }
}

#[test]
fn labels_f() {
    assert_lex! {
        "finally for function",
        FINALLY_KW:7,
        WHITESPACE:1,
        FOR_KW:3,
        WHITESPACE:1,
        FUNCTION_KW:8
    }

    assert_lex! {
        "finally, foreign food!",
        FINALLY_KW:7,
        COMMA:1,
        WHITESPACE:1,
        IDENT:7,
        WHITESPACE:1,
        IDENT:4,
        BANG:1
    }
}

#[test]
fn labels_i() {
    assert_lex! {
        "i in instanceof if import",
        IDENT:1,
        WHITESPACE:1,
        IN_KW: 2,
        WHITESPACE:1,
        INSTANCEOF_KW:10,
        WHITESPACE:1,
        IF_KW:2,
        WHITESPACE:1,
        IMPORT_KW:6
    }

    assert_lex! {
        "icecream is interesting, innit?",
        IDENT:8,
        WHITESPACE:1,
        IDENT:2,
        WHITESPACE:1,
        IDENT:11,
        COMMA:1,
        WHITESPACE:1,
        IDENT:5,
        QUESTION:1
    }
}

#[test]
fn labels_n() {
    assert_lex! {
        "new",
        NEW_KW:3
    }

    assert_lex! {
        "newly n",
        IDENT:5,
        WHITESPACE:1,
        IDENT:1
    }
}

#[test]
fn labels_r() {
    assert_lex! {
        "return",
        RETURN_KW:6
    }

    assert_lex! {
        "returning",
        IDENT:9
    }
}

#[test]
fn labels_s() {
    assert_lex! {
        "switch super",
        SWITCH_KW:6,
        WHITESPACE:1,
        SUPER_KW:5
    }

    assert_lex! {
        "superb switching",
        IDENT:6,
        WHITESPACE:1,
        IDENT:9
    }
}

#[test]
fn labels_t() {
    assert_lex! {
        "this try throw typeof t",
        THIS_KW:4,
        WHITESPACE:1,
        TRY_KW:3,
        WHITESPACE:1,
        THROW_KW:5,
        WHITESPACE:1,
        TYPEOF_KW:6,
        WHITESPACE:1,
        IDENT:1
    }

    assert_lex! {
        "thistle throwing tea",
        IDENT:7,
        WHITESPACE:1,
        IDENT:8,
        WHITESPACE:1,
        IDENT:3
    }
}

#[test]
fn labels_v() {
    assert_lex! {
        "var void v",
        VAR_KW:3,
        WHITESPACE:1,
        VOID_KW:4,
        WHITESPACE:1,
        IDENT:1
    }

    assert_lex! {
        "variable voiding is bad",
        IDENT:8,
        WHITESPACE:1,
        IDENT:7,
        WHITESPACE:1,
        IDENT:2,
        WHITESPACE:1,
        IDENT:3
    }
}

#[test]
fn labels_w() {
    assert_lex! {
        "with while w",
        WITH_KW:4,
        WHITESPACE:1,
        WHILE_KW:5,
        WHITESPACE:1,
        IDENT:1
    }

    assert_lex! {
        "whiley withow",
        IDENT:6,
        WHITESPACE:1,
        IDENT:6
    }
}

#[test]
fn labels_y() {
    assert_lex! {
        "yield",
        YIELD_KW:5
    }

    assert_lex! {
        "yielding",
        IDENT:8
    }
}

#[test]
fn number_basic() {
    assert_lex! {
        "1",
        NUMBER:1
    }

    assert_lex! {
        "123456 ",
        NUMBER:6,
        WHITESPACE:1
    }

    assert_lex! {
        "90",
        NUMBER:2
    }

    assert_lex! {
        ".13",
        NUMBER:3
    }
}

#[test]
fn number_basic_err() {
    assert_lex! {
        "2_?",
        ERROR_TOKEN:2,
        QUESTION:1
    }

    assert_lex! {
        r#"25\u0046abcdef"#,
        ERROR_TOKEN:14
    }

    assert_lex! {
        r#"25\uFEFFb"#,
        NUMBER:2,
        ERROR_TOKEN:6,
        IDENT:1
    }

    assert_lex! {
        r#".32\u0046abde"#,
        ERROR_TOKEN:13
    }
}

#[test]
fn number_complex() {
    assert_lex! {
        "3e-5 123e+56",
        NUMBER:4,
        WHITESPACE:1,
        NUMBER:7
    }

    assert_lex! {
        "3.14159e+1",
        NUMBER:10
    }

    assert_lex! {
        ".0e34",
        NUMBER:5
    }
}

#[test]
fn dot_number_disambiguation() {
    assert_lex! {
        ".e+5",
        DOT:1,
        IDENT:1,
        PLUS:1,
        NUMBER:1
    }

    assert_lex! {
        ".0e+5",
        NUMBER:5
    }
}

#[test]
fn binary_literals() {
    assert_lex! {
        "0b10101010, 0B10101010, 0b10101010n",
        NUMBER:10,
        COMMA:1,
        WHITESPACE:1,
        NUMBER:10,
        COMMA:1,
        WHITESPACE:1,
        NUMBER:11
    }
}

#[test]
fn octal_literals() {
    assert_lex! {
        "0o01742242, 0B10101010, 0b10101010n",
        NUMBER:10,
        COMMA:1,
        WHITESPACE:1,
        NUMBER:10,
        COMMA:1,
        WHITESPACE:1,
        NUMBER:11
    }
}

#[test]
fn bigint_literals() {
    assert_lex! {
        "0n 1743642n 1n",
        NUMBER:2,
        WHITESPACE:1,
        NUMBER:8,
        WHITESPACE:1,
        NUMBER:2
    }
}

#[test]
fn single_line_comments() {
    assert_lex! {
        "//abc
    ",
        COMMENT:5,
        WHITESPACE:5
    }

    assert_lex! {
        "//a",
        COMMENT:3
    }
}

#[test]
fn block_comment() {
    assert_lex! {
        "/*
        */",
        COMMENT:13
    }

    assert_lex! {
        "/* */",
        COMMENT:5
    }

    assert_lex! {
        "/* *",
        COMMENT:4
    }
}

#[test]
fn regex() {
    assert_lex! {
        "var a = /aa/gim",
        VAR_KW:3,
        WHITESPACE:1,
        IDENT:1,
        WHITESPACE:1,
        EQ:1,
        WHITESPACE:1,
        REGEX:7
    }
}

#[test]
fn division() {
    assert_lex! {
        "var a = 5 / 6",
        VAR_KW:3,
        WHITESPACE:1,
        IDENT:1,
        WHITESPACE:1,
        EQ:1,
        WHITESPACE:1,
        NUMBER:1,
        WHITESPACE:1,
        SLASH:1,
        WHITESPACE:1,
        NUMBER:1
    }
}

#[test]
fn template_escape() {
    assert_lex! {
        r"`foo \` bar`",
        BACKTICK:1,
        TEMPLATE_CHUNK:10,
        BACKTICK:1
    }
}
