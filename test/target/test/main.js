import {
    $$match,
    $$equals,
    $$EqPattern,
    $$UnwrapPattern,
    $$WildcardPattern,
    $$BindPattern,
    $$VariantPattern,
} from "../prelude.js"

import * as io from "../std/io.js"
import {List} from "../std/list.js"

export function main() {
    let list = List()
    list.push(1);
    list.push("hello");
    list.push(true);
    list.push(false);
    list.push(444);
    list.delete$(1);
    list.delete$(4);
    io.println(list.to_string());
}
