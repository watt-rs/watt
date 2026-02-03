/// Imports
use crate::cx::GenCx;
use ecow::EcoString;
use genco::{
    lang::{JavaScript, js},
    quote,
};
use watt_ast::ast::{Block, Either, Expression, Pattern};

/// Javascript code generation context
#[derive(Default)]
pub struct JsGenCx;

/// Generation context implementation
impl GenCx<JavaScript> for JsGenCx {
    /// Escapes javascript keyword
    fn escape_kw(&self, keyword: &str) -> String {
        if matches!(
            keyword,
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
            format!("{keyword}$")
        } else {
            keyword.to_owned()
        }
    }

    /// Generates literal
    fn gen_literal(&self, val: EcoString) -> js::Tokens {
        quote!($(val.as_str()))
    }

    /// Generates literal pattern
    fn gen_literal_pattern(&self, val: EcoString, body: Either<Block, Expression>) -> js::Tokens {
        quote!(
            new $("$$")EqPattern($(val.as_str()), function() {
                $(match body {
                    Either::Left(block) => $(self.gen_block_expr(block)),
                    Either::Right(expr) => return $(self.gen_expr(expr))
                })
            })
        )
    }

    /// Generates string pattern
    fn gen_string_pattern(&self, val: EcoString, body: Either<Block, Expression>) -> js::Tokens {
        quote!(
            new $("$$")EqPattern($(val.as_str()), function() {
                $(match body {
                    Either::Left(block) => $(self.gen_block_expr(block)),
                    Either::Right(expr) => return $(self.gen_expr(expr))
                })
            })
        )
    }

    /// Generates pattern
    fn gen_pattern(&self, pattern: Pattern, body: Either<Block, Expression>) -> js::Tokens {
        match pattern {
            Pattern::Unwrap {
                address,
                en,
                fields,
            } => todo!(),
            Pattern::Variant(address, expression) => todo!(),
            Pattern::Int(_, lit) | Pattern::Float(_, lit) | Pattern::Bool(_, lit) => {
                self.gen_literal_pattern(lit, body)
            }
            Pattern::String(address, eco_string) => todo!(),
            Pattern::BindTo(address, eco_string) => todo!(),
            Pattern::Wildcard => todo!(),
            Pattern::Or(pattern, pattern1) => todo!(),
        }
    }

    fn gen_range(&self, range: watt_ast::ast::Range) -> genco::Tokens {
        todo!()
    }

    fn gen_ty_path(&self, decl: watt_ast::ast::TypePath) -> genco::Tokens {
        todo!()
    }

    fn gen_expr(&self, expr: watt_ast::ast::Expression) -> genco::Tokens {
        todo!()
    }

    fn gen_stmt(&self, stmt: watt_ast::ast::Statement) -> genco::Tokens {
        todo!()
    }

    fn gen_fn_decl(&self, decl: watt_ast::ast::FnDeclaration) -> genco::Tokens {
        todo!()
    }

    fn gen_ty_decl(&self, decl: watt_ast::ast::TypeDeclaration) -> genco::Tokens {
        todo!()
    }

    fn gen_const_decl(&self, decl: watt_ast::ast::ConstDeclaration) -> genco::Tokens {
        todo!()
    }

    fn gen_decl(&self, decl: watt_ast::ast::Declaration) -> genco::Tokens {
        todo!()
    }

    fn gen_block(&self, block: watt_ast::ast::Block) -> genco::Tokens {
        todo!()
    }

    fn gen_block_expr(&self, block: watt_ast::ast::Block) -> genco::Tokens {
        todo!()
    }

    fn gen_module(&self, module: watt_ast::ast::Module) -> genco::Tokens {
        todo!()
    }

    fn gen_index(&self, main_module: String) -> genco::Tokens {
        todo!()
    }
}
