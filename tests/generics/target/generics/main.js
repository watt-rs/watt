import {
    $$match,
    $$equals,
    $$todo,
    $$range,
    $$EqPattern,
    $$UnwrapPattern,
    $$WildcardPattern,
    $$BindPattern,
    $$VariantPattern,
} from "../prelude.js"

export class $Mammoth {
    constructor(value) {
        this.$meta = "Type";
        this.$type = "Mammoth";
        this.value = value
    }
}
export function Mammoth(value) {
    return new $Mammoth(value);
}

export class $Iceberg {
    constructor(value) {
        this.$meta = "Type";
        this.$type = "Iceberg";
        this.value = value
    }
}
export function Iceberg(value) {
    return new $Iceberg(value);
}

export function test1() {
    let a = Mammoth(Iceberg(3))
}

export class $Apply {
    constructor(function$) {
        this.$meta = "Type";
        this.$type = "Apply";
        this.function$ = function$
    }
}
export function Apply(function$) {
    return new $Apply(function$);
}

export function test2() {
    let a = Apply(function (a) {
        return a
    })
}

export function a(a) {
    return a
}

export function test3(v) {
    let annotated = a
    return annotated(v)
}

export function b(a) {
    return a
}

export function test4() {
    b("hello");
    b(123);
}

export function c(a) {
    return a
}

export function test5() {
    let a = c
    a(3);
}

export const Result = {
    Ok: (value) => ({
        $meta: "Enum",
        $enum: "Result",
        $variant: "Ok",
        value: value
    }),
    Error: (error) => ({
        $meta: "Enum",
        $enum: "Result",
        $variant: "Error",
        error: error
    })
};

export function test6() {
    let a = Result.Ok(3)
    a = Result.Error(false)
    let b = a
}
