import {$$match, $$equals, $$EqPattern, $$UnwrapPattern} from "../prelude.js"

import * as io from "../std/io.js"
import {Option} from "../std/option.js"
import * as conv from "../std/convert.js"

export class $Node {
    constructor(value) {
        this.$meta = "Node";
        this.next = Option.None();
        this.value = value;
    }
    last() {
        let self = this;
        let $$match_result = $$match(self.next, [
            new $$EqPattern(Option.None(), function() {
                return self;
            }),
            new $$UnwrapPattern(["element"], function($$fields) {
                let element = $$fields.element;
                return element.last();
            })
        ]);
        if ($$match_result != null && $$match_result != undefined) {
            return $$match_result
        }
    }insert(value) {
        let self = this;
        self.next = Option.Some(Node(value));
    }delete$(value) {
        let self = this;
        if ($$equals(self.value, value)) {
            return self.next;
        }
        let $$match_result = $$match(self.next, [
            new $$UnwrapPattern(["element"], function($$fields) {
                let element = $$fields.element;
                if ($$equals(element.value, value)) {
                    self.next = element.next;
                }
                else if (true) {
                    self.next = element.delete$(value);
                }
            })
        ]);
        if ($$match_result != null && $$match_result != undefined) {
            return $$match_result
        }
        return Option.Some(self);
    }to_string() {
        let self = this;
        let string = conv.string(self.value);
        let $$match_result = $$match(self.next, [
            new $$EqPattern(Option.None(), function() {
                return string;
            }),
            new $$UnwrapPattern(["element"], function($$fields) {
                let element = $$fields.element;
                return string + ", " + conv.string(element.to_string());
            })
        ]);
        if ($$match_result != null && $$match_result != undefined) {
            return $$match_result
        }
    }
}
export function Node(next, value) {
    return new $Node(next, value);
}

export class $List {
    constructor() {
        this.$meta = "List";
        this.head = Option.None();
    }
    push(value) {
        let self = this;
        let $$match_result = $$match(self.head, [
            new $$EqPattern(Option.None(), function() {
                self.head = Option.Some(Node(value));
            }),
            new $$UnwrapPattern(["element"], function($$fields) {
                let element = $$fields.element;
                element.last().insert(value);
            })
        ]);
        if ($$match_result != null && $$match_result != undefined) {
            return $$match_result
        }
    }delete$(value) {
        let self = this;
        let $$match_result = $$match(self.head, [
            new $$EqPattern(Option.None(), function() {
                return;
            }),
            new $$UnwrapPattern(["element"], function($$fields) {
                let element = $$fields.element;
                self.head = element.delete$(value);
            })
        ]);
        if ($$match_result != null && $$match_result != undefined) {
            return $$match_result
        }
    }to_string() {
        let self = this;
        let $$match_result = $$match(self.head, [
            new $$EqPattern(Option.None(), function() {
                return "[]";
            }),
            new $$UnwrapPattern(["element"], function($$fields) {
                let element = $$fields.element;
                return "[" + conv.string(element.to_string()) + "]";
            })
        ]);
        if ($$match_result != null && $$match_result != undefined) {
            return $$match_result
        }
    }
}
export function List(head) {
    return new $List(head);
}

export function main() {
    let list = List();
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
