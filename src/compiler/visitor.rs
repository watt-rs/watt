// import
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::lexer::*;
use crate::parser::ast::*;
use crate::parser::import::Import;
use crate::resolver::resolver::ImportsResolver;
use crate::vm::bytecode::{Chunk, Opcode, OpcodeValue};
use crate::vm::values::*;
use std::collections::VecDeque;

/// Visitor
pub struct CompileVisitor<'visitor> {
    opcodes: VecDeque<Vec<Opcode>>,
    resolver: ImportsResolver<'visitor, 'visitor>,
}
/// Visitor implementation
#[allow(unused_variables)]
impl<'visitor> CompileVisitor<'visitor> {
    /// New visitor
    pub fn new() -> Self {
        CompileVisitor {
            opcodes: VecDeque::new(),
            resolver: ImportsResolver::new(),
        }
    }

    /// Visit built-in imports
    ///
    /// visits base.wt ast node.
    ///
    fn visit_builtins(&mut self) {
        let imports = self.resolver.import_builtins();
        for node in &imports {
            self.visit_node(node)
        }
    }

    /// Compile node
    pub unsafe fn compile(&mut self, node: &Node) -> Chunk {
        self.push_chunk();
        self.visit_builtins();
        self.visit_node(node);
        Chunk::new(self.pop_chunk())
    }

    /// Push chunk to opcodes chunk VecDeque
    ///
    /// raises error if compile-visitor stack is empty
    ///
    pub fn push_chunk(&mut self) {
        self.opcodes.push_front(vec![]);
    }

    /// Pop chunk from opcodes chunk VecDeque
    ///
    /// raises error if compile-visitor stack is empty
    ///
    pub fn pop_chunk(&mut self) -> Vec<Opcode> {
        match self.opcodes.pop_front() {
            Some(v) => v,
            None => panic!("couldn't pop from compiler-visitor stack. report to the developer."),
        }
    }

    /// Push instruction to last chunk
    ///
    /// raises error if compile-visitor stack is empty
    ///
    pub fn push_instr(&mut self, op: Opcode) {
        match self.opcodes.front_mut() {
            Some(v) => v.push(op),
            None => {
                panic!("couldn't push instr to compiler-visitor stack. report to the developer.")
            }
        }
    }

    /// Pop instruction from last chunk
    ///
    /// raises error if compile-visitor stack is empty
    ///
    pub fn pop_instr(&mut self) -> Opcode {
        match self.opcodes.front_mut() {
            Some(v) => v.pop().unwrap(),
            None => {
                panic!("couldn't pop instr from compiler-visitor stack. report to the developer.")
            }
        }
    }

    /// Visit node
    pub fn visit_node(&mut self, node: &Node) {
        match node {
            Node::Number { value } => {
                self.visit_number(value);
            }
            Node::String { value } => {
                self.visit_string(value);
            }
            Node::Bool { value } => {
                self.visit_bool(value);
            }
            Node::Bin { left, right, op } => {
                self.visit_binary(left, right, op);
            }
            Node::Unary { value, op } => {
                self.visit_unary(value, op);
            }
            Node::If {
                location,
                logical,
                body,
                elseif,
            } => {
                self.visit_if(location, logical, body, elseif.as_deref());
            }
            Node::While {
                location,
                logical,
                body,
            } => {
                self.visit_while(location, logical, body);
            }
            Node::Define {
                previous,
                name,
                value,
            } => {
                self.visit_define(previous.as_deref(), name, value);
            }
            Node::Assign {
                previous,
                name,
                value,
            } => {
                self.visit_assign(previous.as_deref(), name, value);
            }
            Node::Get {
                previous,
                name,
                should_push,
            } => {
                self.visit_get(previous.as_deref(), name, *should_push);
            }
            Node::Call {
                previous,
                name,
                args,
                should_push,
            } => {
                self.visit_call(previous.as_deref(), name, args, *should_push);
            }
            Node::FnDeclaration {
                name,
                full_name,
                params,
                body,
                make_closure,
            } => {
                self.visit_fn_decl(name, full_name, params, body, *make_closure);
            }
            Node::AnFnDeclaration {
                location,
                params,
                body,
                make_closure,
            } => {
                self.visit_an_fn_decl(location, params, body, *make_closure);
            }
            Node::Break { location } => {
                self.visit_break(location);
            }
            Node::Continue { location } => {
                self.visit_continue(location);
            }
            Node::Import { imports, .. } => {
                self.visit_import(imports);
            }
            Node::List { location, values } => {
                self.visit_list(location, values);
            }
            Node::Cond { left, right, op } => {
                self.visit_cond(left, right, op);
            }
            Node::Logical { left, right, op } => {
                self.visit_logical(left, right, op);
            }
            Node::Map { location, values } => {
                self.visit_map(location, values);
            }
            Node::Match {
                location,
                matchable,
                cases,
                default,
            } => {
                self.visit_match(location, matchable, cases, default);
            }
            Node::Native { name, fn_name } => {
                self.visit_native(name, fn_name);
            }
            Node::Instance {
                name,
                constructor,
                should_push,
            } => {
                self.visit_instance(name, constructor, *should_push);
            }
            Node::Ret { location, value } => {
                self.visit_return(location, value);
            }
            Node::Null { location } => {
                self.visit_null(location);
            }
            Node::Type {
                name,
                full_name,
                constructor,
                body,
                impls,
            } => {
                self.visit_type(name, full_name, constructor, body, impls);
            }
            Node::Unit {
                name,
                full_name,
                body,
            } => {
                self.visit_unit(name, full_name, body);
            }
            Node::For {
                iterable,
                variable_name,
                body,
            } => {
                self.visit_for(iterable, variable_name, body);
            }
            Node::Block { body } => {
                self.visit_block(body);
            }
            Node::Trait {
                name,
                full_name,
                functions,
            } => {
                self.visit_trait(name, full_name, functions);
            }
            Node::ErrorPropagation {
                location,
                value,
                should_push,
            } => {
                self.visit_error_propagation(location, value, *should_push);
            }
            Node::Impls { value, trait_name } => {
                self.visit_impls(value, trait_name);
            }
            Node::Range { location, from, to } => self.visit_range(location, from, to),
        }
    }

