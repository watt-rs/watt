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
import {Option} from "../std/option.js"
import * as conv from "../std/convert.js"

export class $Node {
    constructor(value) {
        this.$meta = "Node";
        this.next = Option.None()
        this.value = value
    }
    last() {
        let self = this;
        return $$match(self.next, [
            new $$VariantPattern("None", function() {
                return self
            }),
            new $$UnwrapPattern(["element"], function($$fields) {
                let element = $$fields.element;
                return element.last()
            })
        ])
    }
    insert(value) {
        let self = this;
        self.next = Option.Some(Node(value))
    }
    delete$(value) {
        let self = this;
        return (() => {
            if ($$equals(self.value, value)) {
                return self.next
            }
            else {
                self.next = $$match(self.next, [
                    new $$UnwrapPattern(["element"], function($$fields) {
                        let element = $$fields.element;
                        return (() => {
                            if ($$equals(element.value, value)) {
                                return element.next
                            }
                            else {
                                return element.delete$(value)
                            }
                        })()
                    }),
                    new $$VariantPattern("None", function() {
                        return Option.None()
                    })
                ])
                return Option.Some(self)
            }
        })()
    }
    to_string() {
        let self = this;
        let string = conv.string(self.value)
        return $$match(self.next, [
            new $$VariantPattern("None", function() {
                return string
            }),
            new $$UnwrapPattern(["element"], function($$fields) {
                let element = $$fields.element;
                return string + ", " + conv.string(element.to_string())
            })
        ])
    }
    ;
}
export function Node(value) {
    return new $Node(value);
}

export class $List {
    constructor() {
        this.$meta = "List";
        this.head = Option.None()
    }
    push(value) {
        let self = this;
        $$match(self.head, [
            new $$VariantPattern("None", function() {
                self.head = Option.Some(Node(value))
            }),
            new $$UnwrapPattern(["element"], function($$fields) {
                let element = $$fields.element;
                element.last().insert(value);
            })
        ]);
    }
    delete$(value) {
        let self = this;
        $$match(self.head, [
            new $$UnwrapPattern(["element"], function($$fields) {
                let element = $$fields.element;
                self.head = element.delete$(value)
            })
        ]);
    }
    to_string() {
        let self = this;
        return $$match(self.head, [
            new $$VariantPattern("None", function() {
                return "[]"
            }),
            new $$UnwrapPattern(["element"], function($$fields) {
                let element = $$fields.element;
                return "[" + conv.string(element.to_string()) + "]"
            })
        ])
    }
    ;
}
export function List() {
    return new $List();
}

export function main() {
    let list = List()
    list.push(1);
    list.push("hello");
    list.push(true);
    list.push(false);
    io.println(list.to_string());
    list.delete$("hello");
    io.println(list.to_string());
    list.delete$(false);
    io.println(list.to_string());
    list.delete$(1);
    io.println(list.to_string());
}
