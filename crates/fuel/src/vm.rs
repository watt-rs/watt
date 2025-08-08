/// Imports
use crate::bytecode::{Chunk, ModuleInfo, Opcode, OpcodeValue};
use crate::call_stack::environment::Environment;
use crate::call_stack::frame::CallFrame;
use crate::flow::ControlFlow;
use crate::memory::gc::Gc;
use crate::{gc_freeze, guard, natives::natives, root, unguard, values::*};
use oil_common::address::Address;
use oil_common::{error, errors::Error};
use rustc_hash::FxHashMap;
use scopeguard::defer;

/// Virtual machine
///
/// Vm that runs opcodes 🤔
///
#[derive(Debug)]
pub struct VirtualMachine {
    pub builtins_table: Gc<Environment>,
    pub natives_table: Gc<Environment>,
    pub operand_stack: Vec<Value>,
    pub modules_info: FxHashMap<usize, ModuleInfo>,
    pub modules: FxHashMap<usize, Gc<Module>>,
    pub call_stack: Vec<Gc<CallFrame>>,
    pub module_stack: Vec<Gc<Module>>,
}

/// Vm implementation
#[allow(unsafe_op_in_unsafe_fn)]
impl VirtualMachine {
    /// New vm
    pub unsafe fn new(
        builtins: Chunk,
        modules_info: FxHashMap<usize, ModuleInfo>,
    ) -> VirtualMachine {
        // vm
        let mut vm = VirtualMachine {
            builtins_table: Gc::new(Environment::new()),
            natives_table: Gc::new(Environment::new()),
            operand_stack: Vec::new(),
            modules_info,
            modules: FxHashMap::default(),
            call_stack: Vec::new(),
            module_stack: Vec::new(),
        };
        root!(vm.natives_table);
        // freezing gc
        gc_freeze!();
        // natives
        if let Err(e) = natives::provide_natives(&mut vm) {
            error!(e)
        }
        // running builtins
        vm.push_frame();
        // pushing environment
        vm.peek_frame().push(Gc::new(Environment::new()));
        if let Err(e) = vm.run(&builtins) {
            error!(Error::own_text(
                Address::unknown(),
                format!("control flow leak: {e:?}"),
                "report this error to the developer."
            ));
        }
        vm.builtins_table = vm.pop_frame().pop();
        root!(vm.builtins_table);
        // unfreezing gc
        gc_freeze!();
        // returning vm
        vm
    }

    /// Pushes frame to vm call stack
    pub fn push_frame(&mut self) {
        let frame = Gc::new(CallFrame::new());
        guard!(frame);
        self.call_stack.push(frame);
        self.peek_frame().push(Gc::new(Environment::new()));
    }

    /// Pushes frame with closure to vm call stack
    pub fn push_frame_with_closure(&mut self, environment: Gc<Environment>) {
        let frame = Gc::new(CallFrame::with_closure(environment));
        guard!(frame);
        self.call_stack.push(frame);
        self.peek_frame().push(Gc::new(Environment::new()));
    }

    /// Pops frame from vm call stack
    pub fn pop_frame(&mut self) -> Gc<CallFrame> {
        match self.call_stack.pop() {
            Some(frame) => {
                unguard!(frame);
                frame
            }
            None => panic!("call stack is empty. report this error to the developer."),
        }
    }

    /// Peeks vm frame
    pub fn peek_frame(&self) -> Gc<CallFrame> {
        match self.call_stack.last() {
            Some(frame) => frame.clone(),
            None => panic!("call stack is empty. report this error to the developer."),
        }
    }

    /// Pushes module to vm module stack
    pub fn push_module(&mut self, module: Gc<Module>) {
        guard!(module);
        self.module_stack.push(module);
    }

    /// Pops module from vm module stack
    pub fn pop_module(&mut self) -> Gc<Module> {
        match self.module_stack.pop() {
            Some(moudle) => {
                unguard!(moudle);
                moudle
            }
            None => panic!("call stack is empty. report this error to the developer."),
        }
    }

    /// Peeks vm module
    pub fn peek_module(&self) -> Gc<Module> {
        match self.module_stack.last() {
            Some(module) => module.clone(),
            None => panic!("call stack is empty. report this error to the developer."),
        }
    }

    /// Pushes value to vm stack
    pub unsafe fn push(&mut self, value: Value) {
        self.operand_stack.push(value);
    }

    /// Popes value from vm stack
    pub fn pop(&mut self, address: &Address) -> Value {
        if self.operand_stack.is_empty() {
            error!(Error::new(
                address.clone(),
                "stack underflow.",
                "check your code."
            ));
        }
        self.operand_stack.pop().unwrap()
    }

    /// Defines variable in last environment, in last frame
    pub fn define_variable(&mut self, address: &Address, name: &str, value: Value) {
        self.peek_frame().define(address, name, value);
    }

    /// Stores variable in last environment, in last frame
    pub fn store_variable(&mut self, address: &Address, name: &str, value: Value) {
        self.peek_frame().store(address, name, value);
    }

    /// Loads variable from last environment, in last frame
    pub fn load_variable(&self, address: &Address, name: &str) -> Value {
        if self.peek_frame().is_exists(name) {
            self.peek_frame().load(address, name)
        } else {
            self.builtins_table.load(address, name)
        }
    }

    /// Deletes variable in last environment, in last frame
    pub fn delete_variable(&mut self, address: &Address, name: &str) {
        self.peek_frame().delete(address, name);
    }

    /// Opcode: Push value to vm stack
    ///
    /// if value is a reference type except
    /// `trait` and `type` it will be registered in gc
    /// *safely* from *freeing* before the *self.push*
    /// itself executes.
    ///
    /// safety guaranteed by pushing value to stack
    /// before registering in gc.
    ///
    pub unsafe fn op_push(&mut self, value: OpcodeValue) -> Result<(), ControlFlow> {
        // checking value type
        match value {
            // primitives
            OpcodeValue::Int(int) => {
                self.push(Value::Int(int));
            }
            OpcodeValue::Float(float) => {
                self.push(Value::Float(float));
            }
            OpcodeValue::Bool(bool) => {
                self.push(Value::Bool(bool));
            }
            // string
            OpcodeValue::String(string) => {
                // allocating string
                let new_string = Value::String(Gc::new(string));
                // pushing string value to stack
                self.push(new_string);
            }
            // raw
            OpcodeValue::Raw(raw) => {
                match raw {
                    Value::Instance(_)
                    | Value::Fn(_)
                    | Value::Native(_)
                    | Value::String(_)
                    | Value::Unit(_)
                    | Value::List(_)
                    | Value::Any(_) => {
                        // push
                        self.push(raw);
                    }
                    _ => {
                        // push
                        self.push(raw);
                    }
                }
            }
        }
        Ok(())
    }

