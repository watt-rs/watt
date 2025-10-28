import {
    $$match,
    $$equals,
    $$EqPattern,
    $$UnwrapPattern,
    $$WildcardPattern,
    $$BindPattern,
    $$VariantPattern,
} from "../prelude.js"

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

export function unit() {
    return;
}

export function meta_of(value) {
    
    if ($meta in value) {
        return value.$meta;
    } else {
        return "Js";
    }

}

export function type_of(value) {
    
    if ($type in value) {
        return value.$type;
    } else {
        return typeof(value);
    }

}

export function variant_of(value) {
    
    if ($enum in value) {
        return value.$variant;
    } else {
        return "NotEnumVariant";
    }

}

export function enum_of(value) {
    
    if ($enum in value) {
        return value.$enum;
    } else {
        return "NotEnum";
    }

}
