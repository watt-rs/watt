// Imports
#[allow(unused_imports)]
use crate::assert_js;

#[test]
fn simple_struct() {
    assert_js!(
        r#"
type House {
    street: string,
    number: int,
    owner_id: int
}
    "#
    )
}

#[test]
fn structs_with_generics() {
    assert_js!(
        r#"
type Mammoth[T] {
    value: Iceberg[T]
}

type Iceberg[T] {
    value: T
}

fn main() {
    let a = Mammoth(Iceberg(3))
}
    "#
    )
}

#[test]
fn structs_with_generics_2() {
    assert_js!(
        r#"
type Mammoth[T] {
    value: Iceberg[T]
}

type Iceberg[T] {
    value: T
}

fn main() {
    let a: Mammoth[int] = Mammoth(Iceberg(3))
}
    "#
    )
}

// note: will report error.
#[test]
fn wrong_struct_with_generics() {
    assert_js!(
        r#"
type Box[T] {
    value: T
}

fn main() {
    let a = Box(123);
    a = Box("hello");
}
    "#
    )
}

// note: will report error.
#[test]
fn wrong_struct_with_generics_2() {
    assert_js!(
        r#"
type Box[T] {
    value: T
}

fn main() {
    let a = Box(123);
    let b: Box[float] = a;
}
    "#
    )
}

// note: will report error.
#[test]
fn struct_types_missmatch() {
    assert_js!(
        r#"
type A {
    value: int
}

type B {
    value: int
}

fn main() {
    let a = A(3);
    a = B(4);
}
    "#
    )
}
