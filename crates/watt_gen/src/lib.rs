/// Imports
use ecow::EcoString;
use genco::{lang::js, quote, tokens::quoted};
use watt_ast::ast::{
    BinaryOp, Block, ConstDeclaration, Declaration, Either, ElseBranch, Expression, FnDeclaration,
    Module, Pattern, Range, Statement, TypeDeclaration, UnaryOp, UseKind,
};

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

/// Generates pattern code
fn gen_pattern(pattern: Pattern, body: Either<Block, Expression>) -> js::Tokens {
    quote! {
        $(match pattern {
            // Int, float, bool patterns
            Pattern::Int(val) | Pattern::Float(val) | Pattern::Bool(val)  => {
                new $("$$")EqPattern($(val.as_str()), function() {
                    $(match body {
                        Either::Left(block) => $(gen_block_expr(block)),
                        Either::Right(expr) => return $(gen_expression(expr))
                    })
                })
            },
            // String pattern
            Pattern::String(val) => {
                new $("$$")EqPattern($(quoted(val.as_str())), function() {
                    $(match body {
                        Either::Left(block) => $(gen_block_expr(block)),
                        Either::Right(expr) => return $(gen_expression(expr))
                    })
                })
            }
            // Unwrap pattern of fields {field, field, n..}
            Pattern::Unwrap { en, fields } => {
                new $("$$")UnwrapPattern(
                    $(match en {
                        Expression::SuffixVar { name, .. } => $(quoted(try_escape_js(&name))),
                        _ => $(quoted("unreachable"))
                    }),
                    [$(for field in fields.clone() join (, ) => $(quoted(try_escape_js(&field.1))))],
                    function($("$$fields")) {
                        $(for field in fields => let $(try_escape_js(&field.1)) = $("$$fields").$(try_escape_js(&field.1));$['\r'])
                        $(match body {
                            Either::Left(block) => $(gen_block_expr(block)),
                            Either::Right(expr) => return $(gen_expression(expr))
                        })
                    }
                )
            },
            // Wildcard pattern
            Pattern::Wildcard => {
                new $("$$")WildcardPattern(function() {
                    $(match body {
                        Either::Left(block) => $(gen_block_expr(block)),
                        Either::Right(expr) => return $(gen_expression(expr))
                    })
                })
            }
            // BindTo(var) pattern
            Pattern::BindTo(var) => {
                new $("$$")BindPattern(function($("$$it")) {
                    $(try_escape_js(var.as_str())) = $("$$it")
                    $(match body {
                        Either::Left(block) => $(gen_block_expr(block)),
                        Either::Right(expr) => return $(gen_expression(expr))
                    })
                })
            }
            // Variant(var) pattern
            Pattern::Variant(var) => {
                new $("$$")VariantPattern(
                    $(match var {
                        Expression::SuffixVar { name, .. } => $(quoted(try_escape_js(name.as_str()))),
                        _ => $(quoted("unreachable"))
                    }),
                    function() {
                        $(match body {
                            Either::Left(block) => $(gen_block_expr(block)),
                            Either::Right(expr) => return $(gen_expression(expr))
                        })
                    }
                )
            }
            // Or(pat1, pat2) pattern
            Pattern::Or(pat1, pat2) => {
                new $("$$")OrPattern($(gen_pattern(*pat1, body.clone())), $(gen_pattern(*pat2, body)))
            }
        })
    }
}

/// Generates range code
fn gen_range(range: Range) -> js::Tokens {
    match range {
        Range::ExcludeLast { from, to, .. } => {
            quote!($("$$range")($(gen_expression(from)), $(gen_expression(to)), 0))
        }
        Range::IncludeLast { from, to, .. } => {
            quote!($("$$range")($(gen_expression(from)), $(gen_expression(to)), 1))
        }
    }
}

