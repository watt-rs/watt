// Imports
#[allow(unused_imports)]
use crate::assert_js;

/*
 * Arithmetic tests
 */
#[test]
fn add_sub_mul_div_mod() {
    assert_js!(
        r#"
fn main() {
    let a = 10;
    let b = 3;
    let c = a + b;
    let d = a - b;
    let e = a * b;
    let f = a / b;
    let g = a % b;
}
        "#
    )
}

/*
 * Casting tests
 */
#[test]
fn cast_int_float() {
    assert_js!(
        r#"
fn main() {
    let a = 10 as float;
}
        "#
    )
}

/*
 * Conditional tests
 */
#[test]
fn simple_if_test() {
    assert_js!(
        r#"
fn check(a: int, b: int): bool {
    if a > b {
        true
    } else {
        false
    }
}
        "#
    )
}

#[test]
fn if_elif_else_test() {
    assert_js!(
        r#"
fn categorize(n: int): string {
    if n > 10 {
        "big"
    } elif n > 5 {
        "medium"
    } else {
        "small"
    }
}
        "#
    )
}

/*
 * Loops tests
 */
#[test]
fn simple_for_loop() {
    assert_js!(
        r#"
fn main() {
    for i in 0..3 {
        let x = i;
    }
}
        "#
    )
}

#[test]
fn simple_for_loop_2() {
    assert_js!(
        r#"
fn main() {
    for i in 0..=3 {
        let x = i;
    }
}
        "#
    )
}

#[test]
fn simple_loop() {
    assert_js!(
        r#"
fn main() {
    let n = 0;
    let flag = true;
    loop flag {
        let x = n;
        let n = n + 1;
        if n == 3 {
            flag = false;
        }
    }
}
        "#
    )
}

/*
 * Boolean expressions
 */
#[test]
fn boolean_logic() {
    assert_js!(
        r#"
fn main() {
    let a = true;
    let b = false;
    let c = a && b;
    let d = a || b;
    let e = !a;
}
        "#
    )
}
