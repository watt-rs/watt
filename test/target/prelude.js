function $enum_equals(a, b) {

    let a_keys = Object.keys(a);
    let b_keys = Object.keys(b);

    if (a_keys.length != b_keys.length) {
        return false
    }

    for (const k1 of a_keys) {

        if (b_keys.includes(k1)) {

            if ($equals(a[k1], b[k1]) == false) {
                return false
            }
        }

        else {
            return false;
        }
    };
    return true;
}

export function $equals(a, b) {

    if (typeof(a) !== "object" || typeof(b) !== "object") {
        return a == b;
    }

    else {

        if ("$meta" in a) {
            if ("$meta" in b) {

                let a_meta = a.$meta;
                let b_meta = b.$meta;

                if (a_meta != b_meta) {
                    return false
                } else {

                    let meta = a_meta;

                    if (meta == "Enum") {

                        return $enum_equals(a, b)
                    }
                    return a === b;
                }
            }
        } else {
            return a == b;
        }
    }
}

export function $match() {

}
