use std::collections::VecDeque;
use crate::errors::Error;
use crate::import::Import;
use crate::lexer::lexer::*;
use crate::parser::ast::*;
use crate::vm::bytecode::{Chunk, Opcode};
use crate::vm::values::Value;
/*
Визитор (компилятор)
 */

struct CompileVisitor {
    opcodes: VecDeque<Vec<Opcode>>,
}

impl CompileVisitor {
    pub fn compile(&mut self, node: Node) -> Result<Chunk, Error> {
        self.push_chunk();
        match self.visit_node(node) {
            Err(e) => Err(e),
            Ok(_) => Ok(Chunk::new(self.pop().clone()))
        }
    }

    pub fn push_chunk(&mut self) {
        self.opcodes.push_front(vec![]);
    }

    pub fn pop_chunk(&mut self) -> Vec<Opcode> {
        match self.opcodes.pop_front() {
            Some(v) => v,
            None => panic!("couldn't pop from compiler-visitor stack. report to the developer."),
        }
    }

    pub fn push_instr(&mut self, op: Opcode) {
        self.opcodes.front_mut().push(op);
    }

    pub fn pop_instr(&mut self) -> Opcode {
        match self.opcodes.front_mut().pop() {
            Some(v) => v,
            None => panic!("couldn't pop instr from compiler-visitor stack. report to the developer."),
        }
    }

    pub fn visit_node(&mut self, node: Node) -> Result<(), Error>   {
        match node {
            Node::Number { value } =>
                self.visit_number(value),
            Node::String { value } =>
                self.visit_string(value),
            Node::Bool { value } =>
                self.visit_bool(value),
            Node::Bin { left, right, op } =>
                self.visit_bin(left, right, op),
            Node::Unary { value, op } =>
                self.visit_unary(value, op),
            Node::If { location, logical, body, elseif } =>
                self.visit_if(location, logical, body, elseif),
            Node::While { location, logical, body } =>
                self.visit_while(location, logical, body),
            Node::Define { previous, name, value } =>
                self.visit_define(previous, name, value),
            Node::Assign { previous, name, value } =>
                self.visit_assign(previous, name, value),
            Node::Get { previous, name, should_push } =>
                self.visit_get(previous, name, should_push),
            Node::Call { previous, name, args, should_push } =>
                self.visit_call(previous, name, args, should_push),
            Node::FnDeclaration { name, full_name, params, body } =>
                self.visit_fn_decl(name, full_name, params, body),
            Node::AnFnDeclaration { params, body, .. } =>
                self.visit_an_fn_decl(params, body),
            Node::Break { location } =>
                self.visit_break(location),
            Node::Continue { location } =>
                self.visit_continue(location),
            Node::Import { imports } =>
                self.visit_import(imports),
            Node::List { location, values } =>
                self.visit_list(location, values),
            Node::Cond { left, right, op } =>
                self.visit_cond(left, right, op),
            Node::Logical { left, right, op } =>
                self.visit_logical(left, right, op),
            Node::Map { location, values } =>
                self.visit_map(location, values),
            Node::Match { location, matchable, cases, default } =>
                self.visit_match(location, matchable, cases, default),
            Node::Native { name, fn_name } =>
                self.visit_native(name, fn_name),
            Node::Instance { name, constructor, should_push } =>
                self.visit_instance(name, constructor, should_push),
            Node::Ret { location, value } =>
                self.visit_ret(location, value),
            Node::Null { location } =>
                self.visit_null(location),
            Node::Type { name, full_name, constructor, body } =>
                self.visit_type(name, full_name, constructor, body),
            Node::Unit { name, full_name, body } =>
                self.visit_unit(name, full_name, body),
            Node::For { iterable, variable_name, body } =>
                self.visit_for(iterable, variable_name, body),
            Node::Block { body } =>
                self.visit_block(body)
        }
    }

    pub fn visit_block(&mut self, body: Vec<Box<Node>>) -> Result<(), Error> {
        for node in body {
            self.visit_node(*node)?
        }
        Ok(())
    }

    pub fn visit_number(&mut self, value: Token) -> Result<(), Error>  {
        if value.value.contains(".") {
            self.push_instr(Opcode::Push {
                value: Value::Float(
                    value.value.parse::<f64>().unwrap()
                )
            });
        }
        else {
            self.push_instr(Opcode::Push {
                value: Value::Integer(
                    value.value.parse::<i64>().unwrap()
                )
            });
        }
        Ok(())
    }

    pub fn visit_string(&mut self, value: Token) -> Result<(), Error>  {
        self.push_instr(Opcode::Push {
            value: Value::String(
                value.value,
            )
        });
        Ok(())
    }

    pub fn visit_bool(&mut self, value: Token) -> Result<(), Error>  {
        self.push_instr(Opcode::Push {
            value: Value::Bool(
                value.value.parse::<bool>().unwrap(),
            )
        });
        Ok(())
    }

