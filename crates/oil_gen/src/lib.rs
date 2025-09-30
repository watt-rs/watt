/// Imports
use ecow::EcoString;
use genco::{lang::js, quote, quote_in, tokens::quoted};
use oil_ir::ir::{
    IrBinaryOp, IrBlock, IrDeclaration, IrDependencyKind, IrExpression, IrModule, IrPattern,
    IrStatement, IrUnaryOp,
};

/// Generates expression code
pub fn gen_expression(expr: IrExpression) -> js::Tokens {
    match expr {
        IrExpression::Float { location: _, value } => quote! ( $(value.to_string()) ),
        IrExpression::Int { location: _, value } => quote! ( $(value.to_string()) ),
        IrExpression::String { location: _, value } => quote! ( $(quoted(value.as_str())) ),
        IrExpression::Bool { location: _, value } => quote! ( $(value.as_str()) ),
        IrExpression::Bin {
            location: _,
            left,
            right,
            op,
        } => match op {
            // With string values
            IrBinaryOp::Concat => quote!( $(gen_expression(*left)) + $(gen_expression(*right)) ),
            // With number values
            IrBinaryOp::Add => quote!( $(gen_expression(*left)) + $(gen_expression(*right)) ),
            IrBinaryOp::Sub => quote!( $(gen_expression(*left)) - $(gen_expression(*right)) ),
            IrBinaryOp::Mul => quote!( $(gen_expression(*left)) * $(gen_expression(*right)) ),
            IrBinaryOp::Div => quote!( $(gen_expression(*left)) / $(gen_expression(*right)) ),
            IrBinaryOp::Or => quote!( $(gen_expression(*left)) || $(gen_expression(*right)) ),
            IrBinaryOp::And => quote!( $(gen_expression(*left)) && $(gen_expression(*right)) ),
            IrBinaryOp::Xor => quote!( $(gen_expression(*left)) ^ $(gen_expression(*right)) ),
            IrBinaryOp::BitwiseAnd => {
                quote!( $(gen_expression(*left)) & $(gen_expression(*right)) )
            }
            IrBinaryOp::BitwiseOr => quote!( $(gen_expression(*left)) | $(gen_expression(*right)) ),
            IrBinaryOp::Mod => quote!( $(gen_expression(*left)) % $(gen_expression(*right)) ),
            IrBinaryOp::Gt => quote!( $(gen_expression(*left)) > $(gen_expression(*right)) ),
            IrBinaryOp::Lt => quote!( $(gen_expression(*left)) < $(gen_expression(*right)) ),
            IrBinaryOp::Ge => quote!( $(gen_expression(*left)) >= $(gen_expression(*right)) ),
            IrBinaryOp::Le => quote!( $(gen_expression(*left)) <= $(gen_expression(*right)) ),
            // With bool
            IrBinaryOp::Eq => {
                quote!( $("$$equals")($(gen_expression(*left)), $(gen_expression(*right))) )
            }
            IrBinaryOp::Neq => {
                quote!( !$("$$equals")($(gen_expression(*left)), $(gen_expression(*right))) )
            }
        },
        IrExpression::Unary { value, op, .. } => match op {
            IrUnaryOp::Negate => quote!( -$(gen_expression(*value)) ),
            IrUnaryOp::Bang => quote!( !$(gen_expression(*value)) ),
        },
        IrExpression::Get { name, .. } => quote!($(name.as_str())),
        IrExpression::FieldAccess {
            location: _,
            container,
            name,
        } => quote!($(gen_expression(*container)).$(name.as_str())),
        IrExpression::Call {
            location: _,
            what,
            args,
        } => quote! {
            $(gen_expression(*what))($(for arg in args join (, ) => $(gen_expression(arg))))
        },
        IrExpression::AnFn { params, body, .. } => {
            // function ($param, $param, n...)
            quote! {
                function ($(for param in params join (, ) => $(param.name.to_string()))) {
                    $(gen_block(body))
                }
            }
        }
        IrExpression::Range { .. } => todo!(),
    }
}

/// Generates statement
pub fn gen_statement(stmt: IrStatement) -> js::Tokens {
    match stmt {
        // If statement
        IrStatement::If {
            logical,
            body,
            elseif,
            ..
        } => {
            // Generating base if
            let mut base = quote! {
                if ($(gen_expression(logical))) {
                    $(gen_block(body))
                }
                $['\r']
            };
            // Generating else cases
            let mut last = elseif;
            while let Some(case) = last
                && let IrStatement::If {
                    logical,
                    body,
                    elseif,
                    ..
                } = *case
            {
                // Quoting else case
                quote_in! { base =>
                    else if ($(gen_expression(logical))) {
                        $(gen_block(body))
                    }
                    $['\r']
                }
                last = elseif;
            }
            base
        }
        // While statement
        IrStatement::While { logical, body, .. } => quote! {
            while $(gen_expression(logical)) {
                $(gen_block(body))
            }
        },
        // Define statement
        IrStatement::Define { name, value, .. } => quote! {
            let $(name.to_string()) = $(gen_expression(value));
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
            function $(name.to_string())($(for param in params join (, ) => $(param.name.to_string()))) {
                $(gen_block(body))
            }
        },
        // Break statement
        IrStatement::Break { .. } => quote!(break;),
        // Continue statement
        IrStatement::Continue { .. } => quote!(continue;),
        // Return statement
        IrStatement::Return { value, .. } => quote!(return $(gen_expression(value));),
        // Match statement
        IrStatement::Match {
            location: _,
            value,
            cases,
        } => {
            // Pattern matching
            quote! {
                $("$$match")($(gen_expression(value)), [
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
                                new $("$$")UnwrapPattern([$(for field in fields.clone() join (, ) => $(quoted(field.as_str())))], function($("fields")) {
                                    $(for field in fields => let $(field.clone().to_string()) = $("fields").$(field.as_str());$['\r'])
                                    $(gen_block(case.body))
                                })
                            },
                            // Range pattern
                            IrPattern::Range { .. } => todo!()
                        })
                    })
                    $['\r']
                ])
            }
        }
        IrStatement::For { .. } => todo!(),
    }
}

