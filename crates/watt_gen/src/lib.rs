/// Imports
use ecow::EcoString;
use genco::{lang::js, quote, quote_in, tokens::quoted};
use watt_ast::ast::{Block, Declaration, Expression, Module, Pattern, Statement, UseKind};

/// Replaces js identifiers equal
/// to some js keywords with `{indentifier}$`
pub fn try_escape_js(identifier: &str) -> String {
    if matches!(
        identifier,
        // Keywords and reserved words
        // Info can be found here:
        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar
        "await"
            | "arguments"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "eval"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "implements"
            | "import"
            | "in"
            | "instanceof"
            | "interface"
            | "let"
            | "new"
            | "null"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "return"
            | "static"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
            | "undefined"
            | "constructor"
            | "prototype"
            | "__proto__"
            | "async"
            | "from"
            | "of"
            | "set"
            | "get"
            | "as"
    ) {
        format!("{identifier}$")
    } else {
        identifier.to_owned()
    }
}

/// Generates expression code
pub fn gen_expression(expr: Expression) -> js::Tokens {
    /*
    match expr {
        Expression::Float { location: _, value } => quote! ( $(value.to_string()) ),
        Expression::Int { location: _, value } => quote! ( $(value.to_string()) ),
        Expression::String { location: _, value } => quote! ( $(quoted(value.as_str())) ),
        Expression::Bool { location: _, value } => quote! ( $(value.as_str()) ),
        Expression::Bin {
            location: _,
            left,
            right,
            op,
        } => match op.as_str() {
            // With string values
            "<>" => quote!( $(gen_expression(*left)) + $(gen_expression(*right)) ),
            // With number values
            "+" => quote!( $(gen_expression(*left)) + $(gen_expression(*right)) ),
            "-" => quote!( $(gen_expression(*left)) - $(gen_expression(*right)) ),
            "*" => quote!( $(gen_expression(*left)) * $(gen_expression(*right)) ),
            "/" => quote!( $(gen_expression(*left)) / $(gen_expression(*right)) ),
            "^" => quote!( $(gen_expression(*left)) ^ $(gen_expression(*right)) ),
            "&" => {
                quote!( $(gen_expression(*left)) & $(gen_expression(*right)) )
            }
            "|" => quote!( $(gen_expression(*left)) | $(gen_expression(*right)) ),
            "%" => quote!( $(gen_expression(*left)) % $(gen_expression(*right)) ),
            ">" => quote!( $(gen_expression(*left)) > $(gen_expression(*right)) ),
            "<" => quote!( $(gen_expression(*left)) < $(gen_expression(*right)) ),
            ">=" => quote!( $(gen_expression(*left)) >= $(gen_expression(*right)) ),
            "<=" => quote!( $(gen_expression(*left)) <= $(gen_expression(*right)) ),
            // With bool
            "||" => quote!( $(gen_expression(*left)) || $(gen_expression(*right)) ),
            "&&" => quote!( $(gen_expression(*left)) && $(gen_expression(*right)) ),
            "==" => {
                quote!( $("$$equals")($(gen_expression(*left)), $(gen_expression(*right))) )
            }
            "!=" => {
                quote!( !$("$$equals")($(gen_expression(*left)), $(gen_expression(*right))) )
            }
            _ => unreachable!(),
        },
        Expression::Unary { value, op, .. } => match op.as_str() {
            "-" => quote!( -$(gen_expression(*value)) ),
            "!" => quote!( !$(gen_expression(*value)) ),
            _ => unreachable!(),
        },
        Expression::PrefixVar { name, .. } => quote!($(try_escape_js(&name))),
        Expression::SuffixVar {
            location: _,
            container,
            name,
        } => quote!($(gen_expression(*container)).$(try_escape_js(&name))),
        Expression::Call {
            location: _,
            what,
            args,
        } => quote! {
            $(gen_expression(*what))($(for arg in args join (, ) => $(gen_expression(arg))))
        },
        Expression::Function { params, body, .. } => {
            // function ($param, $param, n...)
            quote! {
                function ($(for param in params join (, ) => $(param.name.to_string()))) {
                    $(gen_block(body))
                }
            }
        }
        Expression::Match {
            location: _,
            value,
            cases,
        } => {
            quote! {
                $("$$match")($(gen_expression(*value)), [
                    $['\r']
                    $(for case in cases join (,$['\r']) {
                        $(match case.pattern {
                            // Value pattern / eq pattern
                            Pattern::Int(val) | Pattern::Float(val) |
                            Pattern::String(val) | Pattern::Bool(val)  => {
                                new $("$$")EqPattern($(gen_expression(val)), function() {
                                    $(gen_block(case.body))
                                })
                            },
                            // Unwrap pattern of fields {field, field, n..}
                            Pattern::Unwrap { en: _, fields } => {
                                new $("$$")UnwrapPattern([$(for field in fields.clone() join (, ) => $(quoted(field.1.as_str())))], function($("$$fields")) {
                                    $(for field in fields => let $(field.1.as_str()) = $("$$fields").$(field.1.as_str());$['\r'])
                                    $(gen_block(case.body))
                                })
                            },
                            // Default pattern
                            Pattern::Default => {
                                new $("$$")DefPattern(function() {
                                    $(gen_block(case.body))
                                })
                            }
                        })
                    })
                    $['\r']
                ]);
            }
        }
        Expression::If {
            location,
            logical,
            body,
            else_branches,
        } => todo!(),
    }
    */
    quote!(todo)
}

