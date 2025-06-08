use std::collections::VecDeque;
use crate::errors::{Error, ErrorType};
use crate::lexer::address::Address;
use crate::vm::bytecode::{Chunk, Opcode};
use crate::vm::flow::ControlFlow;
use crate::vm::{memory, natives};
use crate::vm::table::Table;
use crate::vm::values::{FnOwner, Function, Instance, Symbol, Type, Unit, Value};

// вм
pub struct VM {
    pub globals: *mut Table,
    stack: VecDeque<*mut Value>,
    types: *mut Table,
    units: *mut Table
}

// имплементация вм
#[allow(unused_qualifications)]
impl VM {
    // новая вм
    pub(crate) unsafe fn new() -> Result<VM, Error> {
        // вм
        let mut vm = VM {
            globals: memory::alloc_value(Table::new()),
            stack: VecDeque::new(),
            types: memory::alloc_value(Table::new()),
            units: memory::alloc_value(Table::new())
        };
        // нативы
        natives::provide(&mut vm)?;
        // возвращаем
        Ok(vm)
    }

    // пуш
    pub fn push(&mut self, value: *mut Value) {
        self.stack.push_back(value);
    }

    // поп
    pub fn pop(&mut self, address: Address) -> Result<*mut Value, ControlFlow> {
        if self.stack.len() == 0 {
            return Err(ControlFlow::Error(Error::new(
                ErrorType::Runtime,
                address,
                "stack underflow.".to_string(),
                "check your code.".to_string()
            )))
        }
        Ok(self.stack.pop_back().unwrap())
    }

    // бинды функций
    unsafe fn bind_functions(&mut self, table: *mut Table, owner: *mut FnOwner) {
        for val in (*table).fields.values() {
            let value = *val;
            if let Value::Fn(function) = *value {
                (*function).owner = owner;
            }
        }
    }

