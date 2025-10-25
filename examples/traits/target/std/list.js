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
import * as conv from "../std/convert.js"
import {Option} from "../std/option.js"

export function list_new() {
    return [];
}

export function list_get(list, idx) {
    return list[idx]
}

export class $List {
    constructor() {
        this.$meta = "Type";
        this.$type = "List";
        this.list = list_new()
    }
    push(val) {
        let self = this;
        return self.list.push(val)
    }
    len() {
        let self = this;
        return self.list.length
    }
    delete$(index) {
        let self = this;
        return (() => {
            if (index > 0 && index < self.len()) {
                self.list.splice(index, 1);
            }
        })()
    }
    unshift(val) {
        let self = this;
        return self.list.unshift(val)
    }
    index_of(val) {
        let self = this;
        return self.list.indexOf(val)
    }
    pop() {
        let self = this;
        return self.list.pop()
    }
    shift() {
        let self = this;
        return self.list.shift()
    }
    contains(value) {
        let self = this;
        return self.list.contains(value)
    }
    get$(index) {
        let self = this;
        return (() => {
            if (index > 0 && index < self.len()) {
                return Option.Some(list_get(self.list, index))
            }
            else {
                return Option.None()
            }
        })()
    }
    copy() {
        let self = this;
        let list = List()
        list.list = self.list.slice()
        return list
    }
    join(by) {
        let self = this;
        return self.list.join(by)
    }
    to_string() {
        let self = this;
        return "[" + self.join(", ") + "]"
    }
    ;
}
export function List() {
    return new $List();
}