/// Generates expression code
pub fn gen_expression(expr: Expression) -> js::Tokens {
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
        } => match op {
            // With string values
            BinaryOp::Concat => quote!( $(gen_expression(*left)) + $(gen_expression(*right)) ),
            // With number values
            BinaryOp::Add => quote!( $(gen_expression(*left)) + $(gen_expression(*right)) ),
            BinaryOp::Sub => quote!( $(gen_expression(*left)) - $(gen_expression(*right)) ),
            BinaryOp::Mul => quote!( $(gen_expression(*left)) * $(gen_expression(*right)) ),
            BinaryOp::Div => quote!( $(gen_expression(*left)) / $(gen_expression(*right)) ),
            BinaryOp::Xor => quote!( $(gen_expression(*left)) ^ $(gen_expression(*right)) ),
            BinaryOp::BitwiseAnd => {
                quote!( $(gen_expression(*left)) & $(gen_expression(*right)) )
            }
            BinaryOp::BitwiseOr => quote!( $(gen_expression(*left)) | $(gen_expression(*right)) ),
            BinaryOp::Mod => quote!( $(gen_expression(*left)) % $(gen_expression(*right)) ),
            BinaryOp::Gt => quote!( $(gen_expression(*left)) > $(gen_expression(*right)) ),
            BinaryOp::Lt => quote!( $(gen_expression(*left)) < $(gen_expression(*right)) ),
            BinaryOp::Ge => quote!( $(gen_expression(*left)) >= $(gen_expression(*right)) ),
            BinaryOp::Le => quote!( $(gen_expression(*left)) <= $(gen_expression(*right)) ),
            // With bool
            BinaryOp::Or => quote!( $(gen_expression(*left)) || $(gen_expression(*right)) ),
            BinaryOp::And => quote!( $(gen_expression(*left)) && $(gen_expression(*right)) ),
            BinaryOp::Eq => {
                quote!( $("$$equals")($(gen_expression(*left)), $(gen_expression(*right))) )
            }
            BinaryOp::NotEq => {
                quote!( !$("$$equals")($(gen_expression(*left)), $(gen_expression(*right))) )
            }
        },
        Expression::As { value, .. } => gen_expression(*value),
        Expression::Unary { value, op, .. } => match op {
            UnaryOp::Neg => quote!( -$(gen_expression(*value)) ),
            UnaryOp::Bang => quote!( !$(gen_expression(*value)) ),
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
                function ($(for param in params join (, ) => $(try_escape_js(&param.name)))) {
                    $(match body {
                        Either::Left(block) => $(gen_block_expr(block)),
                        Either::Right(expr) => return $(gen_expression(*expr))
                    })
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
                        $(gen_pattern(case.pattern, case.body))
                    })
                    $['\r']
                ])
            }
        }
        Expression::If {
            logical,
            body,
            else_branches,
            ..
        } => {
            quote! {
                (() => {
                   if ($(gen_expression(*logical))) {
                       $(match body {
                           Either::Left(block) => $(gen_block_expr(block)),
                           Either::Right(expr) => return $(gen_expression(*expr))
                       })
                   }
                   $(for branch in else_branches {
                       $(match branch {
                           ElseBranch::Elif { logical, body, .. } => {
                               else if ($(gen_expression(logical))) {
                                   $(match body {
                                       Either::Left(block) => $(gen_block_expr(block)),
                                       Either::Right(expr) => return $(gen_expression(expr))
                                   })
                               }
                               $['\r']
                           }
                           ElseBranch::Else { body, .. } => {
                               else {
                                   $(match body {
                                       Either::Left(block) => $(gen_block_expr(block)),
                                       Either::Right(expr) => return $(gen_expression(expr))
                                   })
                               }
                               $['\r']
                           }
                       })
                   })
                })()
            }
        }
        Expression::Todo { .. } => quote!($("$$todo()")),
    }
}

/// Generates statement code
pub fn gen_statement(stmt: Statement) -> js::Tokens {
    match stmt {
        // Loop statement
        Statement::Loop { logical, body, .. } => quote! {
            while ($(gen_expression(logical))) {
                $(match body {
                    Either::Left(block) => $(gen_block(block)),
                    Either::Right(expr) => $(gen_expression(expr));
                })
            }
        },
        // For statement
        Statement::For {
            name, range, body, ..
        } => quote! {
            for (const $(name.as_str()) of $(gen_range(*range))) {
                $(match body {
                    Either::Left(block) => $(gen_block(block)),
                    Either::Right(expr) => $(gen_expression(expr));
                })
            }
        },
        // Variable definition statement
        Statement::VarDef { name, value, .. } => quote! {
            let $(try_escape_js(&name)) = $(gen_expression(value))
        },
        // Variable assignment statement
        Statement::VarAssign { what, value, .. } => quote! {
            $(gen_expression(what)) = $(gen_expression(value))
        },
        // Expression statement
        Statement::Expr(expr) => quote!($(gen_expression(expr))),
        // Semicolon expression statement
        Statement::Semi(expr) => quote!($(gen_expression(expr));),
    }
}

/// Generates function declaration code
pub fn gen_fn_declaration(decl: FnDeclaration) -> js::Tokens {
    match decl {
        FnDeclaration::Function {
            name, params, body, ..
        } => {
            // function $name($param, $param, n...)
            quote! {
                export function $(try_escape_js(&name))($(for param in params join (, ) => $(try_escape_js(&param.name)))) {
                    $(match body {
                        Either::Left(block) => $(gen_block_expr(block)),
                        Either::Right(expr) => return $(gen_expression(expr))
                    })
                }
            }
        }
        FnDeclaration::ExternFunction {
            name, params, body, ..
        } => {
            quote! {
                export function $(try_escape_js(&name))($(for param in params join (, ) => $(try_escape_js(&param.name)))) {
                    $(body.to_string())
                }
            }
        }
    }
}