    /// Visit block
    fn visit_block(&mut self, body: &Vec<Node>) {
        for node in body {
            self.visit_node(node)
        }
    }

    /// Visit number
    fn visit_number(&mut self, value: &Token) {
        if value.value.contains(".") {
            self.push_instr(Opcode::Push {
                addr: value.address.clone(),
                value: OpcodeValue::Float(value.value.parse::<f64>().unwrap()),
            });
        } else {
            let parsed: i64 = {
                if value.value.starts_with("0x") {
                    i64::from_str_radix(&value.value[2..], 16).unwrap()
                } else if value.value.starts_with("0o") {
                    i64::from_str_radix(&value.value[2..], 8).unwrap()
                } else if value.value.starts_with("0b") {
                    i64::from_str_radix(&value.value[2..], 2).unwrap()
                } else {
                    value.value.parse::<i64>().unwrap()
                }
            };
            self.push_instr(Opcode::Push {
                addr: value.address.clone(),
                value: OpcodeValue::Int(parsed),
            });
        }
    }

    /// Visit string
    fn visit_string(&mut self, value: &Token) {
        self.push_instr(Opcode::Push {
            addr: value.address.clone(),
            value: OpcodeValue::String(value.value.clone()),
        });
    }

    /// Visit bool
    fn visit_bool(&mut self, value: &Token) {
        self.push_instr(Opcode::Push {
            addr: value.address.clone(),
            value: OpcodeValue::Bool(value.value.parse::<bool>().unwrap()),
        });
    }

    /// Visit binary operation
    fn visit_binary(&mut self, left: &Node, right: &Node, op: &Token) {
        self.visit_node(right);
        self.visit_node(left);
        self.push_instr(Opcode::Bin {
            addr: op.address.clone(),
            op: op.value.clone(),
        });
    }

    /// Visit if
    fn visit_if(&mut self, location: &Token, logical: &Node, body: &Node, elif: Option<&Node>) {
        // logical chunk
        self.push_chunk();
        self.visit_node(logical);
        let logical_chunk = self.pop_chunk();
        // body chunk
        self.push_chunk();
        self.visit_node(body);
        let body_chunk = self.pop_chunk();
        // elif
        let mut elseif: Option<Chunk> = None;
        if let Some(n) = elif {
            self.visit_node(n);
            elseif = Some(Chunk::of(self.pop_instr()));
        }
        // push if
        self.push_instr(Opcode::If {
            addr: location.address.clone(),
            cond: Chunk::new(logical_chunk),
            body: Chunk::new(body_chunk),
            elif: elseif,
        });
    }

