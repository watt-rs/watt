import {$$match, $$equals} from "../prelude.js"

import * as rt from "../std/rt.js"

export function println(text) {
    console.log(text)
}

export function input() {
    
  if (rt.which() == "Deno") {
    return prompt()
  }
  else {
    return "unimplemented"
  }

}