/// Generates statement
pub fn gen_statement(stmt: Statement) -> js::Tokens {
    /*
    match stmt {
        // If statement
        IrStatement::If {
            logical,
            body,
            else_branches,
            ..
        } => {
            // Generating base if
            let mut base = quote! {
                if ($(gen_expression(logical))) {
                    $(gen_block(body))
                }
                $['\r']
            };
            // Generating else branches
            for branch in else_branches {
                match branch {
                    IrElseBranch::Elif { logical, body, .. } => {
                        // Quoting else if branch
                        quote_in! { base =>
                            else if ($(gen_expression(logical))) {
                                $(gen_block(body))
                            }
                            $['\r']
                        }
                    }
                    IrElseBranch::Else { body, .. } => {
                        // Quoting else else branch
                        quote_in! { base =>
                            else {
                                $(gen_block(body))
                            }
                            $['\r']
                        }
                    }
                }
            }
            base
        }
        // While statement
        IrStatement::While { logical, body, .. } => quote! {
            while ($(gen_expression(logical))) {
                $(gen_block(body))
            }
        },
        // Define statement
        IrStatement::Define { name, value, .. } => quote! {
            let $(try_escape_js(&name)) = $(gen_expression(value));
        },
        // Assing statement
        IrStatement::Assign { what, value, .. } => quote! {
            $(gen_expression(what)) = $(gen_expression(value));
        },
        // Call statement
        IrStatement::Call { what, args, .. } => quote! {
            $(gen_expression(what))($(for arg in args join (, ) => $(gen_expression(arg))));
        },
        // Function statement
        IrStatement::Fn {
            name, params, body, ..
        } => quote! {
            // function $name($param, $param, n...)
            function $(try_escape_js(&name))($(for param in params join (, ) => $(param.name.to_string()))) {
                $(gen_block(body))
            }
        },
        // Break statement
        IrStatement::Break { .. } => quote!(break;),
        // Continue statement
        IrStatement::Continue { .. } => quote!(continue;),
        // Return statement
        IrStatement::Return { value, .. } => match value {
            Some(value) => quote!(return $(gen_expression(value));),
            None => quote!(return;),
        },
        // Match statement
        IrStatement::Match {
            location: _,
            value,
            cases,
        } => {
            // Pattern matching
            quote! {
                let $("$$match_result") = $("$$match")($(gen_expression(value)), [
                    $['\r']
                    $(for case in cases join (,$['\r']) {
                        $(match case.pattern {
                            // Value pattern / eq pattern
                            IrPattern::Value(val) => {
                                new $("$$")EqPattern($(gen_expression(val)), function() {
                                    $(gen_block(case.body))
                                })
                            },
                            // Unwrap pattern of fields {field, field, n..}
                            IrPattern::Unwrap { en: _, fields } => {
                                new $("$$")UnwrapPattern([$(for field in fields.clone() join (, ) => $(quoted(field.as_str())))], function($("$$fields")) {
                                    $(for field in fields => let $(field.clone().to_string()) = $("$$fields").$(field.as_str());$['\r'])
                                    $(gen_block(case.body))
                                })
                            },
                            // Range pattern
                            IrPattern::Range { .. } => todo!(),
                            // Default pattern
                            IrPattern::Default => {
                                new $("$$")DefPattern(function() {
                                    $(gen_block(case.body))
                                })
                            }
                        })
                    })
                    $['\r']
                ]);
                if ($("$$match_result") != null && $("$$match_result") != undefined) {
                    return $("$$match_result")
                }
            }
        }
        // For statement
        IrStatement::For { .. } => todo!(),
    }
    */
    quote!(todo)
}

