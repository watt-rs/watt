import {
    $$match,
    $$equals,
    $$EqPattern,
    $$UnwrapPattern,
    $$WildcardPattern,
    $$BindPattern,
    $$VariantPattern,
} from "../prelude.js"

import {Result} from "../std/result.js"

export const ConvertError = {
    FailedToParseInt: (val) => ({
        $meta: "Enum",
        $enum: "ConvertError",
        $variant: "FailedToParseInt",
        val: val
    }),
    FailedToParseFloat: (val) => ({
        $meta: "Enum",
        $enum: "ConvertError",
        $variant: "FailedToParseFloat",
        val: val
    })
};

export function int(val) {
    
  let int = parseInt(val);
  if (int == NaN) {
    return Result.Err(ConvertError.FailedToParseInt(val));
  }
  return Result.Ok(int);

}

export function float(val) {
    
  let float = parseFloat(val);
  if (float == NaN) {
    return Result.Err(ConvertError.FailedToParseInt(val));
  }
  return Result.Ok(float);

}

export function string(val) {
    return val.toString()
}