    /// Opcode: Binary operation
    unsafe fn op_binary(&mut self, address: &Address, op: &str) -> Result<(), ControlFlow> {
        // operands
        let operand_a = self.pop(address);
        let operand_b = self.pop(address);

        // error generators
        let invalid_op_error = Error::own_text(
            address.clone(),
            format!("could not use '{op}' with {operand_a:?} and {operand_b:?}"),
            "check your code.",
        );
        let division_error = || {
            error!(Error::new(
                address.clone(),
                "division by zero.",
                "undefined operation."
            ));
        };

        // concat
        let concat = |mut string: String, a, b| {
            string.push_str(a);
            string.push_str(b);
            Value::String(Gc::new(string))
        };

        // binary operators
        match op {
            "+" => match operand_a {
                Value::Float(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Float(a + b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Float(a + (b as f64)));
                    }
                    Value::String(b) => {
                        let string = concat(String::with_capacity(b.len()), &a.to_string(), &*b);
                        self.push(string);
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                Value::Int(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Float((a as f64) + b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Int(a + b));
                    }
                    Value::String(b) => {
                        let string = concat(String::with_capacity(b.len()), &a.to_string(), &*b);
                        self.push(string);
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                Value::String(a) => match operand_b {
                    Value::String(b) => {
                        let string = concat(String::with_capacity(a.len() + b.len()), &*a, &*b);
                        self.push(string);
                    }
                    _ => {
                        let string =
                            concat(String::with_capacity(a.len()), &*a, &operand_b.to_string());
                        self.push(string);
                    }
                },
                _ => {
                    if let Value::String(b) = &operand_b {
                        let string = concat(
                            String::with_capacity(b.len()),
                            &operand_a.to_string(),
                            &operand_b.to_string(),
                        );
                        self.push(string);
                    } else {
                        error!(invalid_op_error);
                    }
                }
            },
            "-" => match operand_a {
                Value::Float(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Float(a - b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Float(a - (b as f64)));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                Value::Int(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Float((a as f64) - b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Int(a - b));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                _ => {
                    error!(invalid_op_error);
                }
            },
            "*" => match operand_a {
                Value::Float(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Float(a * b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Float(a * (b as f64)));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                Value::Int(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Float((a as f64) * b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Int(a * b));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                _ => {
                    error!(invalid_op_error);
                }
            },
            "/" => {
                match operand_a {
                    Value::Float(a) => {
                        match operand_b {
                            Value::Float(b) => {
                                // checking division by zero
                                if b == 0f64 {
                                    division_error();
                                }
                                // dividing
                                self.push(Value::Float(a / b));
                            }
                            Value::Int(b) => {
                                // checking division by zero
                                if b == 0 {
                                    division_error();
                                }
                                // dividing
                                self.push(Value::Float(a / (b as f64)));
                            }
                            _ => {
                                error!(invalid_op_error);
                            }
                        }
                    }
                    Value::Int(a) => {
                        match operand_b {
                            Value::Float(b) => {
                                // checking division by zero
                                if b == 0f64 {
                                    division_error();
                                }
                                // dividing
                                self.push(Value::Float((a as f64) / b));
                            }
                            Value::Int(b) => {
                                // checking division by zero
                                if b == 0 {
                                    division_error();
                                }
                                // dividing
                                if a % b == 0 {
                                    self.push(Value::Int(a / b));
                                } else {
                                    self.push(Value::Float(a as f64 / b as f64))
                                }
                            }
                            _ => {
                                error!(invalid_op_error);
                            }
                        }
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                }
            }
            "%" => match operand_a {
                Value::Float(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Float(a % b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Float(a % (b as f64)));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                Value::Int(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Float((a as f64) % b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Int(a % b));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                _ => {
                    error!(invalid_op_error);
                }
            },
            "&" => match operand_a {
                Value::Int(a) => match operand_b {
                    Value::Int(b) => {
                        self.push(Value::Int(a & b));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                _ => {
                    error!(invalid_op_error);
                }
            },
            "|" => match operand_a {
                Value::Int(a) => match operand_b {
                    Value::Int(b) => {
                        self.push(Value::Int(a | b));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                _ => {
                    error!(invalid_op_error);
                }
            },
            "^" => match operand_a {
                Value::Int(a) => match operand_b {
                    Value::Int(b) => {
                        self.push(Value::Int(a ^ b));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                _ => {
                    error!(invalid_op_error);
                }
            },
            _ => {
                panic!("operator = {op} is not found.")
            }
        }
        Ok(())
    }

    /// Opcode: Negate operation
    unsafe fn op_negate(&mut self, address: &Address) -> Result<(), ControlFlow> {
        // operand
        let operand = self.pop(address);
        // negate
        match operand {
            Value::Float(a) => {
                self.push(Value::Float(-a));
            }
            Value::Int(a) => {
                self.push(Value::Int(-a));
            }
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not use 'negate' for {operand:?}"),
                    "check your code."
                ));
            }
        }
        Ok(())
    }

    /// Opcode: Bang operation
    unsafe fn op_bang(&mut self, address: &Address) -> Result<(), ControlFlow> {
        // operand
        let operand = self.pop(address);
        // bang
        match operand {
            Value::Bool(b) => {
                self.push(Value::Bool(!b));
            }
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not use 'bang' for {operand:?}"),
                    "check your code."
                ));
            }
        }
        Ok(())
    }

    /// Opcode: Conditional operation
    unsafe fn op_conditional(&mut self, address: &Address, op: &str) -> Result<(), ControlFlow> {
        // operands
        let operand_a = self.pop(address);
        let operand_b = self.pop(address);
        // error
        let invalid_op_error = Error::own_text(
            address.clone(),
            format!("could not use '{op}' for {operand_a:?} and {operand_b:?}"),
            "check your code.",
        );
        // conditional op
        match op {
            ">" => match operand_a {
                Value::Float(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Bool(a > b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Bool(a > (b as f64)));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                Value::Int(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Bool((a as f64) > b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Bool(a > b));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                Value::String(a) => match operand_b {
                    Value::String(b) => {
                        self.push(Value::Bool(*a > *b));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                _ => {
                    error!(invalid_op_error);
                }
            },
            "<" => match operand_a {
                Value::Float(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Bool(a < b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Bool(a < (b as f64)));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                Value::Int(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Bool((a as f64) < b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Bool(a < b));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                Value::String(a) => match operand_b {
                    Value::String(b) => {
                        self.push(Value::Bool(*a < *b));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                _ => {
                    error!(invalid_op_error);
                }
            },
            ">=" => match operand_a {
                Value::Float(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Bool(a >= b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Bool(a >= (b as f64)));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                Value::Int(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Bool((a as f64) >= b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Bool(a >= b));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                Value::String(a) => match operand_b {
                    Value::String(b) => {
                        self.push(Value::Bool(*a >= *b));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                _ => {
                    error!(invalid_op_error);
                }
            },
            "<=" => match operand_a {
                Value::Float(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Bool(a <= b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Bool(a <= (b as f64)));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                Value::Int(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Bool((a as f64) <= b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Bool(a <= b));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                Value::String(a) => match operand_b {
                    Value::String(b) => {
                        self.push(Value::Bool(*a <= *b));
                    }
                    _ => {
                        error!(invalid_op_error);
                    }
                },
                _ => {
                    error!(invalid_op_error);
                }
            },
            "==" => match operand_a {
                Value::Float(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Bool(a == b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Bool(a == (b as f64)));
                    }
                    _ => {
                        self.push(Value::Bool(false));
                    }
                },
                Value::Int(a) => match operand_b {
                    Value::Float(b) => {
                        self.push(Value::Bool((a as f64) == b));
                    }
                    Value::Int(b) => {
                        self.push(Value::Bool(a == b));
                    }
                    _ => {
                        self.push(Value::Bool(false));
                    }
                },
                Value::Null => match operand_b {
                    Value::Null => {
                        self.push(Value::Bool(true));
                    }
                    _ => {
                        self.push(Value::Bool(false));
                    }
                },
                Value::Fn(f1) => match operand_b {
                    Value::Fn(f2) => {
                        self.push(Value::Bool(f1 == f2));
                    }
                    _ => {
                        self.push(Value::Bool(false));
                    }
                },
                Value::Bool(a) => match operand_b {
                    Value::Bool(b) => {
                        self.push(Value::Bool(a == b));
                    }
                    _ => {
                        self.push(Value::Bool(false));
                    }
                },
                Value::Instance(a) => match operand_b {
                    Value::Instance(b) => {
                        self.push(Value::Bool(a == b));
                    }
                    _ => {
                        self.push(Value::Bool(false));
                    }
                },
                Value::Type(a) => match operand_b {
                    Value::Type(b) => self.push(Value::Bool(a == b)),
                    _ => {
                        self.push(Value::Bool(false));
                    }
                },
                Value::String(a) => match operand_b {
                    Value::String(b) => self.push(Value::Bool(*a == *b)),
                    _ => {
                        self.push(Value::Bool(false));
                    }
                },
                Value::Native(a) => match operand_b {
                    Value::Native(b) => self.push(Value::Bool(a == b)),
                    _ => {
                        self.push(Value::Bool(false));
                    }
                },
                Value::Trait(a) => match operand_b {
                    Value::Trait(b) => self.push(Value::Bool(a == b)),
                    _ => {
                        self.push(Value::Bool(false));
                    }
                },
                _ => {
                    self.push(Value::Bool(false));
                }
            },
            "!=" => {
                // temp operands
                self.push(operand_b);
                self.push(operand_a);
                // running equals cond op
                self.op_conditional(address, "==")?;
                // running bang
                self.op_bang(address)?;
            }
            _ => {
                panic!("operator {op} is not found. report this error to the developer.")
            }
        }
        Ok(())
    }

    /// Opcode: Logical operator with short circuit
    unsafe fn op_logical(
        &mut self,
        address: &Address,
        a: &Chunk,
        b: &Chunk,
        op: &str,
    ) -> Result<(), ControlFlow> {
        // expect bool
        fn expect_bool(value: Value, error: Error) -> bool {
            match value {
                Value::Bool(b) => b,
                _ => {
                    error!(error)
                }
            }
        }

        // running first chunk
        self.run(a)?;
        let operand_a = self.pop(address);

        // operand a
        let a = expect_bool(
            operand_a.clone(),
            Error::own_text(
                address.clone(),
                format!("could not use '{op}' with {operand_a:?}"),
                "check your code.",
            ),
        );

        // logical op
        match op {
            "and" => {
                // if operand_a already pushed false, we shouldn't eval right chunk
                if !a {
                    self.push(Value::Bool(false));
                }
                // if operand_a pushed true
                else {
                    // evaluating second chunk
                    self.run(b)?;
                    let operand_b = self.pop(address);
                    // operand b
                    let b = expect_bool(
                        operand_b.clone(),
                        Error::own_text(
                            address.clone(),
                            format!("could not use '{op}' for {operand_a:?} and {operand_b:?}"),
                            "check your code.",
                        ),
                    );
                    // pushing result
                    self.push(Value::Bool(b));
                }
            }
            "or" => {
                // if operand_a already pushed false, we shouldn't eval right chunk
                if a {
                    self.push(Value::Bool(true));
                }
                // if operand_a pushed true
                else {
                    // evaluating second chunk
                    self.run(b)?;
                    let operand_b = self.pop(address);
                    // operand b
                    let b = expect_bool(
                        operand_b.clone(),
                        Error::own_text(
                            address.clone(),
                            format!("could not use '{op}' for {operand_a:?} and {operand_b:?}"),
                            "check your code.",
                        ),
                    );
                    // pushing result
                    self.push(Value::Bool(b));
                }
            }
            _ => {
                panic!("operator {op} is not found.")
            }
        }

        Ok(())
    }

    /// Opcode: If
    unsafe fn op_if(
        &mut self,
        addr: &Address,
        cond: &Chunk,
        body: &Chunk,
        elif: &Option<Chunk>,
    ) -> Result<(), ControlFlow> {
        // getting frame
        let mut call_frame = self.peek_frame();

        // pushing environment
        call_frame.push(Gc::new(Environment::new()));

        // popping environment
        defer! {
            call_frame.pop();
        }

        // running condition
        self.run(cond)?;
        let bool = self.pop(addr);

        // checking condition returned true
        if let Value::Bool(b) = bool {
            if b {
                self.run(body)?
            } else if let Option::Some(else_if) = elif {
                self.run(else_if)?
            }
        } else {
            error!(Error::own_text(
                addr.clone(),
                format!("condition provided not a bool: {bool:?}"),
                "condition should provide a bool."
            ))
        }

        Ok(())
    }

    /// Opcode: Loop
    #[allow(unused_variables)]
    unsafe fn op_loop(&mut self, addr: &Address, body: &Chunk) -> Result<(), ControlFlow> {
        // getting frame
        let mut call_frame = self.peek_frame();

        // pushing environment
        call_frame.push(Gc::new(Environment::new()));

        // popping environment
        defer! {
            call_frame.pop();
        }

        // loop
        loop {
            if let Err(e) = self.run(body) {
                match e {
                    ControlFlow::Continue => {
                        continue;
                    }
                    ControlFlow::Break => {
                        break;
                    }
                    _ => {
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Opcode: DefineFn
    ///
    /// defines fn in local table
    /// with `safely` allocating it
    ///
    /// safety guaranteed by using gc_guard
    /// before registering in gc, and gc_unguard after
    /// registering in gc
    ///
    unsafe fn op_define_fn(
        &mut self,
        addr: &Address,
        name: &String,
        body: &Chunk,
        params: &[String],
    ) -> Result<(), ControlFlow> {
        // allocating function
        let function = Gc::new(Function::new(
            name.clone(),
            Gc::new(body.clone()),
            params.to_owned(),
            self.peek_module(),
            self.peek_frame().peek(),
        ));

        // function value
        let function_value = Value::Fn(function);

        // defining fn by name and full name
        self.define_variable(addr, name, function_value);

        Ok(())
    }

    /// Opcode: AnonymousFn
    ///
    /// pushing fn to stack by safely` allocating it
    ///
    /// safety guaranteed by pushing value to stack
    /// before registering in gc.
    ///
    unsafe fn op_anonymous_fn(
        &mut self,
        body: &Chunk,
        params: &[String],
    ) -> Result<(), ControlFlow> {
        // allocating function
        let function = Gc::new(Function::new(
            "$lambda".to_string(),
            Gc::new(body.clone()),
            params.to_owned(),
            self.peek_module(),
            self.peek_frame().peek(),
        ));

        // function value
        let function_value = Value::Fn(function);

        // push function value to stack
        self.push(function_value);

        Ok(())
    }

    /// Bind functions
    ///
    /// Goes through the table fields,
    /// search functions and then binds owner
    /// to them.
    unsafe fn bind_functions(&mut self, mut environment: Gc<Environment>, owner: Gc<FnOwner>) {
        for val in environment.variables.values_mut() {
            if let Value::Fn(function) = val {
                function.owner = Some(owner.clone());
            }
        }
    }

    /// Opcode: DefineType
    ///
    /// defines type in `self.types` table
    /// with `safely` allocating it
    ///
    /// safety guaranteed because gc bears
    /// no responsibility to control types,
    /// they will be freed in `self.cleanup()`
    ///
    unsafe fn op_define_type(
        &mut self,
        addr: &Address,
        name: &String,
        body: &Chunk,
        constructor: &[String],
        _impls: &[Chunk],
    ) -> Result<(), ControlFlow> {
        // allocating type
        let t = Gc::new(Type::new(
            name.clone(),
            constructor.to_owned(),
            Gc::new(body.clone()),
            vec![], // todo: impls.to_owned(),
            self.peek_module(),
        ));

        // defining type by name
        self.define_variable(addr, name.as_str(), Value::Type(t));
        Ok(())
    }

    /// Opcode: DefineUnit
    ///
    /// defines type in `self.types` table
    /// with `safely` allocating it
    ///
    /// safety guaranteed by using gc_guard
    /// before registering in gc, and gc_unguard after
    /// registering in gc
    ///
    unsafe fn op_define_unit(
        &mut self,
        addr: &Address,
        name: &String,
        body: &Chunk,
    ) -> Result<(), ControlFlow> {
        // pushing frame
        self.push_frame();

        // executing body
        self.run(body)?;

        // popping environment
        let environment = self.pop_frame().pop();

        // allocating unit
        let unit = Gc::new(Unit::new(name.clone(), environment, self.peek_module()));

        // guarding
        guard!(unit);

        // unit value
        let unit_value = Value::Unit(unit.clone());

        // binding function
        self.bind_functions(
            unit.environment.clone(),
            Gc::new(FnOwner::Unit(unit.clone())),
        );

        // calling optional init fn
        let init_fn = "init";
        if unit.environment.is_exists(init_fn) {
            // pushing frame
            self.push_frame();
            // calling `init`
            self.push(unit_value.clone());
            if let Err(e) = self.op_call(addr, init_fn, true, false, &Chunk::new(vec![])) {
                panic!(
                    "`init` fn returned a {:?}. report error to the developer",
                    e
                )
            }
            // popping frame
            self.pop_frame();
        }

        // unguard
        unguard!(unit);

        // defining unit by name and full name
        self.define_variable(addr, name.as_str(), unit_value);

        Ok(())
    }

    /// Opcode: DefineTrait
    ///
    /// defines trait in `self.traits` table
    /// with `safely` allocating it
    ///
    /// safety guaranteed because gc bears
    /// no responsibility to control types,
    /// they will be freed in `self.cleanup()`
    ///
    unsafe fn op_define_trait(
        &mut self,
        addr: &Address,
        name: &String,
        functions: &[TraitFn],
    ) -> Result<(), ControlFlow> {
        // allocating trait
        let _trait = Gc::new(Trait::new(
            name.clone(),
            functions.to_owned(),
            self.peek_module(),
        ));

        // define trait by name
        self.define_variable(addr, name.as_str(), Value::Trait(_trait));

        Ok(())
    }

    /// Opcode: Define
    ///
    /// defines value in local table
    /// or, if `has_previous` pops
    /// value (instance/unit, otherwise raises error)
    /// from stack and then defines given
    /// value in it by name
    ///
    unsafe fn op_define(
        &mut self,
        addr: &Address,
        name: &str,
        has_previous: bool,
    ) -> Result<(), ControlFlow> {
        // operand
        let operand = self.pop(addr);
        // non-previous
        if !has_previous {
            // store in stack
            self.define_variable(addr, name, operand);
        }
        // previous
        else {
            let previous = self.pop(addr);

            match previous {
                // define in instance
                Value::Instance(mut instance) => {
                    instance.environment.define(addr, name, operand);
                }
                // define in unit
                Value::Unit(mut unit) => {
                    unit.environment.define(addr, name, operand);
                }
                _ => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("could not define variable in {previous:?}"),
                        "you can define variable in unit or instance."
                    ))
                }
            }
        }

        Ok(())
    }

    /// Opcode: Store
    ///
    /// stores value in local table
    /// or, if `has_previous` pops
    /// value (instance/unit, otherwise raises error)
    /// from stack and then sets given
    /// value in it by name
    ///
    unsafe fn op_store(
        &mut self,
        addr: &Address,
        name: &str,
        has_previous: bool,
    ) -> Result<(), ControlFlow> {
        // operand
        let operand = self.pop(addr);
        // non-previous
        if !has_previous {
            // store in stack
            self.store_variable(addr, name, operand);
        }
        // previous
        else {
            let previous = self.pop(addr);
            match previous {
                // store in instance
                Value::Instance(mut instance) => {
                    instance.environment.store(addr, name, operand);
                }
                // store in unit
                Value::Unit(mut unit) => {
                    unit.environment.store(addr, name, operand);
                }
                // store in module
                Value::Module(mut module) => {
                    module.environment.store(addr, name, operand);
                }
                _ => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("could not store variable in {previous:?}."),
                        "you can set variable in unit or instance."
                    ))
                }
            }
        }

        Ok(())
    }

    /// Opcode: Load
    ///
    /// load value from local table,
    /// types table, traits table or units table.
    /// or, if `has_previous` pops
    /// value (instance/unit, otherwise raises error)
    /// from stack and then load from it.
    ///
    unsafe fn op_load(
        &mut self,
        addr: &Address,
        name: &str,
        has_previous: bool,
        should_push: bool,
    ) -> Result<(), ControlFlow> {
        // non-previous
        if !has_previous {
            // loading variable value from stack
            let value = self.load_variable(addr, name);

            // pushing value
            if should_push {
                self.push(value);
            }
        }
        // previous
        else {
            let previous = self.pop(addr);
            match previous {
                // from instance
                Value::Instance(instance) => {
                    let value = instance.environment.load(addr, name);
                    if should_push {
                        self.push(value);
                    }
                }
                // from unit
                Value::Unit(unit) => {
                    let value = unit.environment.load(addr, name);
                    if should_push {
                        self.push(value);
                    }
                }
                // from module
                Value::Module(module) => {
                    let value = module.environment.load(addr, name);
                    if should_push {
                        self.push(value);
                    }
                }
                _ => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("{previous:?} is not a container."),
                        "you can load variable from unit or instance."
                    ))
                }
            }
        }

        Ok(())
    }

    /// Call
    ///
    /// calls fn by name from local table,
    /// or, if `has_previous` pops
    /// value (instance/unit, otherwise raises error)
    /// from stack and then calls fn
    /// by name from it
    ///
    #[allow(unused_parens)]
    pub unsafe fn call(
        &mut self,
        addr: &Address,
        name: &str,
        callable: Value,
        args: &Chunk,
        should_push: bool,
    ) -> Result<(), ControlFlow> {
        /// Just loads arguments to stack
        ///
        /// * `params_amount`: expected params amount
        /// * `args`: args chunk
        /// * `table`: table, where fn called
        /// * `call_table`: call table
        ///
        unsafe fn load_arguments(
            vm: &mut VirtualMachine,
            addr: &Address,
            name: &str,
            params_amount: usize,
            args: &Chunk,
        ) -> Result<(), ControlFlow> {
            // passing args
            let prev_size = vm.operand_stack.len();
            vm.run(args)?;
            let new_size = vm.operand_stack.len();
            let passed_amount = new_size - prev_size;

            // ensuring args && params amount are equal
            if passed_amount == params_amount {
                Ok(())
            } else {
                error!(Error::own(
                    addr.clone(),
                    format!(
                        "invalid args amount: {} to call: {}. stack: {:?}",
                        passed_amount, name, vm.operand_stack
                    ),
                    format!("expected {params_amount} arguments.")
                ));
            }
        }

        // checking value is fn
        if let Value::Fn(function) = callable {
            // loading arguments
            load_arguments(self, addr, name, function.params.len(), args)?;

            // pushing frame
            self.push_frame_with_closure(function.closure.clone());

            // owner
            if let Some(owner) = &function.owner {
                match (**owner).clone() {
                    FnOwner::Unit(unit) => {
                        self.define_variable(addr, "self", Value::Unit(unit.clone()));
                    }
                    FnOwner::Instance(instance) => {
                        self.define_variable(addr, "self", Value::Instance(instance.clone()));
                    }
                }
            }

            // defining params variables with
            // args values
            for param in function.params.iter().rev() {
                let operand = self.pop(addr);
                self.define_variable(addr, param, operand);
            }

            // running body
            if let Err(e) = self.run(&function.body) {
                return match e {
                    // if return
                    ControlFlow::Return(val) => {
                        // if should push, pushing
                        if should_push {
                            self.push(val);
                        }

                        // popping frame
                        self.pop_frame();

                        Ok(())
                    }
                    // otherwise, panic
                    _ => {
                        panic!("unhandled control flow: {e:?}. report this error to the developer.")
                    }
                };
            }
            Ok(())
        }
        // checking value is native
        else if let Value::Native(function) = callable {
            // loading arguments
            load_arguments(self, addr, name, function.params_amount, args)?;

            // pushing frame
            self.push_frame();

            // calling native fn
            let native = function.function;
            let result = native(self, addr.clone(), should_push);

            // error capturing
            if let Err(e) = result {
                panic!(
                    "native function caused control flow leak `{:?}`. report this error to the developer.",
                    e
                )
            }

            // popping frame
            self.pop_frame();

            Ok(())
        } else {
            error!(Error::own_text(
                addr.clone(),
                format!("{name} is not a fn."),
                "you can call only fn-s."
            ));
        }
    }

    /// Opcode: Call
    ///
    /// calls value by name
    ///
    /// if has_previous is true,
    /// safety if previous is tempo,
    /// guaranteed by guarding in gc
    ///
    pub unsafe fn op_call(
        &mut self,
        addr: &Address,
        name: &str,
        has_previous: bool,
        should_push: bool,
        args: &Chunk,
    ) -> Result<(), ControlFlow> {
        // non-previous
        if !has_previous {
            let value = self.load_variable(addr, name);
            self.call(addr, name, value, args, should_push)
        }
        // previous
        else {
            // getting previous and guarding
            let previous = self.pop(addr);
            // calling a function
            match previous {
                // call from instance
                Value::Instance(instance) => {
                    // guarding
                    guard!(instance);
                    // calling
                    let value = instance.environment.load(addr, name);
                    let result = self.call(addr, name, value, args, should_push);
                    // unguarding
                    unguard!(instance);
                    // returning result
                    result
                }
                // call from unit
                Value::Unit(unit) => {
                    // guarding
                    guard!(unit);
                    // calling
                    let value = unit.environment.load(addr, name);
                    let result = self.call(addr, name, value, args, should_push);
                    // unguarding
                    unguard!(unit);
                    // returning result
                    result
                }
                // call from module
                Value::Module(module) => {
                    // guarding
                    guard!(module);
                    // calling
                    let value = module.environment.load(addr, name);
                    let result = self.call(addr, name, value, args, should_push);
                    // unguarding
                    unguard!(module);
                    // returning result
                    result
                }
                _ => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("couldn't call {name} from {previous:?}."),
                        "you can call fn from unit, instance or module."
                    ))
                }
            }
        }
    }

    /// Opcode: Duplicate
    /// duplicates value in stack
    ///
    unsafe fn op_duplicate(&mut self, addr: &Address) -> Result<(), ControlFlow> {
        // duplicating operand
        let operand = self.pop(addr);
        self.push(operand.clone());
        self.push(operand);
        Ok(())
    }

    /// Checking instance impls all traits,
    /// if instance doesn't impl some fn-s,
    /// adds default implementation if exists,
    /// otherwise raises error
    ///
    /*
    unsafe fn check_traits(
        &mut self,
        addr: &Address,
        mut instance: Gc<Instance>,
    ) {
        // type of instance, used to check traits
        let instance_type = instance.oil_type.clone();

        /// Gets trait by name
        unsafe fn get_trait(
            addr: &Address,
            trait_name: &str,
        ) -> Option<Gc<Trait>> {
            // looking up trait
            let trait_value = self.load_variable(addr, trait_name);

            match trait_value {
                Value::Trait(_trait) => Some(_trait),
                _ => {
                    panic!("not a trait in traits table. report to developer.")
                }
            }
        }

        /// Gets impl by name
        unsafe fn get_impl(
            table: Gc<Table>,
            addr: &Address,
            impl_name: &str,
        ) -> Option<Gc<Function>> {
            // looking up for impl
            let fn_value = (*table).lookup(addr, impl_name);

            match fn_value {
                Value::Fn(_fn) => Some(_fn),
                _ => None,
            }
        }

        // checking all traits from a type
        for trait_name in &instance_type.impls {
            let _trait = get_trait(table.clone(), addr, trait_name).unwrap();
            // checking all fn-s
            for function in &_trait.functions {
                // if impl exists, checking it
                if instance.fields.exists(&function.name) {
                    // checking impl
                    let _impl = get_impl(instance.fields.clone(), addr, &function.name);

                    // if impl exists, checking params amount
                    if _impl.is_some() {
                        let implementation = _impl.unwrap();
                        if implementation.params.len() != function.params_amount {
                            error!(Error::own(
                                addr.clone(),
                                format!(
                                    "type {} impls {}, but fn {} has wrong impl.",
                                    instance_type.name, trait_name, function.name
                                ),
                                format!(
                                    "expected args {}, got {}",
                                    function.params_amount,
                                    implementation.params.len()
                                )
                            ));
                        }
                    } else {
                        error!(Error::own(
                            addr.clone(),
                            format!(
                                "type {} impls {}, but doesn't impl fn {}({})",
                                instance_type.name,
                                trait_name,
                                function.name.clone(),
                                function.params_amount
                            ),
                            format!("implement fn {}", function.name.clone())
                        ));
                    }
                } else {
                    // default implementation
                    if function.default.is_some() {
                        // creating default fn
                        let default_impl = function.default.as_ref().unwrap();
                        let default_fn = Value::Fn(Gc::new(Function::new(
                            function.name.clone(),
                            Gc::new(default_impl.chunk.clone()),
                            default_impl.params.clone(),
                        )));

                        // defining fn in fields of instance
                        instance.fields.define(addr, &function.name, default_fn);
                    } else {
                        error!(Error::own(
                            addr.clone(),
                            format!(
                                "type {} impls {}, but doesn't impl fn {}({})",
                                instance_type.name,
                                trait_name,
                                function.name,
                                function.params_amount
                            ),
                            format!("implement fn {}", function.name)
                        ));
                    }
                }
            }
        }
    }
    */

    /// Opcode: Instance
    /// creates instance `safely`
    /// of a given type and then
    /// pushes it to stack
    ///
    /// safety guaranteed by using gc_guard
    /// before registering in gc, and gc_unguard after
    /// registering in gc
    ///
    unsafe fn op_instance(
        &mut self,
        addr: &Address,
        args: &Chunk,
        should_push: bool,
    ) -> Result<(), ControlFlow> {
        /// Just loads constructor args to stack
        ///
        /// * `params_amount`: expected params amount
        /// * `args`: args chunk
        /// * `params`: params vector, used
        ///   to set variables with params names
        ///   to args
        /// * `table`: table, where instance created
        /// * `fields_table`: call table
        ///
        unsafe fn load_constructor(
            vm: &mut VirtualMachine,
            addr: &Address,
            name: &str,
            params_amount: usize,
            args: &Chunk,
        ) -> Result<(), ControlFlow> {
            // passing args
            let prev_size = vm.operand_stack.len();
            vm.run(args)?;
            let new_size = vm.operand_stack.len();
            let passed_amount = new_size - prev_size;

            // ensuring args && params amount are equal
            if passed_amount == params_amount {
                Ok(())
            } else {
                error!(Error::own(
                    addr.clone(),
                    format!("invalid args amount: {passed_amount} to create instance of {name}."),
                    format!("expected {params_amount} arguments.")
                ));
            }
        }

        // getting a type
        let value = self.pop(addr);
        match value {
            Value::Type(oil_type) => {
                // loading constructor
                load_constructor(self, addr, &oil_type.name, oil_type.constructor.len(), args)?;

                // pushing frame
                self.push_frame();

                // defining params variables with
                // args values
                for param in oil_type.constructor.iter().rev() {
                    let operand = self.pop(addr);
                    self.define_variable(addr, param, operand);
                }

                // executing body
                self.run(&oil_type.body)?;

                // popping environment
                let environment = self.pop_frame().pop();

                // creating instance
                let instance = Gc::new(Instance::new(oil_type.clone(), environment));
                let instance_value = Value::Instance(instance.clone());

                // guard
                guard!(instance);

                // binding function
                self.bind_functions(
                    instance.environment.clone(),
                    Gc::new(FnOwner::Instance(instance.clone())),
                );

                // calling optional init fn
                let init_fn = "init";
                if instance.environment.is_exists(init_fn) {
                    // pushing frame
                    self.push_frame();
                    // calling `init`
                    self.push(instance_value.clone());
                    if let Err(e) = self.op_call(addr, init_fn, true, false, &Chunk::new(vec![])) {
                        panic!(
                            "`init` fn returned a {:?}. report error to the developer",
                            e
                        )
                    }
                    // popping frame
                    self.pop_frame();
                }

                // unguard
                unguard!(instance);

                // pushing instance
                if should_push {
                    self.push(instance_value);
                }

                Ok(())
            }
            _ => {
                error!(Error::own_text(
                    addr.clone(),
                    format!("{value} is not a type"),
                    "you can create instances only of types."
                ))
            }
        }
    }

    /// Opcode: EndLoop
    #[allow(unused_variables)]
    unsafe fn op_endloop(
        &mut self,
        addr: &Address,
        current_iteration: bool,
    ) -> Result<(), ControlFlow> {
        // returning control flow
        if current_iteration {
            Err(ControlFlow::Continue)
        } else {
            Err(ControlFlow::Break)
        }
    }

    /// Opcode: Return
    unsafe fn op_return(&mut self, addr: &Address) -> Result<(), ControlFlow> {
        // running value and returning control flow
        let value = self.pop(addr);
        Err(ControlFlow::Return(value))
    }

    /// Opcode: Native
    ///
    /// Sets native fn, provided
    /// in `/vm/natives/natives.rs`
    /// in local table by name
    ///
    unsafe fn op_native(&mut self, addr: &Address, name: &str) -> Result<(), ControlFlow> {
        // finding native function, provided
        // by `vm/natives/natives.rs` and pushing to stack
        let value = self.natives_table.load(addr, name);
        self.push(value);

        Ok(())
    }

    /// Opcode: ErrorPropagation
    ///
    /// If value implements `is_ok`, and if `is_ok` == false,
    /// returns `propagation value` by `ControlFlow::Return(_)`
    ///
    /// Otherwise, if `is_ok` == true, unwraps `propagation value`
    ///
    unsafe fn op_error_propagation(
        &mut self,
        addr: &Address,
        should_push: bool,
    ) -> Result<(), ControlFlow> {
        /// Calls is ok
        /// from an instance
        ///
        unsafe fn call_is_ok(
            vm: &mut VirtualMachine,
            addr: &Address,
            instance: Gc<Instance>,
        ) -> Result<bool, ControlFlow> {
            // finding callable
            let callable = instance.environment.load(addr, "is_ok");
            match callable.clone() {
                Value::Fn(function) => {
                    if !function.params.is_empty() {
                        error!(Error::own_text(
                            addr.clone(),
                            format!("is_ok takes {} params", function.params.len()),
                            "is_ok should take 0 params."
                        ));
                    }
                }
                _ => {
                    error!(Error::new(
                        addr.clone(),
                        "is_ok is not a fn.",
                        "is_ok should be fn."
                    ));
                }
            }

            // calling
            vm.call(addr, "is_ok", callable, &Chunk::new(vec![]), true)?;

            // matching result
            let result = vm.pop(addr);
            match result {
                Value::Bool(boolean) => Ok(boolean),
                _ => {
                    error!(Error::own(
                        addr.clone(),
                        "is_ok should return a bool.".to_string(),
                        format!("it returned: {result:?}")
                    ));
                }
            }
        }

        /// Calls unwrap
        /// from an instance
        ///
        unsafe fn call_unwrap(
            vm: &mut VirtualMachine,
            addr: &Address,
            instance: Gc<Instance>,
        ) -> Result<(), ControlFlow> {
            let callable = instance.environment.load(addr, "unwrap");
            match callable.clone() {
                Value::Fn(function) => {
                    if !function.params.is_empty() {
                        error!(Error::own_text(
                            addr.clone(),
                            format!("unwrap takes {} params", function.params.len()),
                            "unwrap should take 0 params."
                        ));
                    }
                }
                _ => {
                    error!(Error::new(
                        addr.clone(),
                        "unwrap is not a fn.",
                        "unwrap should be fn."
                    ));
                }
            }

            vm.call(addr, "unwrap", callable, &Chunk::new(vec![]), true)?;
            Ok(())
        }

        let value = self.pop(addr);
        if let Value::Instance(ref instance) = value {
            // calling is ok
            let is_ok = call_is_ok(self, addr, instance.clone())?;

            // if it's no ok
            if !is_ok {
                // returning value back
                return Err(ControlFlow::Return(value.clone()));
            } else {
                // calling unwrap
                if should_push {
                    call_unwrap(self, addr, instance.clone())?;
                }
            }
        } else {
            error!(Error::own_text(
                addr.clone(),
                format!("could not use error propagation with {value:?}."),
                "requires instance of type that impls .is_ok() and .unwrap() fn-s."
            ))
        }

        Ok(())
    }

    /// Opcode: Impls
    ///
    /// Checks value is impls a
    /// `trait`, named `trait_name`
    ///
    /// todo
    /*
    unsafe fn op_impls(&mut self, addr: &Address) -> Result<(), ControlFlow> {
        // getting trait
        let trait_value = self.pop(addr);
        // getting instance
        let instance_value = self.pop(addr);
        // if value returned instance, checking trait
        // is implemented
        if let Value::Instance(instance) = instance_value {
            // checking trait is implemented
            match trait_value {
                Value::Trait(_trait) => {
                    let impls = &instance.oil_type.impls;

                    let name = &_trait.name;
                    self.push(Value::Bool(impls.contains(name)));
                }
                _ => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("could not use impls with trait {trait_value:?}."),
                        "impls op requires instance."
                    ))
                }
            }
        } else {
            error!(Error::own_text(
                addr.clone(),
                format!("could not use impls with {instance_value:?}."),
                "impls op requires instance."
            ))
        }

        // успех
        Ok(())
    }
    */

    /// Opcode: DeleteLocal
    ///
    /// Deletes a variable from
    /// last environment by name
    ///
    #[allow(unused_variables)]
    unsafe fn op_delete_local(&mut self, addr: &Address, name: &String) {
        self.delete_variable(addr, name);
    }

    /// Opcode: LoadModule
    ///
    /// Loads module, if not loaded
    unsafe fn op_load_module(
        &mut self,
        addr: &Address,
        id: usize,
        variable: &String,
    ) -> Result<(), ControlFlow> {
        match self.modules.get(&id) {
            Some(module) => {
                self.define_variable(addr, variable, Value::Module(module.clone()));
                Ok(())
            }
            None => match self.modules_info.get(&id).cloned() {
                Some(module_info) => {
                    let module = self.run_module(&module_info.chunk)?;
                    self.modules.insert(id, module.clone());
                    self.define_variable(addr, variable, Value::Module(module));
                    Ok(())
                }
                None => {
                    panic!("module with id {id} is not found. report this error to the developer.")
                }
            },
        }
    }

    /// Runs module chunk in new table
    pub unsafe fn run_module(&mut self, chunk: &Chunk) -> Result<Gc<Module>, ControlFlow> {
        // pushing frame
        self.push_frame();
        // pushing environment
        self.peek_frame().push(Gc::new(Environment::new()));
        // pushing module
        self.push_module(Gc::new(Module::new(self.peek_frame().peek())));
        // running chunk
        if let Err(e) = self.run(chunk) {
            panic!(
                "module chunk caused control flow leak `{:?}`. report this error to the developer.",
                e
            )
        }
        // popping frmae
        self.pop_frame().pop();
        // returning module
        Ok(self.pop_module())
    }

    /// Runs chunk
    #[allow(unused_variables)]
    pub unsafe fn run(&mut self, chunk: &Chunk) -> Result<(), ControlFlow> {
        for op in chunk.opcodes() {
            match op {
                Opcode::Push { addr, value } => {
                    self.op_push(value.clone())?;
                }
                Opcode::Pop { addr } => {
                    self.pop(addr);
                }
                Opcode::Bin { addr, op } => {
                    self.op_binary(addr, op)?;
                }
                Opcode::Neg { addr } => {
                    self.op_negate(addr)?;
                }
                Opcode::Bang { addr } => {
                    self.op_bang(addr)?;
                }
                Opcode::Cond { addr, op } => {
                    self.op_conditional(addr, op)?;
                }
                Opcode::Logic { addr, a, b, op } => {
                    self.op_logical(addr, a, b, op)?;
                }
                Opcode::If {
                    addr,
                    cond,
                    body,
                    elif,
                } => {
                    self.op_if(addr, cond, body, elif)?;
                }
                Opcode::Loop { addr, body } => {
                    self.op_loop(addr, body)?;
                }
                Opcode::DefineFn {
                    addr,
                    name,
                    body,
                    params,
                } => {
                    self.op_define_fn(addr, name, body, params)?;
                }
                Opcode::AnonymousFn { addr, body, params } => {
                    self.op_anonymous_fn(body, params)?;
                }
                Opcode::DefineType {
                    addr,
                    name,
                    body,
                    constructor,
                    impls,
                } => {
                    self.op_define_type(addr, name, body, constructor, impls)?;
                }
                Opcode::DefineUnit { addr, name, body } => {
                    self.op_define_unit(addr, name, body)?;
                }
                Opcode::DefineTrait {
                    addr,
                    name,
                    functions,
                } => {
                    self.op_define_trait(addr, name, functions)?;
                }
                Opcode::Define {
                    addr,
                    name,
                    has_previous,
                } => {
                    self.op_define(addr, name, *has_previous)?;
                }
                Opcode::Store {
                    addr,
                    name,
                    has_previous,
                } => {
                    self.op_store(addr, name, *has_previous)?;
                }
                Opcode::Load {
                    addr,
                    name,
                    has_previous,
                    should_push,
                } => {
                    self.op_load(addr, name, *has_previous, *should_push)?;
                }
                Opcode::Call {
                    addr,
                    name,
                    has_previous,
                    should_push,
                    args,
                } => {
                    self.op_call(addr, name, *has_previous, *should_push, args)?;
                }
                Opcode::Duplicate { addr } => {
                    self.op_duplicate(addr)?;
                }
                Opcode::Instance {
                    addr,
                    args,
                    should_push,
                } => {
                    self.op_instance(addr, args, *should_push)?;
                }
                Opcode::EndLoop {
                    addr,
                    current_iteration,
                } => {
                    self.op_endloop(addr, *current_iteration)?;
                }
                Opcode::Ret { addr } => {
                    self.op_return(addr)?;
                }
                Opcode::Native { addr, fn_name } => {
                    self.op_native(addr, fn_name)?;
                }
                Opcode::ErrorPropagation { addr, should_push } => {
                    self.op_error_propagation(addr, *should_push)?;
                }
                Opcode::Impls { addr } => {
                    // self.op_impls(addr)?;
                }
                Opcode::DeleteLocal { addr, name } => self.op_delete_local(addr, name),
                Opcode::ImportModule { addr, id, variable } => {
                    self.op_load_module(addr, *id, variable)?
                }
            }
        }
        Ok(())
    }
}

/// Send & sync for future multi-threading.
unsafe impl Send for VirtualMachine {}
unsafe impl Sync for VirtualMachine {}