/// Generates declaration
pub fn gen_declaration(decl: Declaration) -> js::Tokens {
    /*
    match decl {
        IrDeclaration::Function(ir_function) => {
            // function $name($param, $param, n...)
            quote! {
                export function $(try_escape_js(&ir_function.name))($(for param in ir_function.params join (, ) => $(param.name.to_string()))) {
                    $(gen_block(ir_function.body))
                }
            }
        }
        IrDeclaration::Variable(ir_variable) => quote! {
            export let $(try_escape_js(&ir_variable.name)) = $(gen_expression(ir_variable.value));
        },
        IrDeclaration::Type(ir_type) => {
            // Methods
            let methods = quote! {
                $(for function in ir_type.functions =>
                    $(try_escape_js(&function.name))($(for param in function.params join (, ) => $(param.name.to_string()))) {
                        let self = this;
                        $(gen_block(function.body))
                    }
                )
            };

            // constructor($field, $field, n...)
            // with meta type field as `type_name`
            let constructor = quote! {
                constructor($(for field in ir_type.constructor.clone() join (, ) => $(try_escape_js(&field.name)))) {
                    this.$("$")meta = $(quoted(ir_type.name.to_string()));
                    $(for field in ir_type.fields.clone() join ($['\r']) => this.$(try_escape_js(&field.name)) = $(gen_expression(field.value));)
                }
            };

            // Class of `Type` named as $type_name
            // and class fabric named as `type_name`
            quote! {
                export class $("$")$(try_escape_js(&ir_type.name)) {
                    $constructor
                    $methods
                }
                export function $(try_escape_js(&ir_type.name))($(for field in ir_type.fields.clone() join (, ) => $(field.name.to_string()))) {
                    return new $("$")$(try_escape_js(&ir_type.name))($(for field in ir_type.fields join (, ) => $(field.name.to_string())));
                }
            }
        }
        IrDeclaration::Enum(ir_enum) => {
            // ($variant_name): ($param, $param, n...): ({
            //    $meta: "Enum"
            //    $enum: $name
            //    $param: $param,
            //    $param: $param,
            //    n...
            // })
            let variants: js::Tokens = quote!($(for variant in ir_enum.variants join(,$['\r']) =>
                $(variant.name.to_string()): ($(for param in variant.params.clone() join (, ) => $(param.name.to_string()))) => ({
                    $("$")meta: "Enum",
                    $("$")enum: $(quoted(ir_enum.name.to_string())),
                    $(for param in variant.params.clone() join (, ) => $(param.name.to_string()): $(param.name.to_string()))
                })
            ));

            // constr $name = {}
            quote! {
                export const $(try_escape_js(&ir_enum.name)) = {
                    $variants
                };
            }
        }
        IrDeclaration::Extern(ir_extern) => {
            quote! {
                export function $(try_escape_js(&ir_extern.name))($(for param in ir_extern.params join (, ) => $(param.name.to_string()))) {
                    $(ir_extern.body.to_string())
                }
            }
        }
    }
    */
    quote!(todo)
}

/// Generates block
pub fn gen_block(block: Block) -> js::Tokens {
    /*
    // Block of statement
    quote! {
        $(for stmt in block.statements join ($['\r']) => $(gen_statement(stmt)))
    }
    */
    quote!(todo)
}

