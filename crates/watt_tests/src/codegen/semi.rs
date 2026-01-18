/// Imports
#[allow(unused_imports)]
use crate::assert_js;

/*
 * For should not require semicolon
 */
#[test]
fn for_should_not_require_semi() {
    assert_js!(
        r#"
fn a(): int {
    for i in 0..100 {

    }
    1 + 1
}

fn main() {}
        "#
    )
}

/*
 * If should not require semicolon
 */
#[test]
fn if_should_not_require_semi() {
    assert_js!(
        r#"
fn a(): int {
    if true {

    } else {

    }
    1 + 1
}

fn main() {}
        "#
    )
}

/*
 * Loop should not require semicolon
 */
#[test]
fn loop_should_not_require_semi() {
    assert_js!(
        r#"
fn a(): int {
    loop true {

    }
    1 + 1
}

fn main() {}
        "#
    )
}