    /// Visit while
    fn visit_while(&mut self, location: &Token, logical: &Node, body: &Node) {
        // logical chunk
        self.push_chunk();
        self.visit_node(logical);
        let logical_chunk = self.pop_chunk();
        // body chunk
        self.push_chunk();
        self.visit_node(body);
        let body_chunk = self.pop_chunk();
        // nested if opcode
        let if_opcode = Opcode::If {
            addr: location.address.clone(),
            cond: Chunk::new(logical_chunk),
            body: Chunk::new(body_chunk),
            elif: Some(Chunk::of(Opcode::If {
                addr: location.address.clone(),
                cond: Chunk::of(Opcode::Push {
                    addr: location.address.clone(),
                    value: OpcodeValue::Bool(true),
                }),
                body: Chunk::of(Opcode::EndLoop {
                    addr: location.address.clone(),
                    current_iteration: false,
                }),
                elif: None,
            })),
        };
        // push loop
        self.push_instr(Opcode::Loop {
            addr: location.address.clone(),
            body: Chunk::of(if_opcode),
        });
    }

    /// Define variable
    fn visit_define(&mut self, previous: Option<&Node>, name: &Token, value: &Node) {
        // previous
        let mut has_previous = false;
        if let Some(prev) = previous {
            self.visit_node(prev);
            has_previous = true;
        }
        // value chunk
        self.push_chunk();
        self.visit_node(value);
        let value_chunk = self.pop_chunk();
        // push define
        self.push_instr(Opcode::Define {
            addr: name.address.clone(),
            name: name.value.clone(),
            value: Chunk::new(value_chunk),
            has_previous,
        });
    }

    /// Visit call
    fn visit_call(
        &mut self,
        previous: Option<&Node>,
        name: &Token,
        args: &Vec<Node>,
        should_push: bool,
    ) {
        // previous
        let mut has_previous = false;
        if let Some(prev) = previous {
            self.visit_node(prev);
            has_previous = true;
        }
        // args chunk
        self.push_chunk();
        self.visit_block(args);
        let args_chunk = self.pop_chunk();
        // push call
        self.push_instr(Opcode::Call {
            addr: name.address.clone(),
            name: name.value.clone(),
            args: Chunk::new(args_chunk),
            has_previous,
            should_push,
        });
    }

    /// Visit fn declaration
    fn visit_fn_decl(
        &mut self,
        name: &Token,
        full_name: &Option<Token>,
        parameters: &Vec<Token>,
        body: &Node,
        make_closure: bool,
    ) {
        // full name
        let full_name = full_name.as_ref().map(|n| n.value.clone());
        // params
        let mut params = Vec::with_capacity(parameters.len());
        for param in parameters {
            params.push(param.value.clone());
        }
        // body chunk
        self.push_chunk();
        self.visit_node(body);
        // last fn `body` opcode
        let last_opcode = self.opcodes.front().and_then(|last| last.last());
        match last_opcode {
            // if it's a return
            Some(&Opcode::Ret { .. }) => {}
            // if not, creating default
            _ => {
                self.visit_node(&Node::Ret {
                    location: name.clone(),
                    value: Box::new(Node::Null {
                        location: name.clone(),
                    }),
                });
            }
        }
        // body chunk
        let chunk = self.pop_chunk();
        // push define fn
        self.push_instr(Opcode::DefineFn {
            addr: name.address.clone(),
            name: name.value.clone(),
            full_name,
            params,
            make_closure,
            body: Chunk::new(chunk),
        });
    }

    /// Visit break
    fn visit_break(&mut self, location: &Token) {
        self.push_instr(Opcode::EndLoop {
            addr: location.address.clone(),
            current_iteration: false,
        });
    }

    /// Visit continue
    fn visit_continue(&mut self, location: &Token) {
        self.push_instr(Opcode::EndLoop {
            addr: location.address.clone(),
            current_iteration: true,
        });
    }

    /// Visit import
    fn visit_import(&mut self, imports: &Vec<Import>) {
        for import in imports {
            let options_node = self.resolver.import(import.addr.clone(), import);
            if let Some(node) = &options_node {
                self.visit_node(node);
            }
        }
    }

    /// Visit list initializer
    fn visit_list(&mut self, location: &Token, list: &Vec<Node>) {
        // list
        self.push_instr(Opcode::Instance {
            addr: location.address.clone(),
            name: "List".to_string(),
            args: Chunk::new(vec![]),
            should_push: true,
        });
        // items
        if !(*list).is_empty() {
            for item in list {
                // duplicate list
                self.push_instr(Opcode::Duplicate {
                    addr: location.address.clone(),
                });
                self.push_chunk();
                // visit item
                self.visit_node(item);
                let chunk = self.pop_chunk();
                // calling add with element
                self.push_instr(Opcode::Call {
                    addr: location.address.clone(),
                    name: "add".to_string(),
                    args: Chunk::new(chunk),
                    has_previous: true,
                    should_push: false,
                })
            }
        }
    }

