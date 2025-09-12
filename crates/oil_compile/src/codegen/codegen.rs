/// Imports
use genco::{lang::js, quote, quote_in, tokens::quoted};
use oil_ir::ir::{IrBinaryOp, IrBlock, IrExpression, IrPattern, IrStatement, IrUnaryOp};

/// Generates expression
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
            IrBinaryOp::Eq => {
                let spec_eq = "$equals";
                quote!( $spec_eq($(gen_expression(*left)) $(gen_expression(*right))) )
            }
            IrBinaryOp::Neq => {
                let spec_eq = "$equals";
                quote!( !$spec_eq($(gen_expression(*left)) $(gen_expression(*right))) )
            }
            IrBinaryOp::Gt => quote!( $(gen_expression(*left)) > $(gen_expression(*right)) ),
            IrBinaryOp::Lt => quote!( $(gen_expression(*left)) < $(gen_expression(*right)) ),
            IrBinaryOp::Ge => quote!( $(gen_expression(*left)) >= $(gen_expression(*right)) ),
            IrBinaryOp::Le => quote!( $(gen_expression(*left)) <= $(gen_expression(*right)) ),
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
        IrExpression::Range {
            location: _,
            from,
            to,
        } => todo!(),
        IrExpression::Match {
            location: _,
            value,
            cases,
        } => todo!(),
    }
}

/// Generates statement
pub fn gen_statement(stmt: IrStatement) -> js::Tokens {
    match stmt {
        IrStatement::If {
            location: _,
            logical,
            body,
            elseif,
        } => todo!(),
        IrStatement::While {
            location: _,
            logical,
            body,
        } => quote! {
            while $(gen_expression(logical)) {
                $(gen_block(body))
            }
        },
        IrStatement::Define {
            location: _,
            name,
            value,
            typ: _,
        } => quote! {
            let $(name.to_string()) = $(gen_expression(value));
        },
        IrStatement::Assign {
            location: _,
            what,
            value,
        } => quote! {
            let $(gen_expression(what)) = $(gen_expression(value));
        },
        IrStatement::Call {
            location: _,
            what,
            args,
        } => quote! {
            $(gen_expression(what))($(for arg in args join (, ) => $(gen_expression(arg))));
        },
        IrStatement::Fn {
            location: _,
            typ: _,
            name,
            params,
            body,
        } => quote! {
            function $(name.to_string())($(for param in params join (, ) => $(param.name.to_string()))) {
                $(gen_block(body))
            }
        },
        IrStatement::Break { location: _ } => quote!(break;),
        IrStatement::Continue { location: _ } => quote!(continue;),
        IrStatement::Return { location: _, value } => quote!(return $(gen_expression(value));),
        IrStatement::Match {
            location: _,
            value,
            cases,
        } => {
            let spec_match = "$match";
            let spec_fields = "$fields";
            let cases = quote!($(for case in cases join (,$['\r']) {
                $(match case.pattern {
                    IrPattern::Value(val) => {
                        new ValPattern($(gen_expression(val)), function() {
                            $(gen_block(case.body))
                        })
                    },
                    IrPattern::Unwrap { en: _, fields } => {
                        new UnwrapPattern([$(for field in fields.clone() join (, ) => $(quoted(field.to_string())))], function($spec_fields) {
                            $(for field in fields => let $(field.clone().to_string()) = $spec_fields.$(field.to_string());$['\r'])
                            $(gen_block(case.body))
                        })
                    },
                    _ => todo!()
                })
            }));
            quote!($spec_match($(gen_expression(value)),
                [
                    $['\r']
                    $cases
                    $['\r']
                ]
            ))
        }
        IrStatement::For {
            location: _,
            iterable,
            variable,
            body,
        } => todo!(),
    }
}

/// Generates block
pub fn gen_block(block: IrBlock) -> js::Tokens {
    quote! {
        $(for stmt in block.nodes join ($['\n'] ) => $(gen_statement(stmt)))
    }
}
