import {
    $$match,
    $$equals,
    $$EqPattern,
    $$UnwrapPattern,
    $$WildcardPattern,
    $$BindPattern,
    $$VariantPattern,
} from "../prelude.js"

import {Option} from "../std/option.js"

export function try$(function$) {
    
    try {
        function$();
        return Option.None();
    } catch (err) {
        return Option.Some(err);
    }

}
