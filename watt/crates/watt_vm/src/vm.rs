use std::collections::HashMap;
use std::ops::Deref;

// imports
use crate::bytecode::{Chunk, ModuleInfo, Opcode, OpcodeValue};
use crate::flow::ControlFlow;
use crate::memory::gc::Gc;
use crate::natives::natives;
use crate::table::Table;
use crate::values::*;
use rustc_hash::FxHashMap;
use watt_common::address::Address;
use watt_common::{error, errors::Error};

/// Virtual machine
///
/// Vm that runs opcodes 🤔
///
#[derive(Debug)]
pub struct VM {
    pub builtins_table: Gc<Table>,
    pub natives_table: Gc<Table>,
    pub stack: Vec<Value>,
    pub modules_info: HashMap<usize, ModuleInfo>,
    pub modules: FxHashMap<usize, Gc<Module>>,
}
/// Vm implementation
#[allow(non_upper_case_globals)]
#[allow(unused_qualifications)]
impl VM {
    /// New vm
    pub unsafe fn new(builtins: Chunk, modules_info: HashMap<usize, ModuleInfo>) -> VM {
        // vm
        let mut vm = VM {
            builtins_table: Gc::new(Table::new()),
            natives_table: Gc::new(Table::new()),
            stack: Vec::new(),
            modules_info,
            modules: FxHashMap::default(),
        };
        // natives
        if let Err(e) = natives::provide_builtins(&mut vm) {
            error!(e)
        }
        // running builtins
        if let Err(e) = vm.run(&builtins, vm.builtins_table.clone()) {
            error!(Error::own_text(
                Address::unknown(),
                format!("control flow leak: {e:?}"),
                "report this error to the developer."
            ));
        }
        // returns vm
        vm
    }