/// Generates module
pub fn gen_module(name: &EcoString, module: &Module) -> js::Tokens {
    // Segments amount for dependencies
    let name_segments_amount = name.split("/").count();
    // Dependencies prefix
    let dependencies_prefix = match name_segments_amount {
        1 => String::from("./"),
        _ => "../".repeat(name_segments_amount - 1),
    };
    // Gen
    quote! {
        // Prelude
        import {$("$$match"), $("$$equals"), $("$$EqPattern"), $("$$UnwrapPattern"), $("$$DefPattern")} from $(quoted(format!("{dependencies_prefix}prelude.js")))
        // Dependencies
        //
        // for `AsName`: import * as $name from "$module"
        // for `ForNames`: import {$name, $name, ...} from "$module"
        $(for dep in module.dependencies.clone() join ($['\r']) => $(match dep.kind {
            UseKind::AsName(name) => {
                import * as $(name.to_string()) from $(quoted(format!("{dependencies_prefix}{}.js", dep.path.module.as_str())))
            },
            UseKind::ForNames(names) => {
                import {$(for name in names join(, ) => $(name.to_string()))} from $(quoted(format!("{dependencies_prefix}{}.js", dep.path.module.as_str())))
            },
        }))
        $['\n']
        // Declarations
        $(for decl in module.declarations.clone() join ($['\n']) => $(gen_declaration(decl)))
    }
}

/// Generates prelude
pub fn gen_prelude() -> js::Tokens {
    quote! {
        // EnumEquals$fn
        function $("$$enum_equals")(a, b) {
            // Gettting keys
            let a_keys = Object.keys(a);
            let b_keys = Object.keys(b);
            // Checking length
            if (a_keys.length != b_keys.length) {
                return false;
            }
            // Checking entries
            for (const k1 of a_keys) {
                // If b keys includes a key
                if (b_keys.includes(k1)) {
                    // Comparing values
                    if ($("$$")equals(a[k1], b[k1]) == false) {
                        return false;
                    }
                }
                // Otherwise
                else {
                    return false;
                }
            };
            return true;
        }

        // Equals$Fn
        export function $("$$equals")(a, b) {
            // If both not objects
            if (typeof(a) !== "object" || typeof(b) !== "object") {
                return a == b;
            }
            // Else
            else {
                // If meta is $Type or other
                if ("$meta" in a) {
                    if ("$meta" in b) {
                        // Getting meta, if it exists
                        let a_meta = a.$("$meta");
                        let b_meta = b.$("$meta");
                        // If meta is different
                        if (a_meta != b_meta) {
                            return false;
                        } else {
                            // Meta
                            let meta = a_meta;
                            // If meta is $Enum
                            if (meta == "Enum") {
                                // Comparing enums
                                return $("$$")enum_equals(a, b);
                            }
                            return a === b;
                        }
                    }
                } else {
                    return a == b;
                }
            }
        }

        // UnwrapPattern$Class
        export class $("$$UnwrapPattern") {
            constructor(fields, unwrap_fn) {
                this.fields = fields;
                this.unwrap_fn = unwrap_fn;
            }
            evaluate(value) {
                // Checking meta existence
                if ("$meta" in value) {
                    // Meta
                    let meta = value.$("$meta");
                    // Checking it's an enum
                    if (meta == "Enum") {
                        // Retrieving keys
                        let keys = Object.keys(value);
                        // Checking for fields
                        for (const field of this.fields) {
                            // If keys isn't includes a field
                            if (!keys.includes(field)) {
                                return [false, null];
                            }
                        };
                        // Unwrap
                        return [true, this.unwrap_fn(value)];
                    } else {
                        return [false, null];
                    }
                } else {
                    return [false, null];
                }
            }
        }

        // EqPattern$Class
        export class $("$$EqPattern") {
            constructor(value, eq_fn) {
                this.value = value;
                this.eq_fn = eq_fn;
            }
            evaluate(value) {
                if ($("$$equals")(this.value, value)) {
                    return [true, this.eq_fn()];
                } else {
                    return [false, null];
                }
            }
        }

        // DefPattern$Class
        export class $("$$DefPattern") {
            constructor(eq_fn) {
                this.eq_fn = eq_fn;
            }
            evaluate(value) {
                return [true, this.eq_fn()];
            }
        }

        // Match$Fn
        export function $("$$match")(value, patterns) {
            for (const pat of patterns) {
                let result = pat.evaluate(value);
                if (result[0] == true) {
                    return result[1]
                }
            }
            return null;
        }
    }
}

/// Generates index file
pub fn gen_index(main_module: String) -> js::Tokens {
    quote! {
        import { main } from $(quoted(format!("./{main_module}.js")))
        main();
    }
}
