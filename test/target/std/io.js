import {$$match, $$equals} from "../prelude.js"

import * as rt from "../std/rt.js"

export function println(text) {
    console.log(text)
}

export function readln() {
    
  let runtime = rt.which();
  if (runtime == "Deno" ||
      runtime == "Bun") {
    return prompt("");
  }
  else {
    return "unimplemented";
  }

}

export function ask(text) {
    
  let runtime = rt.which();
  if (runtime == "Deno" ||
      runtime == "Bun") {
    return prompt(text);
  }
  else {
    return "unimplemented";
  }

}
