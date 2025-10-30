import {
    $$match,
    $$equals,
    $$EqPattern,
    $$UnwrapPattern,
    $$WildcardPattern,
    $$BindPattern,
    $$VariantPattern,
} from "../prelude.js"

import * as internal from "../suit/internal.js"
import {Option} from "../std/option.js"

export class $Test {
    constructor(name, function$) {
        this.$meta = "Type";
        this.$type = "Test";
        this.name = name
        this.function$ = function$
    }
    run() {
        let self = this;
        return internal.try$(self.function$)
    }
    get_function() {
        let self = this;
        return self.function$
    }
}
export function Test(name, function$) {
    return new $Test(name, function$);
}
