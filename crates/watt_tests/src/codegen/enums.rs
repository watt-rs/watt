// Imports
#[allow(unused_imports)]
use crate::assert_js;

#[test]
fn simple_enum() {
    assert_js!(
        r#"
enum Season {
    Winter,
    Spring,
    Summer,
    Autumn
}
    "#
    )
}

#[test]
fn enum_with_fields() {
    assert_js!(
        r#"
enum Color {
    Rgb(r: int, g: int, b: int),
    Hex(hex: string),
    Cmyk(c: int, m: int, y: int, k: int)
}
    "#
    )
}

#[test]
fn enum_with_generics() {
    assert_js!(
        r#"
enum Result[V, E] {
    Ok(value: V),
    Err(error: E)
}

fn main() {
    let a = Result.Ok(200);
    a = Result.Err(false);
    let b: Result[int, bool] = a;
}
    "#
    )
}

#[test]
fn enum_with_generics_2() {
    assert_js!(
        r#"
enum Result[V, E] {
    Ok(value: V),
    Err(error: E)
}

fn main() {
    let a: Result[int, bool] = Result.Ok(200);
    a = Result.Err(false);
}
    "#
    )
}

// note: will report error.
#[test]
fn wrong_enum_with_generics() {
    assert_js!(
        r#"
enum Result[V, E] {
    Ok(value: V),
    Err(error: E)
}

fn main() {
    let a = Result.Ok(200);
    a = Result.Err(false);
    let b: Result[float, bool] = a;
}
    "#
    )
}

// note: will report error.
#[test]
fn wrong_enum_with_generics_2() {
    assert_js!(
        r#"
enum Result[V, E] {
    Ok(value: V),
    Err(error: E)
}

fn main() {
    let a = Result.Ok(200);
    let b: Result[float, bool] = a;
}
    "#
    )
}

// note: will report error.
#[test]
fn recursive_enums() {
    assert_js!(
        r#"
enum Option[T] {
    Some(value: T),
    None
}

fn main() {
    let a = Option.None();
    a = Option.Some(a);
}
    "#
    )
}