    /// Visit map
    fn visit_map(&mut self, location: &Token, map: &Vec<(Node, Node)>) {
        // map
        self.push_instr(Opcode::Instance {
            addr: location.address.clone(),
            name: "Map".to_string(),
            args: Chunk::new(vec![]),
            should_push: true,
        });
        // items
        if !(*map).is_empty() {
            for (k, v) in map {
                // duplicate map
                self.push_instr(Opcode::Duplicate {
                    addr: location.address.clone(),
                });
                // key and value
                self.push_chunk();
                self.visit_node(k);
                self.visit_node(v);
                let chunk = self.pop_chunk();
                // calling set with key and value
                self.push_instr(Opcode::Call {
                    addr: location.address.clone(),
                    name: "set".to_string(),
                    args: Chunk::new(chunk),
                    has_previous: true,
                    should_push: false,
                })
            }
        }
    }

    /// Visit for
    fn visit_for(&mut self, iterable: &Node, variable_name: &Token, body: &Node) {
        // todo: add iterable location
        // iterator chunk
        self.push_chunk();
        self.visit_node(iterable);
        let iterator_chunk = self.pop_chunk();
        // temp variable for iterator
        let iterator_variable_name = format!("@{}", variable_name.value);
        self.push_instr(Opcode::Define {
            addr: variable_name.address.clone(),
            name: iterator_variable_name.clone(),
            value: Chunk::new(iterator_chunk),
            has_previous: false,
        });
        // body chunk
        self.push_chunk();
        self.push_instr(Opcode::Define {
            addr: variable_name.address.clone(),
            name: variable_name.value.clone(),
            value: Chunk::new(vec![
                Opcode::Load {
                    addr: variable_name.address.clone(),
                    name: iterator_variable_name.clone(),
                    has_previous: false,
                    should_push: true,
                },
                Opcode::Call {
                    addr: variable_name.address.clone(),
                    name: "next".to_string(),
                    args: Chunk::new(vec![]),
                    has_previous: true,
                    should_push: true,
                },
            ]),
            has_previous: false,
        });
        self.visit_node(body);
        let body_chunk = self.pop_chunk();
        // if chunk
        self.push_chunk();
        self.push_instr(Opcode::If {
            addr: variable_name.address.clone(),
            cond: Chunk::new(vec![
                Opcode::Load {
                    addr: variable_name.address.clone(),
                    name: iterator_variable_name.clone(),
                    has_previous: false,
                    should_push: true,
                },
                Opcode::Call {
                    addr: variable_name.address.clone(),
                    name: "has_next".to_string(),
                    args: Chunk::new(vec![]),
                    has_previous: true,
                    should_push: true,
                },
            ]),
            body: Chunk::new(body_chunk),
            elif: Some(Chunk::of(Opcode::EndLoop {
                addr: variable_name.address.clone(),
                current_iteration: false,
            })),
        });
        let if_chunk = self.pop_chunk();
        // push loop
        self.push_instr(Opcode::Loop {
            addr: variable_name.address.clone(),
            body: Chunk::new(if_chunk),
        });
        // delete temp variable
        self.push_instr(Opcode::DeleteLocal {
            addr: variable_name.address.clone(),
            name: iterator_variable_name,
        })
    }

    /// Visit match
    fn visit_match(
        &mut self,
        location: &Token,
        matchable: &Node,
        cases: &Vec<MatchCase>,
        default: &Node,
    ) {
        // result opcode
        let mut if_op;
        // default case
        self.push_chunk();
        self.visit_node(default);
        let body_chunk = self.pop_chunk();
        if_op = Opcode::If {
            addr: location.address.clone(),
            cond: Chunk::of(Opcode::Push {
                addr: location.address.clone(),
                value: OpcodeValue::Bool(true),
            }),
            body: Chunk::new(body_chunk),
            elif: None,
        };
        // compiling cases
        for case in cases {
            // logic chunk
            self.push_chunk();
            self.visit_node(&case.value);
            self.visit_node(matchable);
            self.push_instr(Opcode::Cond {
                addr: location.address.clone(),
                op: "==".to_string(),
            });
            let logical_chunk = self.pop_chunk();

            // body chunk
            self.push_chunk();
            self.visit_node(&case.body);
            let body_chunk = self.pop_chunk();

            if_op = Opcode::If {
                addr: location.address.clone(),
                cond: Chunk::new(logical_chunk),
                body: Chunk::new(body_chunk),
                elif: Some(Chunk::of(if_op)),
            }
        }

        // push if
        self.push_instr(if_op);
    }

