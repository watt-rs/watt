// imports
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::bytecode::{Chunk, Opcode, OpcodeValue};
use crate::vm::flow::ControlFlow;
use crate::vm::memory::gc::GC;
use crate::vm::memory::memory;
use crate::vm::natives::natives;
use crate::vm::table::Table;
use crate::vm::values::{FnOwner, Function, Instance, Symbol, Trait, TraitFn, Type, Unit, Value};
use scopeguard::defer;

/// Vm settings,
/// contains gc_threshold, gc_debug
#[derive(Debug)]
pub struct VmSettings {
    gc_threshold: usize,
    gc_debug: bool,
}
/// Vm settings implementation
impl VmSettings {
    pub fn new(gc_threshold: usize, gc_debug: bool) -> Self {
        Self {
            gc_threshold,
            gc_debug,
        }
    }
}

/// Virtual machine
///
/// Vm that runs opcodes 🤔
///
#[derive(Debug)]
pub struct VM {
    pub globals: *mut Table,
    types: *mut Table,
    pub units: *mut Table,
    traits: *mut Table,
    pub natives: *mut Table,
    pub gc: *mut GC,
    settings: VmSettings,
    pub stack: Vec<Value>,
}
/// Vm implementation
#[allow(non_upper_case_globals)]
#[allow(unused_qualifications)]
impl VM {
    /// New vm
    pub unsafe fn new(settings: VmSettings) -> VM {
        // vm
        let mut vm = VM {
            globals: memory::alloc_value(Table::new()),
            types: memory::alloc_value(Table::new()),
            units: memory::alloc_value(Table::new()),
            traits: memory::alloc_value(Table::new()),
            natives: memory::alloc_value(Table::new()),
            gc: memory::alloc_value(GC::new(settings.gc_debug)),
            stack: Vec::new(),
            settings,
        };
        // natives
        if let Err(e) = natives::provide_builtins(&mut vm) {
            error!(e)
        }
        // returns vm
        vm
    }

    /// Push value to vm stack
    pub unsafe fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pop value from vm stack
    pub fn pop(&mut self, address: &Address) -> Result<Value, ControlFlow> {
        if self.stack.is_empty() {
            error!(Error::new(
                address.clone(),
                "stack underflow.",
                "check your code."
            ));
        }
        Ok(self.stack.pop().unwrap())
    }

    /// Cleanup vm
    ///
    /// Frees all values and
    /// all tables themselves
    ///
    pub unsafe fn cleanup(&mut self) {
        // cleanup gc values
        (*self.gc).cleanup();
        // freeing gc
        memory::free_value(self.gc);
        // freeing types table fields
        (*self.types).free_fields();
        // freeing traits table fields
        (*self.traits).free_fields();
        // freeing tables themselves
        memory::free_value(self.traits);
        memory::free_value(self.types);
        memory::free_value(self.natives);
        memory::free_value(self.units);
        memory::free_value(self.globals);
    }

    /// Invoke garbage collector
    pub unsafe fn gc_invoke(&mut self, table: *mut Table) {
        (*self.gc).collect_garbage(self, table);
    }

    /// Registers object in gc
    ///
    /// if gc objects amount > gc_threshold
    /// | gc invokes
    /// | gc_threshold multiplies by 2
    pub unsafe fn gc_register(&mut self, value: Value, table: *mut Table) {
        // adding object
        (*self.gc).add_object(value);
        // checking gc threshold
        if (*self.gc).objects_amount() > self.settings.gc_threshold {
            // calling gc
            self.gc_invoke(table);
            // doubling current max gc threshold
            self.settings.gc_threshold *= 2;
        }
    }

    /// Guard values from being freed by gc,
    /// by pushing to guard stack
    pub unsafe fn gc_guard(&mut self, value: Value) {
        (*self.gc).push_guard(value);
    }

