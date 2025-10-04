import {$$match, $$equals, $$EqPattern, $$UnwrapPattern} from "../prelude.js"

export function which() {
    return (typeof Deno!="undefined"?"Deno":typeof Bun!="undefined"?"Bun":typeof process!="undefined"&&process.versions?.node?"Node.js":"Unknown")
}

export function exit(code) {
    
  const runtime = which();
  if (runtime == "Deno") {
    Deno.exit(code);
  } else if (runtime == "Bun") {
    Bun.exit(code);
  } else if (runtime == "Node.js") {
    process.exit(code);
  } else {
    panic("unimplemented");
  }

}

export function panic(text) {
    
  const err = new Error(text);
  throw err;

}
