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
    constructor(function$) {
        this.$meta = "Type";
        this.$type = "Test";
        this.name = internal.function_name(function$)
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
export function Test(function$) {
    return new $Test(function$);
}