    /// Unguarding value from being freed by gc
    pub unsafe fn gc_unguard(&mut self) {
        (*self.gc).pop_guard();
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
    pub unsafe fn op_push(
        &mut self,
        value: OpcodeValue,
        table: *mut Table,
    ) -> Result<(), ControlFlow> {
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
                let new_string = Value::String(memory::alloc_value(string));
                // pushing string value to stack
                self.push(new_string);
                // registering string value in gc. .
                self.gc_register(new_string, table);
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
                        // then register
                        self.gc_register(raw, table);
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
    unsafe fn op_binary(
        &mut self,
        address: &Address,
        op: &str,
        table: *mut Table,
    ) -> Result<(), ControlFlow> {
        // operands
        let operand_a = self.pop(address)?;
        let operand_b = self.pop(address)?;

        // error generators
        let invalid_op_error = || {
            error!(Error::own_text(
                address.clone(),
                format!(
                    "could not use '{op}' with {operand_a:?} and {operand_b:?}"
                ),
                "check your code."
            ));
        };
        let division_error = || {
            error!(Error::new(
                address.clone(),
                "division by zero.",
                "undefined operation."
            ));
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
                        let string = Value::String(memory::alloc_value(format!("{}{}", a, *b)));
                        self.push(string);
                        self.gc_register(string, table);
                    }
                    _ => {
                        invalid_op_error();
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
                        let string = Value::String(memory::alloc_value(format!("{}{}", a, *b)));
                        self.push(string);
                        self.gc_register(string, table);
                    }
                    _ => {
                        invalid_op_error();
                    }
                },
                Value::String(a) => {
                    let string =
                        Value::String(memory::alloc_value(format!("{}{:?}", *a, operand_b)));
                    self.push(string);
                    self.gc_register(string, table);
                }
                _ => {
                    if let Value::String(_) = operand_b {
                        let string = Value::String(memory::alloc_value(format!(
                            "{operand_a:?}{operand_b:?}"
                        )));
                        self.push(string);
                        self.gc_register(string, table);
                    } else {
                        invalid_op_error();
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
                        invalid_op_error();
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
                        invalid_op_error();
                    }
                },
                _ => {
                    invalid_op_error();
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
                        invalid_op_error();
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
                        invalid_op_error();
                    }
                },
                _ => {
                    invalid_op_error();
                }
            },
            "/" => {
                match operand_a {
                    Value::Float(a) => {
                        match operand_b {
                            Value::Float(b) => {
                                // проверка на деление на 0
                                if b == 0f64 {
                                    division_error();
                                }
                                // деление
                                self.push(Value::Float(a / b));
                            }
                            Value::Int(b) => {
                                // проверка на деление на 0
                                if b == 0 {
                                    division_error();
                                }
                                // деление
                                self.push(Value::Float(a / (b as f64)));
                            }
                            _ => {
                                invalid_op_error();
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
                                invalid_op_error();
                            }
                        }
                    }
                    _ => {
                        invalid_op_error();
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
                        invalid_op_error();
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
                        invalid_op_error();
                    }
                },
                _ => {
                    invalid_op_error();
                }
            },
            "&" => match operand_a {
                Value::Int(a) => match operand_b {
                    Value::Int(b) => {
                        self.push(Value::Int(a & b));
                    }
                    _ => {
                        invalid_op_error();
                    }
                },
                _ => {
                    invalid_op_error();
                }
            },
            "|" => match operand_a {
                Value::Int(a) => match operand_b {
                    Value::Int(b) => {
                        self.push(Value::Int(a | b));
                    }
                    _ => {
                        invalid_op_error();
                    }
                },
                _ => {
                    invalid_op_error();
                }
            },
            "^" => match operand_a {
                Value::Int(a) => match operand_b {
                    Value::Int(b) => {
                        self.push(Value::Int(a ^ b));
                    }
                    _ => {
                        invalid_op_error();
                    }
                },
                _ => {
                    invalid_op_error();
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
        let operand = self.pop(address)?;
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
        let operand = self.pop(address)?;
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
        let operand_a = self.pop(address)?;
        let operand_b = self.pop(address)?;
        // error
        let invalid_op_error = || {
            Error::own_text(
                address.clone(),
                format!(
                    "could not use '{op}' for {operand_a:?} and {operand_b:?}"
                ),
                "check your code.",
            )
        };
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
                        error!(invalid_op_error());
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
                        error!(invalid_op_error());
                    }
                },
                Value::String(a) => match operand_b {
                    Value::String(b) => {
                        self.push(Value::Bool(*a > *b));
                    }
                    _ => {
                        error!(invalid_op_error());
                    }
                },
                _ => {
                    error!(invalid_op_error());
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
                        error!(invalid_op_error());
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
                        error!(invalid_op_error());
                    }
                },
                Value::String(a) => match operand_b {
                    Value::String(b) => {
                        self.push(Value::Bool(*a < *b));
                    }
                    _ => {
                        error!(invalid_op_error());
                    }
                },
                _ => {
                    error!(invalid_op_error());
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
                        error!(invalid_op_error());
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
                        error!(invalid_op_error());
                    }
                },
                Value::String(a) => match operand_b {
                    Value::String(b) => {
                        self.push(Value::Bool(*a >= *b));
                    }
                    _ => {
                        error!(invalid_op_error());
                    }
                },
                _ => {
                    error!(invalid_op_error());
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
                        error!(invalid_op_error());
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
                        error!(invalid_op_error());
                    }
                },
                Value::String(a) => match operand_b {
                    Value::String(b) => {
                        self.push(Value::Bool(*a <= *b));
                    }
                    _ => {
                        error!(invalid_op_error());
                    }
                },
                _ => {
                    error!(invalid_op_error());
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
                panic!(
                    "operator {op} is not found. report this error to the developer."
                )
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
        table: *mut Table,
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
        self.run(a, table)?;
        let operand_a = self.pop(address)?;
        // operand a
        let a = expect_bool(
            operand_a,
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
                    let operand_b = self.pop(address)?;
                    // operand b
                    let b = expect_bool(
                        operand_b,
                        Error::own_text(
                            address.clone(),
                            format!(
                                "could not use '{op}' for {operand_a:?} and {operand_b:?}"
                            ),
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
                    let operand_b = self.pop(address)?;
                    // operand b
                    let b = expect_bool(
                        operand_b,
                        Error::own_text(
                            address.clone(),
                            format!(
                                "could not use '{op}' for {operand_a:?} and {operand_b:?}"
                            ),
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
        root: *mut Table,
    ) -> Result<(), ControlFlow> {
        // condition table
        let table = memory::alloc_value(Table::new());
        (*table).set_root(root);
        // defer condition table free
        defer! {
            memory::free_value(table);
        }
        // running condition
        self.run(cond, table)?;
        let bool = self.pop(addr)?;
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
        root: *mut Table,
    ) -> Result<(), ControlFlow> {
        // loop table
        let table = memory::alloc_value(Table::new());
        (*table).set_root(root);
        // defer loop table free
        defer! {
            memory::free_value(table);
        }
        // loop
        loop {
            if let Err(e) = self.run(body, table) {
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
        symbol: Symbol,
        body: &Chunk,
        params: &[String],
        make_closure: bool,
        table: *mut Table,
    ) -> Result<(), ControlFlow> {
        // allocating function
        let function = memory::alloc_value(Function::new(
            symbol.clone(),
            memory::alloc_value(body.clone()),
            params.to_owned(),
        ));
        // if it's need to make_closure
        if make_closure {
            // creating closure
            let closure = memory::alloc_value(Table::new());
            // copying table
            (*closure).fields = (*table).fields.clone();
            (*closure).closure = (*table).closure;
            // setting closure
            (*function).closure = closure;
        }

        // function value
        let function_value = Value::Fn(function);

        // guarding value in gc and registering it
        self.gc_guard(function_value);
        self.gc_register(function_value, table);

        // defining fn by name and full name
        if let Err(e) = (*table).define(addr, &symbol.name, function_value) {
            error!(e);
        }
        if symbol.full_name.is_some() {
            if let Err(e) =
                (*table).define(addr, symbol.full_name.as_ref().unwrap(), function_value)
            {
                error!(e);
            }
        }
        // deleting gc guard
        self.gc_unguard();

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
        table: *mut Table,
    ) -> Result<(), ControlFlow> {
        // allocating function
        let function = memory::alloc_value(Function::new(
            Symbol::by_name("$lambda".to_string()),
            memory::alloc_value(body.clone()),
            params.to_owned(),
        ));
        // if it's need to make_closure
        if make_closure {
            // creating closure
            let closure = memory::alloc_value(Table::new());
            // copying table
            (*closure).fields = (*table).fields.clone();
            (*closure).closure = (*table).closure;
            // setting closure
            (*function).closure = closure;
        }

        // function value
        let function_value = Value::Fn(function);

        // push function value to stack
        self.push(function_value);
        // register value in gc
        self.gc_register(function_value, table);

        Ok(())
    }

    /// Bind functions
    ///
    /// Goes through the table fields,
    /// search functions and then binds owner
    /// to them.
    unsafe fn bind_functions(&mut self, table: *mut Table, owner: FnOwner) {
        for val in (*table).fields.values() {
            if let Value::Fn(function) = *val {
                (*function).owner = Some(owner.clone());
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
        symbol: &Symbol,
        body: &Chunk,
        constructor: &[String],
        impls: &[String],
    ) -> Result<(), ControlFlow> {
        // allocating type
        let t = memory::alloc_value(Type::new(
            symbol.clone(),
            constructor.to_owned(),
            memory::alloc_value(body.clone()),
            impls.to_owned(),
        ));
        // defining type by name && full name
        if let Err(e) = (*self.types).define(addr, &symbol.name, Value::Type(t)) {
            error!(e);
        }
        if symbol.full_name.is_some() {
            if let Err(e) =
                (*self.types).define(addr, symbol.full_name.as_ref().unwrap(), Value::Type(t))
            {
                error!(e);
            }
        }
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
        symbol: &Symbol,
        body: &Chunk,
        table: *mut Table,
    ) -> Result<(), ControlFlow> {
        // allocating unit
        let unit =
            memory::alloc_value(Unit::new(symbol.clone(), memory::alloc_value(Table::new())));

        // unit value
        let unit_value = Value::Unit(unit);

        // guarding value in gc and registering it
        self.gc_guard(unit_value);
        self.gc_register(unit_value, table);
        // setting root for fields
        (*(*unit).fields).set_root(self.globals);
        // setting temp parent for fields
        (*(*unit).fields).parent = table;
        // inserting temp self
        (*(*unit).fields)
            .fields
            .insert("self".to_string(), unit_value);
        // executing body
        self.run(body, (*unit).fields)?;
        // deleting temp self
        (*(*unit).fields).fields.remove("self");
        // binding function
        self.bind_functions((*unit).fields, FnOwner::Unit(unit));
        // calling optional init fn
        let init_fn = "init";
        if (*(*unit).fields).exists(init_fn) {
            self.push(unit_value);
            self.op_call(addr, init_fn, true, false, &Chunk::new(vec![]), table)?
        }
        // defining unit by name and full name
        if let Err(e) = (*self.units).define(addr, &symbol.name, unit_value) {
            error!(e);
        }
        if symbol.full_name.is_some() {
            if let Err(e) =
                (*self.units).define(addr, symbol.full_name.as_ref().unwrap(), unit_value)
            {
                error!(e);
            }
        }
        // deleting temp parent
        (*(*unit).fields).parent = std::ptr::null_mut();
        // deleting gc guard
        self.gc_unguard();
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
        symbol: &Symbol,
        functions: &[TraitFn],
    ) -> Result<(), ControlFlow> {
        // allocating trait
        let _trait = memory::alloc_value(Trait::new(symbol.clone(), functions.to_owned()));
        // define trait by name and full name
        if let Err(e) = (*self.traits).define(addr, &symbol.name, Value::Trait(_trait)) {
            error!(e);
        }
        if symbol.full_name.is_some() {
            if let Err(e) = (*self.traits).define(
                addr,
                symbol.full_name.as_ref().unwrap(),
                Value::Trait(_trait),
            ) {
                error!(e);
            }
        }
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
        table: *mut Table,
    ) -> Result<(), ControlFlow> {
        // non-previous
        if !has_previous {
            self.run(value, table)?;
            let operand = self.pop(addr)?;
            if let Err(e) = (*table).define(addr, name, operand) {
                error!(e);
            }
        }
        // previous
        else {
            let previous = self.pop(addr)?;
            match previous {
                // define in instance
                Value::Instance(instance) => {
                    self.run(value, table)?;
                    let operand = self.pop(addr)?;
                    if let Err(e) = (*(*instance).fields).define(addr, name, operand) {
                        error!(e);
                    }
                }
                // define in unit
                Value::Unit(unit) => {
                    self.run(value, table)?;
                    let operand = self.pop(addr)?;
                    if let Err(e) = (*(*unit).fields).define(addr, name, operand) {
                        error!(e);
                    }
                }
                _ => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("{previous:?} is not a container."),
                        "you can define variable for unit or instance."
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
        table: *mut Table,
    ) -> Result<(), ControlFlow> {
        // non-previous
        if !has_previous {
            self.run(value, table)?;
            let operand = self.pop(addr)?;
            if let Err(e) = (*table).set(addr.clone(), name, operand) {
                error!(e);
            }
        }
        // previous
        else {
            let previous = self.pop(addr)?;
            match previous {
                // define in instance
                Value::Instance(instance) => {
                    self.run(value, table)?;
                    let operand = self.pop(addr)?;
                    if let Err(e) = (*(*instance).fields).set_local(addr, name, operand) {
                        error!(e);
                    }
                }
                // define in unit
                Value::Unit(unit) => {
                    self.run(value, table)?;
                    let operand = self.pop(addr)?;
                    if let Err(e) = (*(*unit).fields).set_local(addr, name, operand) {
                        error!(e);
                    }
                }
                _ => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("{previous:?} is not a container."),
                        "you can define variable for unit or instance."
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
        table: *mut Table,
    ) -> Result<(), ControlFlow> {
        // non-previous
        if !has_previous {
            // loads from
            let lookup_result;
            if (*table).has(name) {
                lookup_result = (*table).lookup(addr, name);
            } else if (*self.types).has(name) {
                lookup_result = (*self.types).find(addr, name);
            } else if (*self.traits).has(name) {
                lookup_result = (*self.traits).find(addr, name);
            } else {
                lookup_result = (*self.units).find(addr, name);
            }
            // error handling
            if let Err(e) = lookup_result {
                error!(e)
            } else if let Ok(value) = lookup_result {
                if !should_push {
                    return Ok(());
                }
                self.push(value);
            }
        }
        // previous
        else {
            let previous = self.pop(addr)?;
            match previous {
                // from instance
                Value::Instance(instance) => {
                    let lookup_result = (*(*instance).fields).find(addr, name);
                    if let Err(e) = lookup_result {
                        error!(e)
                    } else if let Ok(value) = lookup_result {
                        if !should_push {
                            return Ok(());
                        }
                        self.push(value);
                    }
                }
                // from unit
                Value::Unit(unit) => {
                    let lookup_result = (*(*unit).fields).find(addr, name);
                    if let Err(e) = lookup_result {
                        error!(e)
                    } else if let Ok(value) = lookup_result {
                        if !should_push {
                            return Ok(());
                        }
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
        table: *mut Table,
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
            table: *mut Table,
            call_table: *mut Table,
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
                    let operand = vm.pop(addr)?;
                    if let Err(e) = (*call_table).define(addr, param, operand) {
                        error!(e);
                    }
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
            table: *mut Table,
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
            let call_table = memory::alloc_value(Table::new());
            // parent and closure tables, to chain call_table
            // with current
            (*call_table).parent = table;
            (*call_table).closure = (*function).closure;
            // freeing call table
            defer! {
                memory::free_value(call_table);
            }
            // root & self
            if (*function).owner.is_some() {
                match (*function).owner.clone().unwrap() {
                    FnOwner::Unit(unit) => {
                        (*call_table).set_root((*unit).fields);
                        if let Err(e) = (*call_table).define(addr, "self", Value::Unit(unit)) {
                            error!(e);
                        }
                    }
                    FnOwner::Instance(instance) => {
                        (*call_table).set_root((*instance).fields);
                        if let Err(e) =
                            (*call_table).define(addr, "self", Value::Instance(instance))
                        {
                            error!(e);
                        }
                    }
                }
            } else {
                (*call_table).set_root(self.globals)
            }
            // passing args
            pass_arguments(
                self,
                addr,
                name,
                (*function).params.len(),
                args,
                (*function).params.clone(),
                table,
                call_table,
            )?;
            // running body
            if let Err(e) = self.run(&*(*function).body, call_table) {
                return match e {
                    // if return
                    ControlFlow::Return(val) => {
                        if should_push {
                            self.push(val);
                        }
                        Ok(())
                    }
                    _ => {
                        // otherwise, propagate
                        Err(e)
                    }
                };
            }
            Ok(())
        }
        // checking value is native
        else if let Value::Native(function) = callable {
            // call table
            let call_table = memory::alloc_value(Table::new());
            // parent and closure tables, to chain call_table
            // with current
            (*call_table).parent = table;
            // freeing
            defer! {
                memory::free_value(call_table);
            }
            // root to globals
            (*call_table).set_root(self.globals);
            // loading arguments to stack
            load_arguments(self, addr, name, (*function).params_amount, args, table)?;
            // calling native fn
            let native = (*function).function;
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
    /// calls value by name
    pub unsafe fn op_call(
        &mut self,
        addr: &Address,
        name: &str,
        has_previous: bool,
        should_push: bool,
        args: &Chunk,
        table: *mut Table,
    ) -> Result<(), ControlFlow> {
        // non-previous
        if !has_previous {
            let lookup_result = (*table).lookup(addr, name);
            if let Err(e) = lookup_result {
                error!(e)
            } else if let Ok(value) = lookup_result {
                self.call(addr, name, value, args, table, should_push)?;
            }
        }
        // previous
        else {
            let previous = self.pop(addr)?;
            match previous {
                // call from instance
                Value::Instance(instance) => {
                    let lookup_result = (*(*instance).fields).find(addr, name);

                    if let Err(e) = lookup_result {
                        error!(e)
                    } else if let Ok(value) = lookup_result {
                        self.call(addr, name, value, args, table, should_push)?;
                    }
                }
                // call from unit
                Value::Unit(unit) => {
                    let lookup_result = (*(*unit).fields).find(addr, name);

                    if let Err(e) = lookup_result {
                        error!(e)
                    } else if let Ok(value) = lookup_result {
                        self.call(addr, name, value, args, table, should_push)?;
                    }
                }
                _ => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("couldn't call {name} from {previous:?}."),
                        "you can call fn from unit, instance or foreign."
                    ))
                }
            }
        }
        Ok(())
    }

    /// Opcode: Duplicate
    /// duplicates value in stack
    ///
    unsafe fn op_duplicate(&mut self, addr: &Address) -> Result<(), ControlFlow> {
        let operand = self.pop(addr)?;
        self.push(operand);
        self.push(operand);
        Ok(())
    }

    /// Checking instance impls all traits,
    /// if instance doesn't impl some fn-s,
    /// adds default implementation if exists,
    /// otherwise raises error
    ///
    unsafe fn check_traits(&mut self, addr: &Address, instance: *mut Instance, table: *mut Table) {
        // type of instance, used to check traits
        let instance_type = (*instance).t;

        /// Gets trait by name
        unsafe fn get_trait(
            traits: *mut Table,
            addr: &Address,
            trait_name: &str,
        ) -> Option<*mut Trait> {
            let trait_result = (*traits).find(addr, trait_name);
            if let Err(e) = trait_result {
                error!(e);
            } else if let Ok(trait_value) = trait_result {
                match trait_value {
                    Value::Trait(_trait) => {
                        // перебираем функции
                        return Some(_trait);
                    }
                    _ => {
                        panic!("not a trait in traits table. report to developer.")
                    }
                }
            } else {
                return None;
            }
        }

        /// Gets impl by name
        unsafe fn get_impl(
            table: *mut Table,
            addr: &Address,
            impl_name: &str,
        ) -> Option<*mut Function> {
            // looking up for impl
            let fn_result = (*table).lookup(addr, impl_name);
            // checking existence
            if let Err(e) = fn_result {
                error!(e);
            } else if let Ok(trait_value) = fn_result {
                return match trait_value {
                    Value::Fn(_fn) => {
                        // перебираем функции
                        Some(_fn)
                    }
                    _ => None,
                };
            } else {
                return None;
            }
        }

        // checking all traits from a type
        for trait_name in &(*instance_type).impls {
            let _trait = get_trait(self.traits, addr, trait_name).unwrap();
            // checking all fn-s
            for function in &(*_trait).functions {
                // if impl exists, checking it
                if (*(*instance).fields).exists(&function.name) {
                    // checking impl
                    let _impl = get_impl((*instance).fields, addr, &function.name);

                    // if impl exists, checking params amount
                    if _impl.is_some() {
                        let implementation = _impl.unwrap();
                        if (*implementation).params.len() != function.params_amount {
                            error!(Error::own(
                                addr.clone(),
                                format!(
                                    "type {} impls {}, but fn {} has wrong impl.",
                                    (*instance_type).name.name,
                                    trait_name,
                                    function.name
                                ),
                                format!(
                                    "expected args {}, got {}",
                                    function.params_amount,
                                    (*implementation).params.len()
                                )
                            ));
                        }
                    } else {
                        error!(Error::own(
                            addr.clone(),
                            format!(
                                "type {} impls {}, but doesn't impl fn {}({})",
                                (*instance_type).name.name.clone(),
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
                        let default_fn = Value::Fn(memory::alloc_value(Function::new(
                            Symbol::by_name(function.name.clone()),
                            memory::alloc_value(default_impl.chunk.clone()),
                            default_impl.params.clone(),
                        )));
                        // guarding in gc
                        self.gc_guard(default_fn);
                        // registering in gc
                        self.gc_register(default_fn, table);
                        // defining fn in fields of instance
                        if let Err(e) =
                            (*(*instance).fields).define(addr, &function.name, default_fn)
                        {
                            error!(e);
                        }
                        // deleting gc guard gc
                        self.gc_unguard();
                    } else {
                        error!(Error::own(
                            addr.clone(),
                            format!(
                                "type {} impls {}, but doesn't impl fn {}({})",
                                (*instance_type).name.name,
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
        name: &str,
        args: &Chunk,
        should_push: bool,
        table: *mut Table,
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
            table: *mut Table,
            fields_table: *mut Table,
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
                    let operand = vm.pop(addr)?;
                    if let Err(e) = (*fields_table).define(addr, param, operand) {
                        error!(e);
                    }
                }
                Ok(())
            } else {
                error!(Error::own(
                    addr.clone(),
                    format!(
                        "invalid args amount: {passed_amount} to create instance of {name}."
                    ),
                    format!("expected {params_amount} arguments.")
                ));
            }
        }
        // looking up a type
        let lookup_result = (*self.types).lookup(addr, name);
        if let Ok(value) = lookup_result {
            match value {
                Value::Type(t) => {
                    // creating instance
                    let instance =
                        memory::alloc_value(Instance::new(t, memory::alloc_value(Table::new())));
                    let instance_value = Value::Instance(instance);
                    // guarding instance in gc
                    self.gc_guard(instance_value);
                    // registering in gc
                    self.gc_register(Value::Instance(instance), table);
                    // passing constructor
                    pass_constructor(
                        self,
                        addr,
                        name,
                        (*t).constructor.len(),
                        args,
                        (*t).constructor.clone(),
                        table,
                        (*instance).fields,
                    )?;
                    // setting root
                    (*(*instance).fields).set_root(self.globals);
                    // setting temp parent
                    (*(*instance).fields).parent = table;
                    // setting temp self
                    (*(*instance).fields)
                        .fields
                        .insert("self".to_string(), Value::Instance(instance));
                    // executing body
                    self.run(&*(*t).body, (*instance).fields)?;
                    // deleting temp self
                    (*(*instance).fields).fields.remove("self");
                    // checking traits implementation
                    self.check_traits(addr, instance, table);
                    // binding functions
                    self.bind_functions((*instance).fields, FnOwner::Instance(instance));
                    // calling optional init fn
                    let init_fn = "init";
                    if (*(*instance).fields).exists(init_fn) {
                        self.push(instance_value);
                        self.op_call(addr, init_fn, true, false, &Chunk::new(vec![]), table)?
                    }
                    // pushing instance
                    if should_push {
                        self.push(instance_value);
                    }
                    // deleting temp parent
                    (*(*instance).fields).parent = std::ptr::null_mut();
                    // unguarding from gc
                    self.gc_unguard();
                    Ok(())
                }
                _ => {
                    panic!(
                        "found a non-type value in types table. report this error to the developer."
                    )
                }
            }
        } else {
            error!(lookup_result.unwrap_err());
        }
    }

    /// Opcode: EndLoop
    #[allow(unused_variables)]
    unsafe fn op_endloop(
        &mut self,
        addr: &Address,
        current_iteration: bool,
    ) -> Result<(), ControlFlow> {
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
        table: *mut Table,
    ) -> Result<(), ControlFlow> {
        self.run(value, table)?;
        let value = self.pop(addr)?;

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
        // by `vm/natives/natives.rs`
        let result = (*self.natives).find(addr, name);

        if let Ok(value) = result {
            self.push(value);
        }
        if let Err(e) = result {
            error!(e);
        }

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
        table: *mut Table,
    ) -> Result<(), ControlFlow> {
        // running value
        self.run(value, table)?;
        let value = self.pop(addr)?;

        /// Calls is ok
        /// from an instance
        ///
        unsafe fn call_is_ok(
            vm: &mut VM,
            addr: &Address,
            instance: *mut Instance,
        ) -> Result<bool, ControlFlow> {
            let lookup_result = (*(*instance).fields).find(addr, "is_ok");

            if let Ok(callable) = lookup_result {
                if let Value::Fn(function) = callable {
                    if !(*function).params.is_empty() {
                        error!(Error::own_text(
                            addr.clone(),
                            format!("is_ok takes {} params", (*function).params.len()),
                            "is_ok should take 0 params."
                        ));
                    }
                } else {
                    error!(Error::new(
                        addr.clone(),
                        "is_ok is not a fn.",
                        "is_ok should be fn."
                    ));
                }
                vm.call(
                    addr,
                    "is_ok",
                    callable,
                    &Chunk::new(vec![]),
                    memory::alloc_value(Table::new()),
                    true,
                )?;

                let is_ok = vm.pop(addr)?;

                return if let Value::Bool(boolean) = is_ok {
                    Ok(boolean)
                } else {
                    error!(Error::own(
                        addr.clone(),
                        "is_ok should return a bool.".to_string(),
                        format!("it returned: {is_ok:?}")
                    ));
                };
            } else if let Err(e) = lookup_result {
                error!(e);
            }
            Ok(false)
        }

        /// Calls unwrap
        /// from an instance
        ///
        unsafe fn call_unwrap(
            vm: &mut VM,
            addr: &Address,
            instance: *mut Instance,
        ) -> Result<(), ControlFlow> {
            let lookup_result = (*(*instance).fields).find(addr, "unwrap");

            match lookup_result {
                Ok(callable) => {
                    if let Value::Fn(function) = callable {
                        if !(*function).params.is_empty() {
                            error!(Error::own_text(
                                addr.clone(),
                                format!("unwrap takes {} params", (*function).params.len()),
                                "unwrap should take 0 params."
                            ));
                        }
                    } else {
                        error!(Error::new(
                            addr.clone(),
                            "unwrap is not a fn.",
                            "unwrap should be fn."
                        ));
                    }
                    vm.call(
                        addr,
                        "unwrap",
                        callable,
                        &Chunk::new(vec![]),
                        memory::alloc_value(Table::new()),
                        true,
                    )?;
                    Ok(())
                }
                Err(e) => {
                    error!(e);
                }
            }
        }

        if let Value::Instance(instance) = value {
            // calling is ok
            let is_ok = call_is_ok(self, addr, instance)?;
            // if it's no ok
            if !is_ok {
                // returning value back
                return Err(ControlFlow::Return(value));
            } else {
                // calling unwrap
                call_unwrap(self, addr, instance)?;
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
    unsafe fn op_impls(
        &mut self,
        addr: &Address,
        value: &Chunk,
        trait_name: &str,
        table: *mut Table,
    ) -> Result<(), ControlFlow> {
        // running impl
        self.run(value, table)?;
        let value = self.pop(addr)?;

        // if value returned instance, checking trait
        // is implemented
        if let Value::Instance(instance) = value {
            // checking trait is implemented
            let lookup_result = (*self.traits).lookup(addr, trait_name);

            if let Ok(trait_value) = lookup_result {
                match trait_value {
                    Value::Trait(_trait) => {
                        let impls = &(*(*instance).t).impls;

                        let name = &(*_trait).name.name;
                        let full_name_option = &(*_trait).name.full_name;

                        if let Some(full_name) = full_name_option {
                            self.push(Value::Bool(
                                impls.contains(name) || impls.contains(full_name),
                            ));
                        } else {
                            self.push(Value::Bool(impls.contains(name)));
                        }
                    }
                    _ => {
                        panic!("not a trait in traits table. report to developer.")
                    }
                }
            }
            // если трейта не существует
            else if let Err(e) = lookup_result {
                error!(e);
            }
        } else {
            error!(Error::own_text(
                addr.clone(),
                format!("could not use impls with {value:?}."),
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
    unsafe fn op_delete_local(&self, addr: &Address, name: &String, table: *mut Table) {
        (*table).fields.remove(name);
    }

    /// Running chunk
    #[allow(unused_variables)]
    pub unsafe fn run(&mut self, chunk: &Chunk, table: *mut Table) -> Result<(), ControlFlow> {
        for op in chunk.opcodes() {
            match op {
                Opcode::Push { addr, value } => {
                    self.op_push(value.clone(), table)?;
                }
                Opcode::Pop { addr } => {
                    self.pop(addr)?;
                }
                Opcode::Bin { addr, op } => {
                    self.op_binary(addr, op, table)?;
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
                Opcode::Logic { addr, a, b, op } => self.op_logical(addr, a, b, op, table)?,
                Opcode::If {
                    addr,
                    cond,
                    body,
                    elif,
                } => {
                    self.op_if(addr, cond, body, elif, table)?;
                }
                Opcode::Loop { addr, body } => {
                    self.op_loop(addr, body, table)?;
                }
                Opcode::DefineFn {
                    addr,
                    name,
                    full_name,
                    body,
                    params,
                    make_closure,
                } => {
                    self.op_define_fn(
                        addr,
                        Symbol::new_option(name.clone(), full_name.clone()),
                        body,
                        params,
                        *make_closure,
                        table,
                    )?;
                }
                Opcode::AnonymousFn {
                    addr,
                    body,
                    params,
                    make_closure,
                } => {
                    self.op_anonymous_fn(body, params, *make_closure, table)?;
                }
                Opcode::DefineType {
                    addr,
                    name,
                    full_name,
                    body,
                    constructor,
                    impls,
                } => self.op_define_type(
                    addr,
                    &Symbol::new_option(name.clone(), full_name.clone()),
                    body,
                    constructor,
                    impls,
                )?,
                Opcode::DefineUnit {
                    addr,
                    name,
                    full_name,
                    body,
                } => self.op_define_unit(
                    addr,
                    &Symbol::new_option(name.clone(), full_name.clone()),
                    body,
                    table,
                )?,
                Opcode::DefineTrait {
                    addr,
                    name,
                    full_name,
                    functions,
                } => self.op_define_trait(
                    addr,
                    &Symbol::new_option(name.clone(), full_name.clone()),
                    functions,
                )?,
                Opcode::Define {
                    addr,
                    name,
                    value,
                    has_previous,
                } => {
                    self.op_define(addr, name, *has_previous, value, table)?;
                }
                Opcode::Set {
                    addr,
                    name,
                    value,
                    has_previous,
                } => {
                    self.op_set(addr, name, *has_previous, value, table)?;
                }
                Opcode::Load {
                    addr,
                    name,
                    has_previous,
                    should_push,
                } => {
                    self.op_load(addr, name, *has_previous, *should_push, table)?;
                }
                Opcode::Call {
                    addr,
                    name,
                    has_previous,
                    should_push,
                    args,
                } => self.op_call(addr, name, *has_previous, *should_push, args, table)?,
                Opcode::Duplicate { addr } => {
                    self.op_duplicate(addr)?;
                }
                Opcode::Instance {
                    addr,
                    name,
                    args,
                    should_push,
                } => {
                    self.op_instance(addr, name, args, *should_push, table)?;
                }
                Opcode::EndLoop {
                    addr,
                    current_iteration,
                } => {
                    self.op_endloop(addr, *current_iteration)?;
                }
                Opcode::Ret { addr, value } => {
                    self.op_return(addr, value, table)?;
                }
                Opcode::Native { addr, fn_name } => {
                    self.op_native(addr, fn_name)?;
                }
                Opcode::ErrorPropagation { addr, value } => {
                    self.op_error_propagation(addr, value, table)?;
                }
                Opcode::Impls {
                    addr,
                    value,
                    trait_name,
                } => {
                    self.op_impls(addr, value, trait_name, table)?;
                }
                Opcode::DeleteLocal { addr, name } => self.op_delete_local(addr, name, table),
            }
        }
        Ok(())
    }
}

/// Send & sync for future multi-threading.
unsafe impl Send for VM {}
unsafe impl Sync for VM {}
