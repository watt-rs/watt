// Imports
#[allow(unused_imports)]
use crate::assert_js;

// note: will report error.
#[test]
fn wrong_argument() {
    assert_js!(
        r#"
fn sum(a: int, b: int): int {
    a + b
}

fn main() {
    sum(3, 4.5)
}
    "#
    )
}