/// Generates declaration
pub fn gen_declaration(decl: IrDeclaration) -> js::Tokens {
    match decl {
        IrDeclaration::Function(ir_function) => {
            // function $name($param, $param, n...)
            quote! {
                export function $(ir_function.name.to_string())($(for param in ir_function.params join (, ) => $(param.name.to_string()))) {
                    $(gen_block(ir_function.body))
                }
            }
        }
        IrDeclaration::Variable(ir_variable) => quote! {
            export let $(ir_variable.name.to_string()) = $(gen_expression(ir_variable.value));
        },
        IrDeclaration::Type(ir_type) => {
            // Methods
            let methods = quote! {
                $(for function in ir_type.functions =>
                    $(function.name.to_string())($(for param in function.params join (, ) => $(param.name.to_string()))) {
                        let self = this;
                        $(gen_block(function.body))
                    }
                )
            };

            // constructor($field, $field, n...)
            // with meta type field as `type_name`
            let constructor = quote! {
                constructor($(for field in ir_type.fields.clone() join (, ) => $(field.name.to_string()))) {
                    this.$("$")meta = $(quoted(ir_type.name.to_string()));
                    $(for field in ir_type.fields.clone() join ($['\r']) => this.$(field.name.to_string()) = $(gen_expression(field.value));)
                }
            };

            // Class of `Type` named as $type_name
            // and class fabric named as `type_name`
            quote! {
                export class $("$")$(ir_type.name.to_string()) {
                    $constructor
                    $methods
                }
                export function $(ir_type.name.to_string())($(for field in ir_type.fields.clone() join (, ) => $(field.name.to_string()))) {
                    return new $("$")$(ir_type.name.to_string())($(for field in ir_type.fields join (, ) => $(field.name.to_string())));
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
                export const $(ir_enum.name.to_string()) = {
                    $variants
                };
            }
        }
        IrDeclaration::Extern(ir_extern) => {
            quote! {
                export function $(ir_extern.name.to_string())($(for param in ir_extern.params join (, ) => $(param.name.to_string()))) {
                    $(ir_extern.body.to_string())
                }
            }
        }
    }
}

/// Generates block
pub fn gen_block(block: IrBlock) -> js::Tokens {
    // Block of statement
    quote! {
        $(for stmt in block.nodes join ($['\r']) => $(gen_statement(stmt)))
    }
}

/// Generates module
pub fn gen_module(name: &EcoString, module: &IrModule) -> js::Tokens {
    // Segments amount for dependencies
    let name_segments_amount = name.split("/").count();
    // Dependencies prefix
    let dependencies_prefix = match name_segments_amount {
        1 => String::from("./"),
        _ => String::from("../".repeat(name_segments_amount - 1)),
    };
    // Gen
    quote! {
        // Prelude
        import {$("$$match"), $("$$equals")} from $(quoted(format!("{dependencies_prefix}prelude.js")))
        // Dependencies
        //
        // for `AsName`: import * as $name from "$module"
        // for `ForNames`: import {$name, $name, ...} from "$module"
        $(for dep in module.dependencies.clone() join ($['\r']) => $(match dep.kind {
            IrDependencyKind::AsName(name) => {
                import * as $(name.to_string()) from $(quoted(format!("{dependencies_prefix}{}.js", dep.path.as_str())))
            },
            IrDependencyKind::ForNames(names) => {
                import {$(for name in names join(, ) => $(name.to_string()))} from $(quoted(format!("{dependencies_prefix}{}.js", dep.path.as_str())))
            },
        }))
        $['\n']
        // Declarations
        $(for decl in module.definitions.clone() join ($['\n']) => $(gen_declaration(decl)))
    }
}

/// Generates prelude
pub fn gen_prelude() -> js::Tokens {
    quote! {
        // EnumEquals$fn
        function $("$$")enum_equals(a, b) {
            // Gettting keys
            let a_keys = Object.keys(a);
            let b_keys = Object.keys(b);
            // Checking length
            if (a_keys.length != b_keys.length) {
                return false
            }
            // Checking entries
            for (const k1 of a_keys) {
                // If b keys includes a key
                if (b_keys.includes(k1)) {
                    // Comparing values
                    if ($("$")equals(a[k1], b[k1]) == false) {
                        return false
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
        export function $("$$")equals(a, b) {
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
                        let a_meta = a.$("$")meta;
                        let b_meta = b.$("$")meta;
                        // If meta is different
                        if (a_meta != b_meta) {
                            return false
                        } else {
                            // Meta
                            let meta = a_meta;
                            // If meta is $Enum
                            if (meta == "Enum") {
                                // Comparing enums
                                return $("$")enum_equals(a, b)
                            }
                            return a === b;
                        }
                    }
                } else {
                    return a == b;
                }
            }
        }

        // Match$Fn
        export function $("$$")match() {

        }
    }
}

/// Generates index file
pub fn gen_index(main_module: String) -> js::Tokens {
    quote! {
        import { main } from $(quoted(format!("./{}.js", main_module)))
        main();
    }
}
