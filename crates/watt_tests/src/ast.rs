// Imports
#[allow(unused_imports)]
use crate::assert_ast;

#[test]
fn logical_or_and() {
    assert_ast!(
        r#"
fn main() {
    if a && b || c && d {
    }
}
        "#
    )
}