    pub fn visit_bin(&mut self, left: Box<Node>, right: Box<Node>, op: Token) -> Result<(), Error>  {
        self.visit_node(*left)?;
        self.visit_node(*right)?;
        self.push_instr(Opcode::Bin {
           op: op.value,
        });
        Ok(())
    }

    pub fn visit_if(&mut self, location: Token, logical: Box<Node>,
                    body: Box<Node>, elif: Option<Box<Node>>) -> Result<(), Error>  {
        // logical
        self.push_chunk();
        self.visit_node(*logical)?;
        let logical = self.pop_chunk();
        // body
        self.push_chunk();
        self.visit_node(*body)?;
        let body = self.pop_chunk();
        // elif
        let mut elseif: Option<Box<Opcode>> = None;
        if let Some(n) = elif {
            self.visit_node(*n)?;
            elseif = Some(Box::new(
                self.pop_instr()
            ));
        }
        // if
        self.push(Opcode::If {
            cond: Box::new(Chunk::new(logical)),
            body: Box::new(Chunk::new(body)),
            elif: elseif,
        });
        Ok(())
    }

    pub fn visit_while(&mut self, location: Token, logical: Box<Node>,
                       body: Box<Node>) -> Result<(), Error>  {
        // logical
        self.push_chunk();
        self.visit_node(*logical)?;
        let logical = self.pop_chunk();
        // body
        self.push_chunk();
        self.visit_node(*body)?;
        let body = self.pop_chunk();
        // if
        let _if = Opcode::If {
            cond: Box::new(Chunk::new(logical)),
            body: Box::new(Chunk::new(body)),
            elif: None
        };
        // loop
        self.push(Opcode::Loop {
            body: Box::new(Chunk::new(_if)),
        });
        Ok(())
    }

    pub fn visit_define(&mut self, previous: Option<Box<Node>>, name: Token,
                        value: Box<Node>) -> Result<(), Error>  {

    }

    pub fn visit_call(&mut self, previous: Option<Box<Node>>,
                      name: Token, args: Vec<Box<Node>>, should_push: bool) -> Result<(), Error>  {

    }

    pub fn visit_fn_decl(&mut self, name: Token, full_name: Option<Token>,
                         args: Vec<Token>, body: Box<Node>) -> Result<(), Error>  {

    }

    pub fn visit_break(&mut self, location: Token) -> Result<(), Error>  {

    }

    pub fn visit_continue(&mut self, location: Token) -> Result<(), Error>  {

    }

    pub fn visit_import(&mut self, imports: Vec<Import>) -> Result<(), Error>  {

    }

    pub fn visit_list(&mut self, location: Token, list: Box<Vec<Box<Node>>>) -> Result<(), Error>  {

    }

    pub fn visit_map(&mut self, location: Token,
                     map: Box<Vec<(Box<Node>, Box<Node>)>>) -> Result<(), Error>  {

    }

    pub fn visit_for(&mut self, iterable: Box<Node>,
                     variable_name: Token, body: Box<Node>) -> Result<(), Error>  {

    }

    pub fn visit_unary(&mut self, value: Box<Node>, op: Token) -> Result<(), Error>  {

    }

    pub fn visit_type(&mut self, name: Token, full_name: Option<Token>,
                      constructor: Vec<Token>, body: Box<Node>) -> Result<(), Error>  {

    }

    pub fn visit_unit(&mut self, name: Token, full_name: Option<Token>,
                      body: Box<Node>)  -> Result<(), Error> {

    }

    pub fn visit_match(&mut self, location: Token, matchable: Box<Node>,
                       cases: Vec<Box<Node>>, default: Box<Node>) -> Result<(), Error>  {

    }

    pub fn visit_an_fn_decl(&mut self, args: Vec<Token>, body: Box<Node>) -> Result<(), Error>  {

    }

    pub fn visit_native(&mut self, name: Token, fn_name: Token) -> Result<(), Error>  {

    }

    pub fn visit_cond(&mut self, left: Box<Node>,
                      right: Box<Node>, op: Token) -> Result<(), Error>  {

    }

    pub fn visit_logical(&mut self, left: Box<Node>,
                         right: Box<Node>, op: Token) -> Result<(), Error>  {

    }

    pub fn visit_ret(&mut self, location: Token, value: Box<Node>) -> Result<(), Error>  {

    }

    pub fn visit_null(&mut self, location: Token) -> Result<(), Error>  {

    }

    pub fn visit_instance(&mut self, name: Token, constructor: Vec<Box<Node>>,
                          should_push: bool) -> Result<(), Error>  {

    }

    pub fn visit_assign(&mut self, previous: Option<Box<Node>>,
                        name: Token, value: Box<Node>) -> Result<(), Error>  {

    }

    pub fn visit_get(&mut self, previous: Option<Box<Node>>,
                     name: Token, should_push: bool) -> Result<(), Error>  {

    }
}