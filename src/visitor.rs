use std::collections::HashMap;
use crate::ast::Node;
use crate::lexer::Token;
/*
Визитор (компилятор)
 */

pub fn visit_node(node: Node) {
    match node {
        Node::Number { value } =>
            visit_number(value),
        Node::String { value } =>
            visit_string(value),
        Node::Bool { value } =>
            visit_bool(value),
        Node::Bin { left, right, op } =>
            visit_bin(left, right, op),
        Node::Unary { value, op } =>
            visit_unary(value, op),
        Node::If { location, logical, body, elseif } =>
            visit_if(location, logical, body, elseif),
        Node::While { location, logical, body } =>
            visit_while(location, logical, body),
        Node::Define { previous, name, value } =>
            visit_define(previous, name, value),
        Node::Assign { previous, name, value } =>
            visit_assign(previous, name, value),
        Node::Get { previous, name, should_push } =>
            visit_get(previous, name, should_push),
        Node::Call { previous, name, args, should_push } =>
            visit_call(previous, name, args, should_push),
        Node::FnDeclaration { name, full_name, args, body } =>
            visit_fn_decl(name, full_name, args, body),
        Node::AnFnDeclaration { args, body } =>
            visit_an_fn_decl(args, body),
        Node::Break { location } =>
            visit_break(location),
        Node::Continue { location } =>
            visit_continue(location),
        Node::Import { imports } =>
            visit_import(imports),
        Node::List { location, values } =>
            visit_list(location, values),
        Node::Cond { left, right, op } =>
            visit_cond(left, right, op),
        Node::Logical { left, right, op } =>
            visit_logical(left, right, op),
        Node::Map { location, values } =>
            visit_map(location, values),
        Node::Match { location, matchable, cases, default } =>
            visit_match(location, matchable, cases, default),
        Node::Native { name } =>
            visit_native(name),
        Node::Instance { name, constructor } =>
            visit_instance(name, constructor),
        Node::Ret { location, value } =>
            visit_ret(location, value),
        Node::Null { location } =>
            visit_null(location),
        Node::Type { name, fullname, constructor, body } =>
            visit_type(name, fullname, constructor, body),
        Node::Unit { name, fullname, body } =>
            visit_unit(name, fullname, body),
        Node::For { iterable, variable_name } => {
            visit_for(iterable, variable_name)
        }
        Node::Block { body } => {
            visit_block(body);
        }
    }
}

pub fn visit_block(body: Vec<Box<Node>>) {

}

pub fn visit_number(value: Token) {

}

pub fn visit_string(value: Token) {

}

pub fn visit_bool(value: Token) {

}

pub fn visit_bin(left: Box<Node>, right: Box<Node>, op: Token) {

}

pub fn visit_if(location: Token, logical: Box<Node>, body: Box<Node>, elif: Option<Box<Node>>) {

}

pub fn visit_while(location: Token, logical: Box<Node>, body: Box<Node>) {

}

pub fn visit_define(previous: Option<Box<Node>>, name: Token, value: Box<Node>) {

}

pub fn visit_call(previous: Option<Box<Node>>, name: Token, args: Vec<Box<Node>>, should_push: bool) {

}

pub fn visit_fn_decl(name: Token, full_name: Token, args: Vec<Token>, body: Box<Node>) {

}

pub fn visit_break(location: Token) {

}

pub fn visit_continue(location: Token) {

}

pub fn visit_import(imports: Vec<Token>) {

}

pub fn visit_list(location: Token, list: Vec<Box<Node>>) {

}

pub fn visit_map(location: Token, map: HashMap<Box<Node>, Box<Node>>) {

}

pub fn visit_for(iterable: Box<Node>, variable_name: Token) {

}

pub fn visit_unary(value: Box<Node>, op: Token) {

}

pub fn visit_type(name: Token, fullname: Token, constructor: Vec<Token>, body: Box<Node>) {

}

pub fn visit_unit(name: Token, fullname: Token, body: Box<Node>) {

}

pub fn visit_match(location: Token, matchable: Box<Node>, cases: Vec<Box<Node>>, default: Box<Node>) {

}

pub fn visit_an_fn_decl(args: Vec<Token>, body: Box<Node>) {

}

pub fn visit_native(name: Token) {

}

pub fn visit_cond(left: Box<Node>, right: Box<Node>, op: Token) {

}

pub fn visit_logical(left: Box<Node>, right: Box<Node>, op: Token) {

}

pub fn visit_ret(location: Token, value: Box<Node>) {

}

pub fn visit_null(location: Token) {

}

pub fn visit_instance(name: Token, constructor: Vec<Box<Node>>) {

}

pub fn visit_assign(previous: Option<Box<Node>>, name: Token, value: Box<Node>) {

}

pub fn visit_get(previous: Option<Box<Node>>, name: Token, should_push: bool) {

}