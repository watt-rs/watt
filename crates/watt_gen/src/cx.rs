/// Imports
use ecow::EcoString;
use genco::Tokens;
use watt_ast::ast::{
    Block, ConstDeclaration, Declaration, Either, Expression, FnDeclaration, Module, Pattern,
    Range, Statement, TypeDeclaration, TypePath,
};

/// Generation context trait
pub trait GenCx<T: genco::lang::Lang> {
    /// Escapes target keyword
    fn escape_kw(&self, keyword: &str) -> String;

    /// Generates literal
    fn gen_literal(&self, lit: EcoString) -> Tokens<T>;

    /// Generates literal pattern
    fn gen_literal_pattern(&self, lit: EcoString, body: Either<Block, Expression>) -> Tokens<T>;

    /// Generates string pattern
    fn gen_string_pattern(&self, lit: EcoString, body: Either<Block, Expression>) -> Tokens<T>;

    /// Generates unwrap pattern
    fn gen_unwrap_pattern(
        &self,
        en: Expression,
        fields: Vec<(Address, EcoString)>,
        body: Either<Block, Expression>,
    ) -> Tokens<T>;

    /// Generates pattern
    fn gen_pattern(&self, pattern: Pattern, body: Either<Block, Expression>) -> Tokens<T>;

    /// Generates range code
    fn gen_range(&self, range: Range) -> Tokens<T>;

    /// Generates type path
    fn gen_ty_path(&self, decl: TypePath) -> Tokens<T>;

    /// Generates single expression
    fn gen_expr(&self, expr: Expression) -> Tokens<T>;

    /// Generates single statement
    fn gen_stmt(&self, stmt: Statement) -> Tokens<T>;

    /// Generates function declaration
    fn gen_fn_decl(&self, decl: FnDeclaration) -> Tokens<T>;

    /// Generates type declaration
    fn gen_ty_decl(&self, decl: TypeDeclaration) -> Tokens<T>;

    /// Generates constant declaration
    fn gen_const_decl(&self, decl: ConstDeclaration) -> Tokens<T>;

    /// Generates single declaration
    fn gen_decl(&self, decl: Declaration) -> Tokens<T>;

    /// Generates block
    fn gen_block(&self, block: Block) -> Tokens<T>;

    /// Generates block
    fn gen_block_expr(&self, block: Block) -> Tokens<T>;

    /// Generates all module declaration
    fn gen_module(&self, module: Module) -> Tokens<T>;

    /// Generates index file code
    fn gen_index(&self, main_module: String) -> Tokens<T>;
}