    /// Push value to vm stack
    pub unsafe fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pop value from vm stack
    pub fn pop(&mut self, address: &Address) -> Value {
        if self.stack.is_empty() {
            error!(Error::new(
                address.clone(),
                "stack underflow.",
                "check your code."
            ));
        }
        self.stack.pop().unwrap()
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
        table: Gc<Table>,
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
        self.run(a, table.clone())?;
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
                    self.run(b, table)?;
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
                    self.run(b, table)?;
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
        root: Gc<Table>,
    ) -> Result<(), ControlFlow> {
        // condition table
        let mut table = Gc::new(Table::new());
        table.set_root(root);

        // running condition
        self.run(cond, table.clone())?;
        let bool = self.pop(addr);

        // checking condition returned true
        if let Value::Bool(b) = bool {
            if b {
                self.run(body, table)?
            } else if let Option::Some(else_if) = elif {
                self.run(else_if, table)?
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
    unsafe fn op_loop(
        &mut self,
        addr: &Address,
        body: &Chunk,
        root: Gc<Table>,
    ) -> Result<(), ControlFlow> {
        // loop table
        let mut table = Gc::new(Table::new());
        table.set_root(root);

        // loop
        loop {
            if let Err(e) = self.run(body, table.clone()) {
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
        make_closure: bool,
        mut table: Gc<Table>,
    ) -> Result<(), ControlFlow> {
        // allocating function
        let mut function = Gc::new(Function::new(
            name.clone(),
            Gc::new(body.clone()),
            params.to_owned(),
        ));

        // if it's need to make_closure
        if make_closure {
            // setting closure
            function.closure = Some(table.clone());
        }

        // function value
        let function_value = Value::Fn(function);

        // defining fn by name and full name
        table.define(addr, name, function_value);

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
        make_closure: bool,
        table: Gc<Table>,
    ) -> Result<(), ControlFlow> {
        // allocating function
        let mut function = Gc::new(Function::new(
            "$lambda".to_string(),
            Gc::new(body.clone()),
            params.to_owned(),
        ));

        // if it's need to make_closure
        if make_closure {
            // setting closure
            function.closure = Some(table.clone());
        }

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
    unsafe fn bind_functions(&mut self, mut table: Gc<Table>, owner: Gc<FnOwner>) {
        for val in table.fields.values_mut() {
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
        impls: &[String], // todo: replace &[String] with &[Opcode]
        mut table: Gc<Table>,
    ) -> Result<(), ControlFlow> {
        // allocating type
        let t = Gc::new(Type::new(
            name.clone(),
            constructor.to_owned(),
            Gc::new(body.clone()),
            impls.to_owned(),
            table.clone(),
        ));

        // defining type by name && full name
        table.define(addr, name.as_str(), Value::Type(t));
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
        mut table: Gc<Table>,
    ) -> Result<(), ControlFlow> {
        // allocating unit
        let mut unit = Gc::new(Unit::new(name.clone(), Gc::new(Table::new())));

        // unit value
        let unit_value = Value::Unit(unit.clone());

        // setting root for fields
        unit.fields.set_root(table.clone());

        // inserting temp self
        unit.fields
            .fields
            .insert("self".to_string(), unit_value.clone());

        // executing body
        self.run(body, unit.fields.clone())?;

        // deleting temp self
        unit.fields.fields.remove("self");

        // binding function
        self.bind_functions(unit.fields.clone(), Gc::new(FnOwner::Unit(unit.clone())));

        // calling optional init fn
        let init_fn = "init";
        if unit.fields.exists(init_fn) {
            self.push(unit_value.clone());
            self.op_call(
                addr,
                init_fn,
                true,
                false,
                &Chunk::new(vec![]),
                unit.fields.clone(),
            )?
        }

        // defining unit by name and full name
        table.define(addr, name.as_str(), unit_value);

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
        mut table: Gc<Table>,
    ) -> Result<(), ControlFlow> {
        // allocating trait
        let _trait = Gc::new(Trait::new(name.clone(), functions.to_owned()));

        // define trait by name
        table.define(addr, name.as_str(), Value::Trait(_trait));

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
        value: &Chunk,
        mut table: Gc<Table>,
    ) -> Result<(), ControlFlow> {
        // non-previous
        if !has_previous {
            self.run(value, table.clone())?;
            let operand = self.pop(addr);
            table.define(addr, name, operand);
        }
        // previous
        else {
            let previous = self.pop(addr);

            match previous {
                // define in instance
                Value::Instance(mut instance) => {
                    self.run(value, table)?;
                    let operand = self.pop(addr);
                    instance.fields.define(addr, name, operand);
                }
                // define in unit
                Value::Unit(mut unit) => {
                    self.run(value, table)?;
                    let operand = self.pop(addr);
                    unit.fields.define(addr, name, operand);
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

    /// Opcode: Set
    ///
    /// sets value in local table
    /// or, if `has_previous` pops
    /// value (instance/unit, otherwise raises error)
    /// from stack and then sets given
    /// value in it by name
    ///
    unsafe fn op_set(
        &mut self,
        addr: &Address,
        name: &str,
        has_previous: bool,
        value: &Chunk,
        mut table: Gc<Table>,
    ) -> Result<(), ControlFlow> {
        // non-previous
        if !has_previous {
            self.run(value, table.clone())?;
            let operand = self.pop(addr);
            table.set(addr, name, operand);
        }
        // previous
        else {
            let previous = self.pop(addr);

            match previous {
                // define in instance
                Value::Instance(mut instance) => {
                    self.run(value, table)?;
                    let operand = self.pop(addr);
                    instance.fields.set_local(addr, name, operand);
                }
                // define in unit
                Value::Unit(mut unit) => {
                    self.run(value, table)?;
                    let operand = self.pop(addr);
                    unit.fields.set_local(addr, name, operand);
                }
                _ => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("could not set variable in {previous:?}."),
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
        table: Gc<Table>,
    ) -> Result<(), ControlFlow> {
        // non-previous
        if !has_previous {
            // loading variable value from table
            let value = table.lookup(addr, name);

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
                    let value = instance.fields.find(addr, name);
                    if should_push {
                        self.push(value);
                    }
                }
                // from unit
                Value::Unit(unit) => {
                    let value = unit.fields.find(addr, name);
                    if should_push {
                        self.push(value);
                    }
                }
                // from module
                Value::Module(module) => {
                    let value = module.table.find(addr, name);
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
        table: Gc<Table>,
        should_push: bool,
    ) -> Result<(), ControlFlow> {
        /// Pass arguments
        ///
        /// * `params_amount`: expected params amount
        /// * `args`: args chunk
        /// * `params`: params vector, used
        ///   to set variables with params names
        ///   to args
        /// * `table`: table, where fn called
        /// * `call_table`: call table
        ///
        unsafe fn pass_arguments(
            vm: &mut VM,
            addr: &Address,
            name: &str,
            params_amount: usize,
            args: &Chunk,
            params: Vec<String>,
            table: Gc<Table>,
            mut call_table: Gc<Table>,
        ) -> Result<(), ControlFlow> {
            // passing args
            let prev_size = vm.stack.len();
            vm.run(args, table)?;
            let new_size = vm.stack.len();
            let passed_amount = new_size - prev_size;

            // ensuring args && params amount are equal
            if passed_amount == params_amount {
                // defining params variables with
                // args values
                for param in params.iter().rev() {
                    let operand = vm.pop(addr);
                    call_table.define(addr, param, operand);
                }

                Ok(())
            } else {
                error!(Error::own(
                    addr.clone(),
                    format!(
                        "invalid args amount: {} to call: {}. stack: {:?}",
                        passed_amount, name, vm.stack
                    ),
                    format!("expected {params_amount} arguments.")
                ));
            }
        }

        /// Just loads arguments to stack
        ///
        /// * `params_amount`: expected params amount
        /// * `args`: args chunk
        /// * `table`: table, where fn called
        /// * `call_table`: call table
        ///
        unsafe fn load_arguments(
            vm: &mut VM,
            addr: &Address,
            name: &str,
            params_amount: usize,
            args: &Chunk,
            table: Gc<Table>,
        ) -> Result<(), ControlFlow> {
            // passing args
            let prev_size = vm.stack.len();
            vm.run(args, table)?;
            let new_size = vm.stack.len();
            let passed_amount = new_size - prev_size;

            // ensuring args && params amount are equal
            if passed_amount == params_amount {
                Ok(())
            } else {
                error!(Error::own(
                    addr.clone(),
                    format!(
                        "invalid args amount: {} to call: {}. stack: {:?}",
                        passed_amount, name, vm.stack
                    ),
                    format!("expected {params_amount} arguments.")
                ));
            }
        }

        // checking value is fn
        if let Value::Fn(function) = callable {
            // call table
            let mut call_table = Gc::new(Table::new());

            // parent and closure tables, to chain call_table
            // with current
            call_table.closure = function.closure.clone();

            // root & self
            if let Some(owner) = &function.owner {
                match (*owner).deref() {
                    FnOwner::Unit(unit) => {
                        call_table.set_root(unit.fields.clone());
                        call_table.define(addr, "self", Value::Unit(unit.clone()));
                    }
                    FnOwner::Instance(instance) => {
                        call_table.set_root(instance.fields.clone());
                        call_table.define(addr, "self", Value::Instance(instance.clone()));
                    }
                    FnOwner::Module(module) => {
                        call_table.set_root(module.table.clone());
                    }
                }
            }

            // passing args
            pass_arguments(
                self,
                addr,
                name,
                function.params.len(),
                args,
                function.params.clone(),
                table,
                call_table.clone(),
            )?;

            // running body
            if let Err(e) = self.run(&function.body, call_table) {
                return match e {
                    // if return
                    ControlFlow::Return(val) => {
                        if should_push {
                            self.push(val);
                        }
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
            // call table
            let mut call_table = Gc::new(Table::new());

            // root to globals
            call_table.set_root(function.defined_in.clone());

            // loading arguments to stack
            load_arguments(self, addr, name, function.params_amount, args, table)?;

            // calling native fn
            let native = function.function;
            native(self, addr.clone(), should_push, call_table)?;

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
        table: Gc<Table>,
    ) -> Result<(), ControlFlow> {
        // non-previous
        if !has_previous {
            let value = table.lookup(addr, name);
            self.call(addr, name, value, args, table, should_push)
        }
        // previous
        else {
            // getting previous and guarding
            let previous = self.pop(addr);
            // calling a function
            match previous {
                // call from instance
                Value::Instance(instance) => {
                    let value = instance.fields.find(addr, name);
                    self.call(addr, name, value, args, table, should_push)
                }
                // call from unit
                Value::Unit(unit) => {
                    let value = unit.fields.find(addr, name);
                    self.call(addr, name, value, args, table, should_push)
                }
                // call from module
                Value::Module(module) => {
                    let value = module.table.find(addr, name);
                    self.call(addr, name, value, args, table, should_push)
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
    unsafe fn check_traits(
        &mut self,
        addr: &Address,
        mut instance: Gc<Instance>,
        table: Gc<Table>,
    ) {
        // type of instance, used to check traits
        let instance_type = instance.t.clone();

        /// Gets trait by name
        unsafe fn get_trait(
            traits: Gc<Table>,
            addr: &Address,
            trait_name: &str,
        ) -> Option<Gc<Trait>> {
            // looking up trait
            let trait_value = traits.lookup(addr, trait_name);

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
        table: Gc<Table>,
    ) -> Result<(), ControlFlow> {
        /// Pass constructor
        ///
        /// * `params_amount`: expected params amount
        /// * `args`: args chunk
        /// * `params`: params vector, used
        ///   to set variables with params names
        ///   to args
        /// * `table`: table, where instance created
        /// * `fields_table`: call table
        ///
        unsafe fn pass_constructor(
            vm: &mut VM,
            addr: &Address,
            name: &str,
            params_amount: usize,
            args: &Chunk,
            params: Vec<String>,
            table: Gc<Table>,
            mut fields_table: Gc<Table>,
        ) -> Result<(), ControlFlow> {
            // passing args
            let prev_size = vm.stack.len();
            vm.run(args, table)?;
            let new_size = vm.stack.len();
            let passed_amount = new_size - prev_size;

            // ensuring args && params amount are equal
            if passed_amount == params_amount {
                // defining params variables with
                // args values
                for param in params.iter().rev() {
                    let operand = vm.pop(addr);
                    fields_table.define(addr, param, operand);
                }
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
            Value::Type(t) => {
                // creating instance
                let mut instance = Gc::new(Instance::new(t.clone(), Gc::new(Table::new())));
                let instance_value = Value::Instance(instance.clone());

                // passing constructor
                pass_constructor(
                    self,
                    addr,
                    &t.name,
                    t.constructor.len(),
                    args,
                    t.constructor.clone(),
                    table.clone(),
                    instance.fields.clone(),
                )?;

                // setting root
                instance.fields.set_root(t.defined_in.clone());

                // setting temp self
                instance
                    .fields
                    .fields
                    .insert("self".to_string(), instance_value.clone());

                // executing body
                self.run(&t.body, instance.fields.clone())?;

                // deleting temp self
                instance.fields.fields.remove("self");

                // checking traits implementation
                self.check_traits(addr, instance.clone(), t.defined_in.clone());

                // binding functions
                self.bind_functions(
                    instance.fields.clone(),
                    Gc::new(FnOwner::Instance(instance.clone())),
                );

                // calling optional init fn
                let init_fn = "init";
                if instance.fields.exists(init_fn) {
                    self.push(instance_value.clone());
                    self.op_call(addr, init_fn, true, false, &Chunk::new(vec![]), table)?
                }

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
    unsafe fn op_return(
        &mut self,
        addr: &Address,
        value: &Chunk,
        table: Gc<Table>,
    ) -> Result<(), ControlFlow> {
        // running value and returning control flow
        self.run(value, table)?;
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
        let value = self.natives_table.find(addr, name);
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
        value: &Chunk,
        table: Gc<Table>,
        should_push: bool,
    ) -> Result<(), ControlFlow> {
        // running value
        self.run(value, table.clone())?;
        let value = self.pop(addr);

        /// Calls is ok
        /// from an instance
        ///
        unsafe fn call_is_ok(
            vm: &mut VM,
            addr: &Address,
            instance: Gc<Instance>,
            table: Gc<Table>,
        ) -> Result<bool, ControlFlow> {
            // finding callable
            let callable = instance.fields.find(addr, "is_ok");
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
            vm.call(addr, "is_ok", callable, &Chunk::new(vec![]), table, true)?;

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
            vm: &mut VM,
            addr: &Address,
            instance: Gc<Instance>,
            table: Gc<Table>,
        ) -> Result<(), ControlFlow> {
            let callable = instance.fields.find(addr, "unwrap");

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

            vm.call(addr, "unwrap", callable, &Chunk::new(vec![]), table, true)?;
            Ok(())
        }

        if let Value::Instance(ref instance) = value {
            // calling is ok
            let is_ok = call_is_ok(self, addr, instance.clone(), table.clone())?;

            // if it's no ok
            if !is_ok {
                // returning value back
                return Err(ControlFlow::Return(value.clone()));
            } else {
                // calling unwrap
                if should_push {
                    call_unwrap(self, addr, instance.clone(), table)?;
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
                    let impls = &instance.t.impls;

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

    /// Opcode: DeleteLocal
    ///
    /// Deletes a variable from
    /// local table by name
    ///
    #[allow(unused_variables)]
    unsafe fn op_delete_local(&self, addr: &Address, name: &String, mut table: Gc<Table>) {
        table.fields.remove(name);
    }

    /// Opcode: LoadModule
    ///
    /// Loads module, if not loaded
    unsafe fn op_load_module(
        &mut self,
        addr: &Address,
        id: usize,
        variable: &String,
        mut table: Gc<Table>,
    ) -> Result<(), ControlFlow> {
        match self.modules.get(&id) {
            Some(module) => {
                table.define(addr, variable, Value::Module(module.clone()));
                Ok(())
            }
            None => match self.modules_info.get(&id).cloned() {
                Some(module_info) => {
                    let module = self.run_module(&module_info.chunk)?;
                    self.modules.insert(id, module.clone());
                    table.define(addr, variable, Value::Module(module));
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
        // creating module table
        let mut module_table = Gc::new(Table::new());
        module_table.set_root(self.builtins_table.clone());
        // running chunk
        self.run(chunk, module_table.clone())?;
        // module
        let module = Gc::new(Module::new(module_table.clone()));
        // binding function
        self.bind_functions(module_table, Gc::new(FnOwner::Module(module.clone())));
        // returning module
        Ok(module)
    }

    /// Runs chunk
    #[allow(unused_variables)]
    pub unsafe fn run(&mut self, chunk: &Chunk, table: Gc<Table>) -> Result<(), ControlFlow> {
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
                    self.op_logical(addr, a, b, op, table.clone())?;
                }
                Opcode::If {
                    addr,
                    cond,
                    body,
                    elif,
                } => {
                    self.op_if(addr, cond, body, elif, table.clone())?;
                }
                Opcode::Loop { addr, body } => {
                    self.op_loop(addr, body, table.clone())?;
                }
                Opcode::DefineFn {
                    addr,
                    name,
                    body,
                    params,
                    make_closure,
                } => {
                    self.op_define_fn(addr, name, body, params, *make_closure, table.clone())?;
                }
                Opcode::AnonymousFn {
                    addr,
                    body,
                    params,
                    make_closure,
                } => {
                    self.op_anonymous_fn(body, params, *make_closure, table.clone())?;
                }
                Opcode::DefineType {
                    addr,
                    name,
                    body,
                    constructor,
                    impls,
                } => {
                    self.op_define_type(addr, name, body, constructor, impls, table.clone())?;
                }
                Opcode::DefineUnit { addr, name, body } => {
                    self.op_define_unit(addr, name, body, table.clone())?;
                }
                Opcode::DefineTrait {
                    addr,
                    name,
                    functions,
                } => {
                    self.op_define_trait(addr, name, functions, table.clone())?;
                }
                Opcode::Define {
                    addr,
                    name,
                    value,
                    has_previous,
                } => {
                    self.op_define(addr, name, *has_previous, value, table.clone())?;
                }
                Opcode::Set {
                    addr,
                    name,
                    value,
                    has_previous,
                } => {
                    self.op_set(addr, name, *has_previous, value, table.clone())?;
                }
                Opcode::Load {
                    addr,
                    name,
                    has_previous,
                    should_push,
                } => {
                    self.op_load(addr, name, *has_previous, *should_push, table.clone())?;
                }
                Opcode::Call {
                    addr,
                    name,
                    has_previous,
                    should_push,
                    args,
                } => {
                    self.op_call(addr, name, *has_previous, *should_push, args, table.clone())?;
                }
                Opcode::Duplicate { addr } => {
                    self.op_duplicate(addr)?;
                }
                Opcode::Instance {
                    addr,
                    args,
                    should_push,
                } => {
                    self.op_instance(addr, args, *should_push, table.clone())?;
                }
                Opcode::EndLoop {
                    addr,
                    current_iteration,
                } => {
                    self.op_endloop(addr, *current_iteration)?;
                }
                Opcode::Ret { addr, value } => {
                    self.op_return(addr, value, table.clone())?;
                }
                Opcode::Native { addr, fn_name } => {
                    self.op_native(addr, fn_name)?;
                }
                Opcode::ErrorPropagation {
                    addr,
                    value,
                    should_push,
                } => {
                    self.op_error_propagation(addr, value, table.clone(), *should_push)?;
                }
                Opcode::Impls { addr } => {
                    self.op_impls(addr)?;
                }
                Opcode::DeleteLocal { addr, name } => {
                    self.op_delete_local(addr, name, table.clone())
                }
                Opcode::ImportModule { addr, id, variable } => {
                    self.op_load_module(addr, *id, variable, table.clone())?
                }
            }
        }
        Ok(())
    }
}

/// Send & sync for future multi-threading.
unsafe impl Send for VM {}
unsafe impl Sync for VM {}