/// Generates type declaration code
pub fn gen_type_declaration(decl: TypeDeclaration) -> js::Tokens {
    match decl {
        TypeDeclaration::Struct { name, fields, .. } => {
            // constructor($field, $field, n...)
            // with meta type field as `type_name`
            let generated_constructor = quote! {
                constructor($(for field in &fields join (, ) => $(try_escape_js(&field.name)))) {
                    this.$("$meta") = "Type";
                    this.$("$type") = $(quoted(name.to_string()));
                    $(for decl in &fields {
                        this.$(try_escape_js(&decl.name)) = $(try_escape_js(&decl.name))
                        $['\r']
                    })
                }
            };

            // Class of `Type` named as $type_name
            // and class fabric named as `type_name`
            quote! {
                export class $("$")$(try_escape_js(&name)) {
                    $generated_constructor
                }
                export function $(try_escape_js(&name))($(for field in &fields join (, ) => $(try_escape_js(&field.name)))) {
                    return new $("$")$(try_escape_js(&name))($(for field in &fields join (, ) => $(try_escape_js(&field.name))));
                }
            }
        }
        TypeDeclaration::Enum { name, variants, .. } => {
            // ($variant_name): ($param, $param, n...): ({
            //    $meta: "Enum"
            //    $enum: $name
            //    $param: $param,
            //    $param: $param,
            //    n...
            // })
            let variants: js::Tokens = quote!($(for variant in variants join(,$['\r']) =>
                $(variant.name.as_str()): ($(for param in variant.params.clone() join (, ) => $(try_escape_js(&param.name)))) => ({
                    $("$meta"): "Enum",
                    $("$enum"): $(quoted(name.as_str())),
                    $("$variant"): $(quoted(variant.name.as_str())),
                    $(for param in variant.params.clone() join (, ) => $(try_escape_js(&param.name)): $(try_escape_js(&param.name)))
                })
            ));

            // constr $name = {}
            quote! {
                export const $(try_escape_js(&name)) = {
                    $variants
                };
            }
        }
    }
}

/// Generates const declaration code
pub fn gen_const_declaration(decl: ConstDeclaration) -> js::Tokens {
    quote! {
        export const $(try_escape_js(&decl.name)) = $(gen_expression(decl.value));
    }
}

/// Generates declaration code
pub fn gen_declaration(decl: Declaration) -> js::Tokens {
    match decl {
        Declaration::Fn(decl) => gen_fn_declaration(decl),
        Declaration::Const(decl) => gen_const_declaration(decl),
        Declaration::Type(decl) => gen_type_declaration(decl),
    }
}

/// Generates block code
pub fn gen_block(block: Block) -> js::Tokens {
    quote! {
        $(for stmt in block.body join ($['\r']) => $(gen_statement(stmt)))
    }
}

/// Generates block code with last statement as return
pub fn gen_block_expr(mut block: Block) -> js::Tokens {
    let last = match block.body.pop() {
        Some(last) => last,
        None => return quote!(),
    };
    quote! {
        $(for stmt in block.body join ($['\r']) => $(gen_statement(stmt)))
        $(match last {
            Statement::Expr(last) => return $(gen_expression(last)),
            it => $(gen_statement(it))
        })
    }
}

/// Generates module code
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
        import {
            $("$$match"),
            $("$$equals"),
            $("$$todo"),
            $("$$range"),
            $("$$EqPattern"),
            $("$$UnwrapPattern"),
            $("$$WildcardPattern"),
            $("$$BindPattern"),
            $("$$VariantPattern"),
        } from $(quoted(format!("{dependencies_prefix}prelude.js")))
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

/// Generates prelude code
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
            constructor(variant, fields, unwrap_fn) {
                this.variant = variant;
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
                        // If variant is same
                        if (value.$("$variant") == this.variant) {
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

        // WildcardPattern$Class
        export class $("$$WildcardPattern") {
            constructor(eq_fn) {
                this.eq_fn = eq_fn;
            }
            evaluate(value) {
                return [true, this.eq_fn()];
            }
        }

        // BindPattern$Class
        export class $("$$BindPattern") {
            constructor(eq_fn) {
                this.eq_fn = eq_fn;
            }
            evaluate(value) {
                return [true, this.eq_fn(value)];
            }
        }

        // VariantPattern$Class
        export class $("$$VariantPattern") {
            constructor(variant, eq_fn) {
                this.variant = variant
                this.eq_fn = eq_fn;
            }
            evaluate(value) {
                // Checking meta existence
                if ("$meta" in value) {
                    // Meta
                    let meta = value.$("$meta");
                    // Checking it's an enum
                    if (meta == "Enum") {
                        // If variant is same
                        if (value.$("$variant") == this.variant) {
                            return [true, this.eq_fn(value)];
                        } else {
                            return [false, null]
                        }
                    }
                }
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

        // Todo$Fn
        export function $("$$todo")() {
            throw "reached todo code.";
        }

        // Range$Fn
        export function $("$$range")(from, to, offset) {
            const result = [];
            // 0..5
            if (from < to) {
                for (let i = from; i < to + offset; i += 1) {
                    result.push(i);
                }
            }
            // 5..0
            else {
                for (let i = from; i > to - offset; i -= 1) {
                    result.push(i);
                }
            }
            return result;
        }
    }
}

/// Generates index file code
pub fn gen_index(main_module: String) -> js::Tokens {
    quote! {
        import { main } from $(quoted(format!("./{main_module}.js")))
        main();
    }
}
