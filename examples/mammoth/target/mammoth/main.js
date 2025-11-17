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
    constructor(iceberg) {
        this.$meta = "Type";
        this.$type = "Mammoth";
        this.iceberg = iceberg
    }
}
export function Mammoth(iceberg) {
    return new $Mammoth(iceberg);
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
    let iceberge = Iceberg(3)
}