    // бинарная операция
    unsafe fn op_binary(&mut self, address: Address, op: &str) -> Result<(), ControlFlow> {
        // два операнда
        let operand_a = self.pop(address.clone())?;
        let operand_b = self.pop(address.clone())?;
        // ошибка
        let error = ControlFlow::Error(Error::new(
            ErrorType::Runtime,
            address.clone(),
            format!("could not use '{}' with {:?} and {:?}", op, *operand_a, *operand_b),
            "check your code.".to_string()
        ));
        // бинарная операция
        match op.clone() {
            "+" => {
                match (*operand_a) {
                    Value::Float(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Float(a + b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Float(a + (b as f64)))); }
                        _ => { return Err(error) }
                    }}
                    Value::Int(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Float((a as f64) + b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Int(a + b))); }
                        _ => { return Err(error) }
                    }}
                    _ => { return Err(error) }
                }
            }
            "-" => {
                match (*operand_a) {
                    Value::Float(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Float(a - b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Float(a - (b as f64)))); }
                        _ => { return Err(error) }
                    }}
                    Value::Int(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Float((a as f64) - b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Int(a - b))); }
                        _ => { return Err(error) }
                    }}
                    _ => { return Err(error) }
                }
            }
            "*" => {
                match (*operand_a) {
                    Value::Float(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Float(a * b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Float(a * (b as f64)))); }
                        _ => { return Err(error) }
                    }}
                    Value::Int(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Float((a as f64) * b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Int(a * b))); }
                        _ => { return Err(error) }
                    }}
                    _ => { return Err(error) }
                }
            }
            "/" => {
                match (*operand_a) {
                    Value::Float(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Float(a / b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Float(a / (b as f64)))); }
                        _ => { return Err(error) }
                    }}
                    Value::Int(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Float((a as f64) / b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Int(a / b))); }
                        _ => { return Err(error) }
                    }}
                    _ => { return Err(error) }
                }
            }
            "%" => {
                match (*operand_a) {
                    Value::Float(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Float(a % b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Float(a % (b as f64)))); }
                        _ => { return Err(error) }
                    }}
                    Value::Int(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Float((a as f64) % b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Int(a % b))); }
                        _ => { return Err(error) }
                    }}
                    _ => { return Err(error) }
                }
            }
            _ => { panic!("operator = {} is not found.", op)}
        }
        Ok(())
    }

    // негэйт
    unsafe fn op_negate(&mut self, address: Address) -> Result<(), ControlFlow> {
        // операнд
        let operand = self.pop(address.clone())?;
        // ошибка
        let error = ControlFlow::Error(Error::new(
            ErrorType::Runtime,
            address.clone(),
            format!("could not use 'negate' for {:?}", *operand),
            "check your code.".to_string()
        ));
        // негэйт
        match (*operand) {
            Value::Float(a) => {
                self.push(memory::alloc_value(Value::Float(-a)));
            }
            Value::Int(a) => {
                self.push(memory::alloc_value(Value::Int(-a)));
            }
            _ => { return Err(error) }
        }
        // успех
        Ok(())
    }

    // бэнг
    unsafe fn op_bang(&mut self, address: Address) -> Result<(), ControlFlow> {
        // операнд
        let operand = self.pop(address.clone())?;
        let error = ControlFlow::Error(Error::new(
            ErrorType::Runtime,
            address.clone(),
            format!("could not use 'bang' for {:?}", *operand),
            "check your code.".to_string()
        ));
        // бэнг
        match (*operand) {
            Value::Bool(b) => {
                self.push(memory::alloc_value(Value::Bool(!b)));
            }
            _ => { return Err(error) }
        }
        // успех
        Ok(())
    }

    // условие
    unsafe fn op_conditional(&mut self, address: Address, op: &str) -> Result<(), ControlFlow> {
        // операнды
        let operand_a = self.pop(address.clone())?;
        let operand_b = self.pop(address.clone())?;
        let error = ControlFlow::Error(Error::new(
            ErrorType::Runtime,
            address.clone(),
            format!("could not use '{}' for {:?} and {:?}", op, *operand_a, *operand_b),
            "check your code.".to_string()
        ));
        // условие
        match op {
            ">" => {
                match *operand_a {
                    Value::Float(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Bool(a > b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Bool(a > (b as f64)))); }
                        _ => { return Err(error); }
                    }}
                    Value::Int(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Bool((a as f64) > b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Bool(a > b))); }
                        _ => { return Err(error); }
                    }}
                    _ => { return Err(error); }
                }
            },
            "<" => {
                match *operand_a {
                    Value::Float(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Bool(a < b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Bool(a < (b as f64)))); }
                        _ => { return Err(error); }
                    }}
                    Value::Int(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Bool((a as f64) < b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Bool(a < b))); }
                        _ => { return Err(error); }
                    }}
                    _ => { return Err(error); }
                }
            },
            ">=" => {
                match *operand_a {
                    Value::Float(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Bool(a >= b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Bool(a >= (b as f64)))); }
                        _ => { return Err(error); }
                    }}
                    Value::Int(a) => { match (*operand_b) {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Bool((a as f64) >= b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Bool(a >= b))); }
                        _ => { return Err(error); }
                    }}
                    _ => { return Err(error); }
                }
            }
            "<=" => {
                match *operand_a {
                    Value::Float(a) => { match *operand_b {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Bool(a <= b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Bool(a <= (b as f64)))); }
                        _ => { return Err(error); }
                    }}
                    Value::Int(a) => { match *operand_b {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Bool((a as f64) <= b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Bool(a <= b))); }
                        _ => { return Err(error); }
                    }}
                    _ => { return Err(error) }
                }
            }
            "==" => {
                match *operand_a {
                    Value::Float(a) => { match *operand_b {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Bool(a == b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Bool(a == (b as f64)))); }
                        _ => { self.push(memory::alloc_value(Value::Bool(false))); }
                    }}
                    Value::Int(a) => { match *operand_b {
                        Value::Float(b) => { self.push(memory::alloc_value(Value::Bool((a as f64) == b))); }
                        Value::Int(b) => { self.push(memory::alloc_value(Value::Bool(a == b))); }
                        _ => { self.push(memory::alloc_value(Value::Bool(false))); }
                    }}
                    Value::Null => { match *operand_b {
                        Value::Null => { self.push(memory::alloc_value(Value::Bool(true))); }
                        _ => { self.push(memory::alloc_value(Value::Bool(false)));  }
                    }}
                    Value::Fn(f1) => { match *operand_b {
                        Value::Fn(f2) => { self.push(memory::alloc_value(Value::Bool(f1 == f2))); }
                        _ => { self.push(memory::alloc_value(Value::Bool(false))); }
                    }}
                    Value::Bool(a) => { match *operand_b {
                        Value::Bool(b) => { self.push(memory::alloc_value(Value::Bool(a == b))); }
                        _ => { self.push(memory::alloc_value(Value::Bool(false))); }
                    }}
                    Value::Instance(a) => { match *operand_b {
                        Value::Instance(b) => { self.push(memory::alloc_value(Value::Bool(a == b))); }
                        _ => { self.push(memory::alloc_value(Value::Bool(false))); }
                    }}
                    Value::Type(a) => { match *operand_b {
                        Value::Type(b) => { self.push(memory::alloc_value(Value::Bool(a == b)))}
                        _ => { self.push(memory::alloc_value(Value::Bool(false))); }
                    }}
                    Value::String(a) => { match *operand_b {
                        Value::String(b) => { self.push(memory::alloc_value(Value::Bool(a == b)))}
                        _ => { self.push(memory::alloc_value(Value::Bool(false))); }
                    }}
                    Value::Native(a) => { match *operand_b {
                        Value::Native(b) => { self.push(memory::alloc_value(Value::Bool(a == b)))}
                        _ => { self.push(memory::alloc_value(Value::Bool(false))); }
                    }}
                    _ => {
                        self.push(memory::alloc_value(Value::Bool(false)));
                    }
                }
            }
            _ => { panic!("operator = {} is not found.", op)}
        }
        // успех
        Ok(())
    }

    // логика
    unsafe fn op_logical(&mut self, address: Address, op: &str) -> Result<(), ControlFlow> {
        // операнды
        let operand_a = self.pop(address.clone())?;
        let operand_b = self.pop(address.clone())?;
        let error = ControlFlow::Error(Error::new(
            ErrorType::Runtime,
            address.clone(),
            format!("could not use '{}' for {:?} and {:?}", op, *operand_a, *operand_b),
            "check your code.".to_string()
        ));
        // логика
        match op {
            "and" => {
                match *operand_a {
                    Value::Bool(a) => {
                        match *operand_b {
                            Value::Bool(b) => { self.push(memory::alloc_value(Value::Bool(a && b))); }
                            _ => { return Err(error); }
                        }
                    }
                    _ => { return Err(error); }
                }
            }
            "or" => {
                match *operand_a {
                    Value::Bool(a) => {
                        match *operand_b {
                            Value::Bool(b) => { self.push(memory::alloc_value(Value::Bool(a || b))); }
                            _ => { return Err(error); }
                        }
                    }
                    _ => { return Err(error); }
                }
            }
            _ => { panic!("operator = {} is not found.", op)}
        }
        // успех
        Ok(())
    }

    // иф
    unsafe fn op_if(&mut self, addr: Address, cond: Box<Chunk>, body: Box<Chunk>,
                    elif: Option<Box<Opcode>>, root: *mut Table) -> Result<(), ControlFlow> {
        // таблица
        let mut table = memory::alloc_value(Table::new());
        (*table).set_root(root);
        // условие
        self.run(*cond, table)?;
        let bool = self.pop(addr.clone())?;
        // проверка
        if let Value::Bool(b) = *bool {
            if b {
                self.run(*body, table)?
            } else {
                if let Option::Some(else_if) = elif {
                    self.run(Chunk::of(*else_if), table)? // todo: chunk::of has high runtime cost!
                }
            }
        } else {
            return Err(ControlFlow::Error(Error::new(
                ErrorType::Runtime,
                addr.clone(),
                format!("condition provided not a bool: {:?}", bool),
                "condition should provide a bool.".to_string()
            )))
        }
        // успех
        Ok(())
    }

    // луп
    #[allow(unused_variables)]
    unsafe fn op_loop(&mut self, addr: Address, body: Box<Chunk>, root: *mut Table) -> Result<(), ControlFlow> {
        // таблица
        let mut table = memory::alloc_value(Table::new());
        (*table).set_root(root);
        // проверка
        loop {
            if let Err(e) = self.run(*body.clone(), table) {
                match e {
                    ControlFlow::Continue => {
                        continue;
                    },
                    ControlFlow::Break => {
                        break;
                    }
                    _ => {
                        return Err(e);
                    }
                }
            }
        }
        // успех
        Ok(())
    }

    // дефайн функции
    unsafe fn op_define_fn(&mut self, addr: Address, symbol: Symbol, body: Box<Chunk>,
                        params: Vec<String>, table: *mut Table) -> Result<(), ControlFlow> {
        // создаём функцию
        let function = memory::alloc_value(
            Function::new(
                symbol.clone(),
                memory::alloc_value(*body),
                params
            )
        );
        // дефайн функции
        if let Err(e) = (*table).define(addr.clone(), symbol.name.clone(), memory::alloc_value(
            Value::Fn(function))
        ) {
            return Err(ControlFlow::Error(e))
        }
        // дефайн функции по full-name
        if symbol.full_name.is_some() {
            if let Err(e) = (*table).define(addr.clone(), symbol.full_name.unwrap(), memory::alloc_value(
                Value::Fn(function))
            ) {
                return Err(ControlFlow::Error(e))
            }
        }
        // успех
        Ok(())
    }

    // дефайн типа
    unsafe fn op_define_type(&mut self, addr: Address, symbol: Symbol, body: Box<Chunk>,
                             constructor: Vec<String>) -> Result<(), ControlFlow> {
        // создаём тип
        let t = memory::alloc_value(
            Type::new(
                symbol.clone(),
                constructor,
                memory::alloc_value(*body),
            )
        );
        // дефайн типа
        if let Err(e) = (*self.types).define(addr.clone(), symbol.name.clone(), memory::alloc_value(
            Value::Type(t))
        ) {
            return Err(ControlFlow::Error(e))
        }
        // дефайн по full-name
        if symbol.full_name.is_some() {
            if let Err(e) = (*self.types).define(addr.clone(), symbol.full_name.unwrap().clone(), memory::alloc_value(
                Value::Type(t))
            ) {
                return Err(ControlFlow::Error(e))
            }
        }
        // успех
        Ok(())
    }

    // дефайн юнита
    unsafe fn op_define_unit(&mut self, addr: Address, symbol: Symbol,
                             body: Box<Chunk>, table: *mut Table) -> Result<(), ControlFlow> {
        // создаём юнит
        let mut unit = memory::alloc_value(
            Unit::new(
                symbol.clone(),
                memory::alloc_value(Table::new())
            )
        );
        // временный рут
        (*(*unit).fields).set_root(table);
        // исполняем тело
        self.run(*body, table)?;
        // удаляем рут
        (*(*unit).fields).del_root();
        // бинды
        self.bind_functions((*unit).fields, memory::alloc_value(FnOwner::Unit(unit)));
        // дефайн юнита
        if let Err(e) = (*self.units).define(addr.clone(), symbol.name.clone(), memory::alloc_value(
            Value::Unit(unit))
        ) {
            return Err(ControlFlow::Error(e))
        }
        // дефайн по full-name
        if symbol.full_name.is_some() {
            if let Err(e) = (*self.units).define(addr.clone(), symbol.full_name.unwrap().clone(), memory::alloc_value(
                Value::Unit(unit))
            ) {
                return Err(ControlFlow::Error(e))
            }
        }
        // успех
        Ok(())
    }

    // дефайн
    unsafe fn op_define(&mut self, addr: Address, name: String, has_previous: bool,
                        value: Box<Chunk>, table: *mut Table) -> Result<(), ControlFlow> {
        // если нет предыдущего
        if !has_previous {
            // исполняем значение
            self.run(*value, table)?;
            // получаем значение
            let operand = self.pop(addr.clone())?;
            // дефайним
            if let Err(e) = (*table).define(addr.clone(), name, operand) {
                return Err(ControlFlow::Error(e));
            }
        }
        // если есть
        else {
            // получаем значение
            let previous = self.pop(addr.clone())?;
            // првоеряем
            match *previous {
                Value::Instance(instance) => {
                    // исполняем значение
                    self.run(*value, table)?;
                    // получаем значение
                    let operand = self.pop(addr.clone())?;
                    // дефайним
                    if let Err(e) = (*(*instance).fields).define(addr.clone(), name, operand) {
                        return Err(ControlFlow::Error(e))
                    }
                }
                Value::Unit(unit) => {
                    // исполняем значение
                    self.run(*value, table)?;
                    // получаем значение
                    let operand = self.pop(addr.clone())?;
                    // дефайним
                    if let Err(e) = (*(*unit).fields).define(addr.clone(), name, operand) {
                        return Err(ControlFlow::Error(e))
                    }
                }
                _ => {
                    return Err(ControlFlow::Error(Error::new(
                        ErrorType::Runtime,
                        addr.clone(),
                        format!("{:?} is not a container.", *previous),
                        "you can define variable for unit or instance.".to_string()
                    )))
                }
            }
        }
        // успех
        Ok(())
    }

    // установка значения переменной
    unsafe fn op_set(&mut self, addr: Address, name: String, has_previous: bool,
                        value: Box<Chunk>, table: *mut Table) -> Result<(), ControlFlow> {
        // если нет предыдущего
        if !has_previous {
            // исполняем значение
            self.run(*value, table)?;
            // получаем значение
            let operand = self.pop(addr.clone())?;
            // дефайним
            if let Err(e) = (*table).set(addr.clone(), name, operand) {
                return Err(ControlFlow::Error(e));
            }
        }
        // если есть
        else {
            // получаем значение
            let previous = self.pop(addr.clone())?;
            // проверяем
            match *previous {
                Value::Instance(instance) => {
                    // исполняем значение
                    self.run(*value, table)?;
                    // получаем значение
                    let operand = self.pop(addr.clone())?;
                    // дефайним
                    if let Err(e) = (*(*instance).fields).set(addr.clone(), name, operand) {
                        return Err(ControlFlow::Error(e))
                    }
                }
                Value::Unit(unit) => {
                    // исполняем значение
                    self.run(*value, table)?;
                    // получаем значение
                    let operand = self.pop(addr.clone())?;
                    // дефайним
                    if let Err(e) = (*(*unit).fields).set(addr.clone(), name, operand) {
                        return Err(ControlFlow::Error(e))
                    }
                }
                _ => {
                    return Err(ControlFlow::Error(Error::new(
                        ErrorType::Runtime,
                        addr.clone(),
                        format!("{:?} is not a container.", *previous),
                        "you can define variable for unit or instance.".to_string()
                    )))
                }
            }
        }
        // успех
        Ok(())
    }

    // загрузка значения переменной
    unsafe fn op_load(&mut self, addr: Address, name: String, has_previous: bool,
                      should_push: bool, table: *mut Table) -> Result<(), ControlFlow> {
        // если нет предыдущего
        if !has_previous {
            // получаем значение
            let lookup_result = (*table).lookup(addr.clone(), name);
            // проверяем на ошибку
            if let Err(e) = lookup_result {
                // ошибка
                return Err(ControlFlow::Error(e))
            }
            else if let Ok(value) = lookup_result {
                // пушим в стек
                if !should_push { return Ok(()) }
                self.push(value);
            }
        }
        // если есть
        else {
            // получаем значение
            let previous = self.pop(addr.clone())?;
            // проверяем
            match *previous {
                Value::Instance(instance) => {
                    // получаем значение
                    let lookup_result = (*(*instance).fields).lookup(addr.clone(), name);
                    // проверяем на ошибку
                    if let Err(e) = lookup_result {
                        // ошибка
                        return Err(ControlFlow::Error(e))
                    }
                    else if let Ok(value) = lookup_result {
                        // пушим в стек
                        if !should_push { return Ok(()) }
                        self.push(value);
                    }
                }
                Value::Unit(unit) => {
                    // получаем значение
                    let lookup_result = (*(*unit).fields).lookup(addr.clone(), name);
                    // проверяем на ошибку
                    if let Err(e) = lookup_result {
                        // ошибка
                        return Err(ControlFlow::Error(e))
                    }
                    else if let Ok(value) = lookup_result {
                        // пушим в стек
                        if !should_push { return Ok(()) }
                        self.push(value);
                    }
                }
                _ => {
                    return Err(ControlFlow::Error(Error::new(
                        ErrorType::Runtime,
                        addr.clone(),
                        format!("{:?} is not a container.", *previous),
                        "you can load variable from unit or instance.".to_string()
                    )))
                }
            }
        }
        // успех
        Ok(())
    }

    // загрузка значения переменной
    unsafe fn op_call(&mut self, addr: Address, name: String, has_previous: bool,
                      should_push: bool, args: Box<Chunk>, table: *mut Table) -> Result<(), ControlFlow> {

        // подгрузка аргументов
        unsafe fn pass_arguments(vm: &mut VM, addr: Address, name: String, params_amount: usize,
                                 args: Box<Chunk>, params: Vec<String>, table: *mut Table) -> Result<(), ControlFlow> {
            // фиксируем размер стека
            let prev_size = vm.stack.len();
            // загрузка аргументов
            vm.run(*args, table)?;
            // фиксируем новый размер стека
            let new_size = vm.stack.len();
            // количество переданных аргументов
            let passed_amount = new_size-prev_size;
            // проверяем
            if passed_amount == params_amount {
                // реверсируем параметры
                let mut reversed_params = params.clone();
                reversed_params.reverse();
                // проходимся
                for param in reversed_params {
                    // получаем аргумент из стека
                    let operand = vm.pop(addr.clone())?;
                    // устанавливаем в таблице
                    if let Err(e) = (*table).define(addr.clone(), param.clone(), operand) {
                        return Err(ControlFlow::Error(e));
                    }
                }
                Ok(())
            } else {
                Err(ControlFlow::Error(Error::new(
                    ErrorType::Runtime,
                    addr.clone(),
                    format!("invalid args amount: {} to call: {}.", passed_amount, name),
                    format!("expected {} arguments.", params_amount)
                )))
            }
        }

        // только загрузка аргументов
        unsafe fn load_arguments(vm: &mut VM, addr: Address, name: String, params_amount: usize,
                                  args: Box<Chunk>, table: *mut Table) -> Result<(), ControlFlow> {
            // фиксируем размер стека
            let prev_size = vm.stack.len();
            // загрузка аргументов
            vm.run(*args, table)?;
            // фиксируем новый размер стека
            let new_size = vm.stack.len();
            // количество переданных аргументов
            let passed_amount = new_size-prev_size;
            // проверяем
            if passed_amount == params_amount {
                Ok(())
            } else {
                Err(ControlFlow::Error(Error::new(
                    ErrorType::Runtime,
                    addr.clone(),
                    format!("invalid args amount: {} to call: {}.", passed_amount, name),
                    format!("expected {} arguments.", params_amount)
                )))
            }
        }

        // вызов функции
        unsafe fn call(vm: &mut VM, addr: Address, name: String,
                   callable: *mut Value, args: Box<Chunk>,
                   table: *mut Table, should_push: bool) -> Result<(), ControlFlow> {
            // проверка на функцию
            if let Value::Fn(function) = *callable {
                // создаём таблицу под вызов.
                let call_table = memory::alloc_value(Table::new());
                // рут и self
                if !(*function).owner.is_null() {
                    match (*(*function).owner) {
                        FnOwner::Unit(unit) => {
                            (*call_table).set_root((*unit).fields);
                            if let Err(e) = (*(*unit).fields).define(
                                addr.clone(), "self".to_string(), memory::alloc_value(Value::Unit(unit))
                            ) {
                                return Err(ControlFlow::Error(e));
                            }
                        },
                        FnOwner::Instance(instance) => {
                            (*call_table).set_root((*instance).fields);
                            if let Err(e) = (*(*instance).fields).define(
                                addr.clone(), "self".to_string(), memory::alloc_value(Value::Instance(instance))
                            ) {
                                return Err(ControlFlow::Error(e));
                            }
                        }
                    }
                } else {
                    (*call_table).set_root(table)
                }
                // замыкание
                (*call_table).closure = (*function).closure;
                // загрузка аргументов
                pass_arguments(vm, addr, name, (*function).params.len(), args,
                               (*function).params.clone(), call_table)?;
                // вызов
                match vm.run((*(*function).body).clone(), call_table) {
                    Err(e) => {
                        match e {
                            ControlFlow::Return(val) => {
                                if should_push {
                                    vm.push(val);
                                }
                            },
                            _ => {
                                return Err(e);
                            }
                        }
                    }
                    _ => {}
                }
                // успех
                Ok(())
            }
            // проверка на нативную функцию
            else if let Value::Native(function) = *callable {
                // создаём таблицу под вызов.
                let call_table = memory::alloc_value(Table::new());
                (*call_table).set_root(table);
                // загрузка аргументов
                load_arguments(vm, addr.clone(), name.clone(), (*function).params_amount, args, call_table)?;
                // вызов
                let callable = (*function).function;
                callable(vm, addr.clone(), should_push)?;
                // успех
                Ok(())
            }
            else {
                Err(ControlFlow::Error(Error::new(
                    ErrorType::Runtime,
                    addr.clone(),
                    format!("{} is not a fn.", name),
                    "you can call only fn-s.".to_string()
                )))
            }
        }

        // если нет предыдущего
        if !has_previous {
            // получаем значение
            let lookup_result = (*table).lookup(addr.clone(), name.clone());
            // проверяем на ошибку
            if let Err(e) = lookup_result {
                // ошибка
                return Err(ControlFlow::Error(e))
            }
            else if let Ok(value) = lookup_result {
                // вызываем
                call(self, addr.clone(), name.clone(), value, args.clone(), table, should_push)?;
            }
        }
        // если есть
        else {
            // получаем значение
            let previous = self.pop(addr.clone())?;
            // проверяем
            match *previous {
                Value::Instance(instance) => {
                    // получаем значение
                    let lookup_result = (*(*instance).fields).lookup(addr.clone(), name.clone());
                    // проверяем на ошибку
                    if let Err(e) = lookup_result {
                        // ошибка
                        return Err(ControlFlow::Error(e))
                    }
                    else if let Ok(value) = lookup_result {
                        // вызываем
                        call(self, addr.clone(), name.clone(), value, args.clone(), table, should_push)?;
                    }
                }
                Value::Unit(unit) => {
                    // получаем значение
                    let lookup_result = (*(*unit).fields).lookup(addr.clone(), name.clone());
                    // проверяем на ошибку
                    if let Err(e) = lookup_result {
                        // ошибка
                        return Err(ControlFlow::Error(e))
                    }
                    else if let Ok(value) = lookup_result {
                        // вызываем
                        call(self, addr.clone(), name.clone(), value, args.clone(), table, should_push)?;
                    }
                }
                _ => {
                    return Err(ControlFlow::Error(Error::new(
                        ErrorType::Runtime,
                        addr.clone(),
                        format!("{:?} is not a container.", *previous),
                        "you can load variable from unit or instance.".to_string()
                    )))
                }
            }
        }
        // успех
        Ok(())
    }

    // дублирование значения в стеке
    unsafe fn op_duplicate(&mut self, addr: Address) -> Result<(), ControlFlow> {
        // операнд
        let operand = self.pop(addr)?;
        // пушим
        self.push(operand);
        self.push(operand);
        // успех
        Ok(())
    }

    // созедание экземпляра типа
    unsafe fn op_instance(&mut self, addr: Address, name: String,
                          args: Box<Chunk>, should_push: bool, table: *mut Table) -> Result<(), ControlFlow> {

        // подгрузка конструктора
        unsafe fn pass_constructor(vm: &mut VM, addr: Address, name: String, params_amount: usize,
                                 args: Box<Chunk>, params: Vec<String>, table: *mut Table) -> Result<(), ControlFlow> {
            // фиксируем размер стека
            let prev_size = vm.stack.len();
            // загрузка аргументов
            vm.run(*args, table)?;
            // фиксируем новый размер стека
            let new_size = vm.stack.len();
            // количество переданных аргументов
            let passed_amount = new_size-prev_size;
            // проверяем
            if passed_amount == params_amount {
                // реверсируем параметры
                let mut reversed_params = params.clone();
                reversed_params.reverse();
                // проходимся
                for param in reversed_params {
                    // получаем аргумент из стека
                    let operand = vm.pop(addr.clone())?;
                    // устанавливаем в таблице
                    if let Err(e) = (*table).define(addr.clone(), param.clone(), operand) {
                        return Err(ControlFlow::Error(e));
                    }
                }
                Ok(())
            } else {
                Err(ControlFlow::Error(Error::new(
                    ErrorType::Runtime,
                    addr.clone(),
                    format!("invalid args amount: {} to create instance of {}.", passed_amount, name),
                    format!("expected {} arguments.", params_amount)
                )))
            }
        }
        // ищем тип
        let lookup_result = (*self.types).lookup(addr.clone(), name.clone());
        // проверяем, найден ли
        if let Ok(value) = lookup_result {
            // проверяем тип ли
            match *value {
                Value::Type(t) => {
                    // создаём экземпляр
                    let instance = memory::alloc_value(Instance::new(
                        t,
                        memory::alloc_value(Table::new()),
                    ));
                    // временный рут
                    (*(*instance).fields).set_root(table);
                    // исполняем тело
                    self.run((*(*t).body).clone(), table)?;
                    // удаляем рут
                    (*(*instance).fields).del_root();
                    // конструктор
                    pass_constructor(
                        self,
                        addr.clone(),
                        name,
                        (*t).constructor.len(),
                        args,
                        (*t).constructor.clone(),
                        table
                    )?;
                    // бинды
                    self.bind_functions((*instance).fields, memory::alloc_value(FnOwner::Instance(instance)));
                    // значение экземпляра
                    let instance_value = memory::alloc_value(Value::Instance(
                        instance
                    ));
                    // init функция
                    let init_fn = "init".to_string();
                    if (*(*instance).fields).exists(init_fn.clone()) {
                        // пушим инстанс
                        self.push(instance_value);
                        // вызываем
                        let args = Box::new(Chunk::new(vec![]));
                        self.op_call(addr, init_fn, true, false, args, table)?
                    }
                    // пушим
                    if should_push {
                        self.push(instance_value);
                    }
                    // успех
                    Ok(())
                }
                _ => {
                    panic!("found a non-type value in types table.")
                }
            }
        }
        else {
            Err(ControlFlow::Error(lookup_result.unwrap_err()))
        }
    }

    // окончание цикла
    #[allow(unused_variables)]
    unsafe fn op_endloop(&mut self, addr: Address, current_iteration: bool) -> Result<(), ControlFlow> {
        if current_iteration {
            Err(ControlFlow::Continue)
        } else {
            Err(ControlFlow::Break)
        }
    }

    // создание замыкания
    unsafe fn op_make_closure(&mut self, addr: Address, name: String, table: *mut Table) -> Result<(), ControlFlow> {
        // ищем
        let lookup_result = (*table).lookup(addr.clone(), name.clone());
        // проверяем, нашло ли
        if let Ok(value) = lookup_result {
            // проверяем, функция ли
            if let Value::Fn(function) = *value {
                // устанавливаем замыкание
                (*function).closure = table;
                // успех
                Ok(())
            }
            else {
                // ошибка
                Err(ControlFlow::Error(Error::new(
                    ErrorType::Runtime,
                    addr.clone(),
                    format!("could not make closure for: {}", name.clone()),
                    "not a function.".to_string()
                )))
            }
        }
        else {
            Err(ControlFlow::Error(
                lookup_result.unwrap_err()
            ))
        }
    }

    // возврат значения из функции
    unsafe fn op_return(&mut self, addr: Address, value: Box<Chunk>, table: *mut Table) -> Result<(), ControlFlow> {
        // выполняем
        self.run(*value.clone(), table)?;
        let value = self.pop(addr)?;
        // возвращаем
        Err(ControlFlow::Return(value))
    }

    // запуск байткода
    #[allow(unused_variables)]
    pub unsafe fn run(&mut self, chunk: Chunk, table: *mut Table) -> Result<(), ControlFlow> {
        for op in chunk.opcodes() {
            match op {
                Opcode::Push { addr, value } => {
                    self.push(memory::alloc_value(value));
                }
                Opcode::Pop { addr } => {
                    self.pop(addr.clone())?;
                }
                Opcode::Bin { addr, op } => {
                    self.op_binary(addr, op.as_str())?;
                }
                Opcode::Neg { addr } => {
                    self.op_negate(addr)?;
                }
                Opcode::Bang { addr } => {
                    self.op_bang(addr)?;
                }
                Opcode::Cond { addr, op } => {
                    self.op_conditional(addr, op.as_str())?;
                }
                Opcode::Logic { addr, op } => {
                    self.op_logical(addr, op.as_str())?
                }
                Opcode::If { addr, cond, body, elif } => {
                    self.op_if(addr, cond, body, elif, table)?;
                }
                Opcode::Loop { addr, body } => {
                    self.op_loop(addr, body, table)?;
                }
                Opcode::DefineFn { addr, name, full_name, body, params } => {
                    self.op_define_fn(addr, Symbol::new_option(name, full_name), body, params, table)?;
                }
                Opcode::DefineType { addr, name, full_name, body, constructor } => {
                    self.op_define_type(addr, Symbol::new_option(name, full_name), body, constructor)?
                }
                Opcode::DefineUnit { addr, name, full_name, body } => {
                    self.op_define_unit(addr, Symbol::new_option(name, full_name), body, table)?
                }
                Opcode::Define { addr, name, value, has_previous} => {
                    self.op_define(addr, name, has_previous, value, table)?;
                }
                Opcode::Set { addr, name, value, has_previous } => {
                    self.op_set(addr, name, has_previous, value, table)?;
                }
                Opcode::Load { addr, name, has_previous, should_push } => {
                    self.op_load(addr, name, has_previous, should_push, table)?;
                }
                Opcode::Call { addr, name, has_previous, should_push, args } => {
                    self.op_call(addr, name, has_previous, should_push, args, table)?
                }
                Opcode::Duplicate { addr } => {
                    self.op_duplicate(addr)?;
                }
                Opcode::Instance { addr, name, args, should_push } => {
                    self.op_instance(addr, name, args, should_push, table)?;
                }
                Opcode::EndLoop { addr, current_iteration } => {
                    self.op_endloop(addr, current_iteration)?;
                }
                Opcode::Closure { addr, name } => {
                    self.op_make_closure(addr, name, table)?;
                }
                Opcode::Ret { addr, value } => {
                    self.op_return(addr, value, table)?;
                }
            }
        }
        Ok(())
    }
}