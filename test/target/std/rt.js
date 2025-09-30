import {$match, $equals} from "../prelude.js"

export function which() {
    return (typeof Deno!="undefined"?"Deno":typeof Bun!="undefined"?"Bun":typeof process!="undefined"&&process.versions?.node?"Node.js":"Unknown")
}