    /// Visit anonymous fn declaration
    fn visit_an_fn_decl(
        &mut self,
        location: &Token,
        parameters: &Vec<Token>,
        body: &Node,
        make_closure: bool,
    ) {
        // params
        let mut params = Vec::new();
        for param in parameters {
            params.push(param.value.clone());
        }
        // body chunk
        self.push_chunk();
        self.visit_node(body);
        // last fn `body` opcode
        let last_opcode = self.opcodes.front().and_then(|last| last.last());
        match last_opcode {
            // if it's a return
            Some(&Opcode::Ret { .. }) => {}
            // if not, creating default
            _ => {
                self.visit_node(&Node::Ret {
                    location: location.clone(),
                    value: Box::new(Node::Null {
                        location: location.clone(),
                    }),
                });
            }
        }
        // получаем чанк тела
        let chunk = self.pop_chunk();
        // создание анонимной функции
        self.push_instr(Opcode::AnonymousFn {
            addr: location.address.clone(),
            params,
            make_closure,
            body: Chunk::new(chunk),
        });
    }

    /// Visit native
    fn visit_native(&mut self, name: &Token, fn_name: &Token) {
        self.push_instr(Opcode::Define {
            addr: name.address.clone(),
            name: name.value.clone(),
            value: Chunk::of(Opcode::Native {
                addr: fn_name.address.clone(),
                fn_name: fn_name.value.clone(),
            }),
            has_previous: false,
        });
    }

    /// Visit unary
    fn visit_unary(&mut self, value: &Node, op: &Token) {
        self.visit_node(value);
        match op.value.as_str() {
            // negate operator
            "-" => self.push_instr(Opcode::Neg {
                addr: op.address.clone(),
            }),
            // bang operator
            "!" => self.push_instr(Opcode::Bang {
                addr: op.address.clone(),
            }),
            _ => {
                error!(Error::own_text(
                    op.address.clone(),
                    format!("undefined unary op: {:?}", op.value),
                    "available: -, !",
                ))
            }
        }
    }

    /// Visit type
    fn visit_type(
        &mut self,
        name: &Token,
        full_name: &Option<Token>,
        constructor: &Vec<Token>,
        body: &Node,
        impl_tokens: &Vec<Token>,
    ) {
        // full name
        let full_name = full_name.as_ref().map(|name| name.value.clone());
        // constructor
        let mut constructor_params = Vec::new();
        for param in constructor {
            constructor_params.push(param.value.clone());
        }
        // body chunk
        self.push_chunk();
        self.visit_node(body);
        let chunk = self.pop_chunk();
        // trait impls
        let mut impls = Vec::with_capacity(impl_tokens.len());
        for i in impl_tokens {
            impls.push(i.value.clone())
        }
        // push define type
        self.push_instr(Opcode::DefineType {
            addr: name.address.clone(),
            name: name.value.clone(),
            full_name,
            constructor: constructor_params,
            body: Chunk::new(chunk),
            impls,
        });
    }

    /// Visit trait
    fn visit_trait(&mut self, name: &Token, full_name: &Option<Token>, functions: &[TraitNodeFn]) {
        // full name
        let full_name = full_name.as_ref().map(|name| name.value.clone());
        // trait functions
        let mut trait_functions: Vec<TraitFn> = Vec::new();
        for node_fn in functions {
            // default
            let default: Option<DefaultTraitFn> = if node_fn.default.is_some() {
                // body chunk and params
                self.push_chunk();
                self.visit_node(node_fn.default.as_ref().unwrap());
                let chunk = Chunk::new(self.pop_chunk());
                let params: Vec<String> = node_fn
                    .params
                    .iter()
                    .map(|param| param.value.clone())
                    .collect();

                // setting default
                Some(DefaultTraitFn::new(params, chunk))
            } else {
                // setting default
                None
            };

            trait_functions.push(TraitFn::new(
                node_fn.name.value.clone(),
                node_fn.params.len(),
                default,
            ))
        }
        // push define trait
        self.push_instr(Opcode::DefineTrait {
            addr: name.address.clone(),
            name: name.value.clone(),
            full_name,
            functions: trait_functions,
        });
    }

