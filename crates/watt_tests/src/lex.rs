// Imports
#[allow(unused_imports)]
use crate::assert_tokens;

#[test]
fn escape_sequence_1() {
    assert_tokens!(
        r#"
"\n"
"\r"
        "#
    )
}

#[test]
fn escape_sequence_2() {
    assert_tokens!(
        r#"
"\u{00C0}"
"\U{0001F600}"
        "#
    )
}

// note: will report error.
#[test]
fn escape_sequence_3() {
    assert_tokens!(
        r#"
"\u{"
        "#
    )
}

// note: will report error.
#[test]
fn escape_sequence_4() {
    assert_tokens!(
        r#"
"\u{00C1"
        "#
    )
}

// note: will report error.
#[test]
fn escape_sequence_5() {
    assert_tokens!(
        r#"
"\u{00"
        "#
    )
}

// note: will report error.
#[test]
fn escape_sequence_6() {
    assert_tokens!(
        r#"
"\u{00C}"
        "#
    )
}

// note: will report error.
#[test]
fn escape_sequence_7() {
    assert_tokens!(
        r#"
"\U{00C10011}"
        "#
    )
}

// note: will report error.
#[test]
fn escape_sequence_8() {
    assert_tokens!(
        r#"
"\U{приветик}"
        "#
    )
}

#[test]
fn escape_sequence_9() {
    assert_tokens!(
        r#"
"\x{FF}"
        "#
    )
}

#[test]
// note: will report error.
fn escape_sequence_10() {
    assert_tokens!(
        r#"
"\x{0B"
        "#
    )
}

#[test]
// note: will report error.
fn escape_sequence_11() {
    assert_tokens!(
        r#"
"\x{"
        "#
    )
}

#[test]
fn escape_sequence_12() {
    assert_tokens!(
        r#"
"\x{7F}"
        "#
    )
}

#[test]
fn escape_sequence_13() {
    assert_tokens!(
        r#"
"\\n"
"\\x"
"\\u"
"\\t"
"\\r"
"\\U"
        "#
    )
}

#[test]
fn escape_sequence_14() {
    assert_tokens!(
        r#"
"\""
`\``
        "#
    )
}

// note: will report error.
#[test]
fn escape_sequence_15() {
    assert_tokens!(
        r#"
"""
        "#
    )
}

// note: will report error.
#[test]
fn escape_sequence_16() {
    assert_tokens!(
        r#"
```
        "#
    )
}
