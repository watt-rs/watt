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

export function main() {
    let a = Mammoth(Iceberg(3))
}