    /// Visit unit
    fn visit_unit(&mut self, name: &Token, full_name: &Option<Token>, body: &Node) {
        // full name
        let full_name = full_name.as_ref().map(|name| name.value.clone());
        // body chunk
        self.push_chunk();
        self.visit_node(body);
        let chunk = self.pop_chunk();
        // push define unit
        self.push_instr(Opcode::DefineUnit {
            addr: name.address.clone(),
            name: name.value.clone(),
            full_name,
            body: Chunk::new(chunk),
        });
    }

    /// Visit condition
    fn visit_cond(&mut self, left: &Node, right: &Node, op: &Token) {
        self.visit_node(right);
        self.visit_node(left);
        self.push_instr(Opcode::Cond {
            addr: op.address.clone(),
            op: op.value.clone(),
        });
    }

    /// Visit logical
    fn visit_logical(&mut self, left: &Node, right: &Node, op: &Token) {
        self.push_chunk();
        self.visit_node(left);
        let a = Chunk::new(self.pop_chunk());

        self.push_chunk();
        self.visit_node(right);
        let b = Chunk::new(self.pop_chunk());

        self.push_instr(Opcode::Logic {
            addr: op.address.clone(),
            a,
            b,
            op: op.value.clone(),
        });
    }

    /// Visit return
    fn visit_return(&mut self, location: &Token, value: &Node) {
        self.push_chunk();
        self.visit_node(value);
        let chunk = self.pop_chunk();

        self.push_instr(Opcode::Ret {
            addr: location.address.clone(),
            value: Chunk::new(chunk),
        });
    }

    /// Visit null
    fn visit_null(&mut self, location: &Token) {
        self.push_instr(Opcode::Push {
            addr: location.address.clone(),
            value: OpcodeValue::Raw(Value::Null),
        });
    }

    /// Visit instance
    fn visit_instance(&mut self, name: &Token, constructor: &Vec<Node>, should_push: bool) {
        // constructor
        self.push_chunk();
        for arg in constructor {
            self.visit_node(arg);
        }
        let constructor_args = self.pop_chunk();
        // instance
        self.push_instr(Opcode::Instance {
            addr: name.address.clone(),
            name: name.value.clone(),
            args: Chunk::new(constructor_args),
            should_push,
        });
    }

    /// Visit assign
    fn visit_assign(&mut self, previous: Option<&Node>, name: &Token, value: &Node) {
        // previous
        let mut has_previous = false;
        if let Some(prev) = &previous {
            self.visit_node(prev);
            has_previous = true;
        }
        // value chunk
        self.push_chunk();
        self.visit_node(value);
        let chunk = self.pop_chunk();
        // push set
        self.push_instr(Opcode::Set {
            addr: name.address.clone(),
            name: name.value.clone(),
            value: Chunk::new(chunk),
            has_previous,
        });
    }

    /// Visit get
    fn visit_get(&mut self, previous: Option<&Node>, name: &Token, should_push: bool) {
        // previous
        let mut has_previous = false;
        if let Some(prev) = previous {
            self.visit_node(prev);
            has_previous = true;
        }
        // push load
        self.push_instr(Opcode::Load {
            addr: name.address.clone(),
            name: name.value.clone(),
            has_previous,
            should_push,
        });
    }

    /// Visit error propagation
    fn visit_error_propagation(&mut self, location: &Token, value: &Node, should_push: bool) {
        self.push_chunk();
        self.visit_node(value);
        let chunk = self.pop_chunk();

        self.push_instr(Opcode::ErrorPropagation {
            addr: location.address.clone(),
            value: Chunk::new(chunk),
            should_push,
        });
    }

    /// Visit impls trait
    pub fn visit_impls(&mut self, value: &Node, trait_name: &Token) {
        self.push_chunk();
        self.visit_node(value);
        let chunk = self.pop_chunk();

        self.push_instr(Opcode::Impls {
            addr: trait_name.address.clone(),
            value: Chunk::new(chunk),
            trait_name: trait_name.value.clone(),
        })
    }

    /// Visit range
    fn visit_range(&mut self, location: &Token, from: &Node, to: &Node) {
        // range call args
        self.push_chunk();
        self.visit_node(from);
        self.visit_node(to);
        let chunk = self.pop_chunk();
        // range call
        self.push_instr(Opcode::Call {
            addr: location.address.clone(),
            name: "_range".to_string(),
            args: Chunk::new(chunk),
            has_previous: false,
            should_push: true,
        })
    }
}
