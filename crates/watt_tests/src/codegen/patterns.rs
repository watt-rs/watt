// Imports
#[allow(unused_imports)]
use crate::assert_js;

/*
 * Simple enum match
 */
#[test]
fn match_simple_enum() {
    assert_js!(
        r#"
enum Color {
    Red,
    Green,
    Blue
}

fn describe(c: Color): string {
    match c {
        Color.Red -> "red"
        Color.Green -> "green"
        Color.Blue -> "blue"
    }
}
        "#
    )
}

/*
 * Enum with parameters
 */
#[test]
fn match_enum_with_params() {
    assert_js!(
        r#"
enum Option[T] {
    Some(value: T),
    None
}

fn unwrap[T](opt: Option[T], default: T): T {
    match opt {
        Option.Some(value) -> value
        Option.None -> default
    }
}
        "#
    )
}

/*
 * Match with literal patterns
 */
#[test]
fn match_literals() {
    assert_js!(
        r#"
fn check_number(n: int): string {
    match n {
        0 -> "zero"
        1 -> "one"
        2 -> "two"
        _ -> "many"
    }
}
        "#
    )
}

/*
 * Nested matches
 */
#[test]
fn match_nested() {
    assert_js!(
        r#"
enum Shape {
    Circle(r: float),
    Rectangle(w: float, h: float)
}

fn area(s: Shape): float {
    match s {
        Shape.Circle(r) -> 3.14 * r * r
        Shape.Rectangle(w, h) -> w * h
    }
}
        "#
    )
}

/*
 * Enum variants with default case
 */
#[test]
fn match_enum_with_lost_case_covered_by_default() {
    assert_js!(
        r#"
enum Animal {
    Dog,
    Cat
}

fn test(): int {
    let animal = Animal.Cat();
    match animal {
        Animal.Dog -> 1
        _ -> 2
    }
}
        "#
    )
}

/*
 * Match with boolean patterns
 */
#[test]
fn match_boolean() {
    assert_js!(
        r#"
fn bool_check(b: bool): string {
    match b {
        true -> "yes"
        false -> "no"
    }
}
        "#
    )
}
