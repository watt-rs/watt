// импорты
use std::collections::VecDeque;
use crate::error;
use crate::errors::errors::{Error};
use crate::lexer::address::Address;
use crate::vm::bytecode::{Chunk, Opcode};
use crate::vm::flow::ControlFlow;
use crate::vm::natives::natives;
use crate::vm::table::Table;
use crate::vm::values::{FnOwner, Function, Instance, Symbol, Trait, TraitFn, Type, Unit, Value};
use crate::vm::memory::gc::GC;
use crate::vm::memory::memory;

// настройки
#[derive(Debug)]
pub struct VmSettings {
    gc_threshold: usize,
    gc_debug: bool,
}
// имплементация
impl VmSettings {
    pub fn new(gc_threshold: usize, gc_debug: bool) -> Self {
        Self { gc_threshold, gc_debug }
    }
}

// вм
#[derive(Debug)]
pub struct VM {
    pub globals: *mut Table,
    types: *mut Table,
    pub units: *mut Table,
    traits: *mut Table,
    pub natives: *mut Table,
    pub gc: *mut GC,
    settings: VmSettings,
    pub stack: VecDeque<Value>,
}
// имплементация вм
#[allow(non_upper_case_globals)]
#[allow(unused_qualifications)]
impl VM {
    // новая вм
    pub unsafe fn new(settings: VmSettings) -> VM {
        // вм
        let mut vm = VM {
            globals: memory::alloc_value(Table::new()),
            types: memory::alloc_value(Table::new()),
            units: memory::alloc_value(Table::new()),
            traits: memory::alloc_value(Table::new()),
            natives: memory::alloc_value(Table::new()),
            gc: memory::alloc_value(GC::new(settings.gc_debug)),
            stack: VecDeque::new(),
            settings
        };
        // нативы
        if let Err(e) = natives::provide_builtins(&mut vm) {
            error!(e)
        }
        // возвращаем
        vm
    }

    // пуш
    pub unsafe fn push(&mut self, value: Value) {
        self.stack.push_back(value);
    }

    // поп
    pub fn pop(&mut self, address: Address) -> Result<Value, ControlFlow> {
        if self.stack.len() == 0 {
            error!(Error::new(
                address,
                "stack underflow.".to_string(),
                "check your code.".to_string()
            ));
        }
        Ok(self.stack.pop_back().unwrap())
    }

    // очистка мусора
    pub unsafe fn gc_invoke(&mut self, table: *mut Table) {
        (*self.gc).collect_garbage(self, table);
    }

    // бинды функций
    unsafe fn bind_functions(&mut self, table: *mut Table, owner: *mut FnOwner) {
        // биндим
        for val in (*table).fields.values() {
            if let Value::Fn(function) = *val {
                (*function).owner = owner;
            }
            else if let Value::Native(function) = *val {
                (*function).owner = owner;
            }
        }
    }

    // добавление в учет сборщика мусора
    pub unsafe fn gc_register(&mut self, value: Value, table: *mut Table) {
        // gil
        // добавляем объект
        (*self.gc).add_object(value);
        // проверяем порог gc
        if (*self.gc).objects_amount() > self.settings.gc_threshold {
            // вызываем gc
            self.gc_invoke(table);
            // увеличиваем порог
            self.settings.gc_threshold *= 2;
        }
    }
    
    // пуш в стек
    pub unsafe fn op_push(&mut self, value: Value, table: *mut Table) -> Result<(), ControlFlow> {
        // проверяем значение
        match value {
            Value::Int(_) | Value::Float(_) | Value::Bool(_) => {
                self.push(value);
            }
            Value::String(s) => {
                let new_string = Value::String(
                    memory::alloc_value(
                        (*s).clone()
                    )
                );
                self.gc_register(new_string, table);
                self.push(new_string);
            }
            _ => {
                self.gc_register(value, table);
                self.push(value);
            }
        }
        // успех
        Ok(())
    }

    // бинарная операция
    unsafe fn op_binary(&mut self, address: Address, op: &str, table: *mut Table) -> Result<(), ControlFlow> {
        // два операнда
        let operand_a = self.pop(address.clone())?;
        let operand_b = self.pop(address.clone())?;
        // ошибка
        let error = Error::new(
            address.clone(),
            format!("could not use '{}' with {:?} and {:?}", op, operand_a, operand_b),
            "check your code.".to_string()
        );
        // бинарная операция
        match op {
            "+" => {
                match operand_a {
                    Value::Float(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float(a + b)); }
                        Value::Int(b) => { self.push(Value::Float(a + (b as f64))); }
                        _ => { error!(error); }
                    }}
                    Value::Int(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float((a as f64) + b)); }
                        Value::Int(b) => { self.push(Value::Int(a + b)); }
                        _ => { error!(error); }
                    }}
                    Value::String(a) => {
                        let string = Value::String(
                            memory::alloc_value(format!("{}{:?}", *a, operand_b))
                        );
                        self.gc_register(string, table);
                        self.push(string);
                    }
                    _ => { error!(error); }
                }
            }
            "-" => {
                match operand_a {
                    Value::Float(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float(a - b)); }
                        Value::Int(b) => { self.push(Value::Float(a - (b as f64))); }
                        _ => { error!(error); }
                    }}
                    Value::Int(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float((a as f64) - b)); }
                        Value::Int(b) => { self.push(Value::Int(a - b)); }
                        _ => { error!(error); }
                    }}
                    _ => { error!(error); }
                }
            }
            "*" => {
                match operand_a {
                    Value::Float(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float(a * b)); }
                        Value::Int(b) => { self.push(Value::Float(a * (b as f64))); }
                        _ => { error!(error); }
                    }}
                    Value::Int(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float((a as f64) * b)); }
                        Value::Int(b) => { self.push(Value::Int(a * b)); }
                        _ => { error!(error); }
                    }}
                    _ => { error!(error); }
                }
            }
            "/" => {
                match operand_a {
                    Value::Float(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float(a / b)); }
                        Value::Int(b) => { self.push(Value::Float(a / (b as f64))); }
                        _ => { error!(error); }
                    }}
                    Value::Int(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float((a as f64) / b)); }
                        Value::Int(b) => { self.push(Value::Int(a / b)); }
                        _ => { error!(error); }
                    }}
                    _ => { error!(error); }
                }
            }
            "%" => {
                match operand_a {
                    Value::Float(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float(a % b)); }
                        Value::Int(b) => { self.push(Value::Float(a % (b as f64))); }
                        _ => { error!(error); }
                    }}
                    Value::Int(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float((a as f64) % b)); }
                        Value::Int(b) => { self.push(Value::Int(a % b)); }
                        _ => { error!(error); }
                    }}
                    _ => { error!(error); }
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
        let error = Error::new(
            address.clone(),
            format!("could not use 'negate' for {:?}", operand),
            "check your code.".to_string()
        );
        // негэйт
        match operand {
            Value::Float(a) => {
                self.push(Value::Float(-a));
            }
            Value::Int(a) => {
                self.push(Value::Int(-a));
            }
            _ => { error!(error); }
        }
        // успех
        Ok(())
    }

    // бэнг
    unsafe fn op_bang(&mut self, address: Address) -> Result<(), ControlFlow> {
        // операнд
        let operand = self.pop(address.clone())?;
        let error = Error::new(
            address.clone(),
            format!("could not use 'bang' for {:?}", operand),
            "check your code.".to_string()
        );
        // бэнг
        match operand {
            Value::Bool(b) => {
                self.push(Value::Bool(!b));
            }
            _ => { error!(error); }
        }
        // успех
        Ok(())
    }

    // условие
    unsafe fn op_conditional(&mut self, address: Address, op: &str) -> Result<(), ControlFlow> {
        // операнды
        let operand_a = self.pop(address.clone())?;
        let operand_b = self.pop(address.clone())?;
        let error = Error::new(
            address.clone(),
            format!("could not use '{}' for {:?} and {:?}", op, operand_a, operand_b),
            "check your code.".to_string()
        );
        // условие
        match op {
            ">" => {
                match operand_a {
                    Value::Float(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Bool(a > b)); }
                        Value::Int(b) => { self.push(Value::Bool(a > (b as f64))); }
                        _ => { error!(error); }
                    }}
                    Value::Int(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Bool((a as f64) > b)); }
                        Value::Int(b) => { self.push(Value::Bool(a > b)); }
                        _ => { error!(error); }
                    }}
                    _ => { error!(error); }
                }
            },
            "<" => {
                match operand_a {
                    Value::Float(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Bool(a < b)); }
                        Value::Int(b) => { self.push(Value::Bool(a < (b as f64))); }
                        _ => { error!(error); }
                    }}
                    Value::Int(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Bool((a as f64) < b)); }
                        Value::Int(b) => { self.push(Value::Bool(a < b)); }
                        _ => { error!(error); }
                    }}
                    _ => { error!(error); }
                }
            },
            ">=" => {
                match operand_a {
                    Value::Float(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Bool(a >= b)); }
                        Value::Int(b) => { self.push(Value::Bool(a >= (b as f64))); }
                        _ => { error!(error); }
                    }}
                    Value::Int(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Bool((a as f64) >= b)); }
                        Value::Int(b) => { self.push(Value::Bool(a >= b)); }
                        _ => { error!(error); }
                    }}
                    _ => { error!(error); }
                }
            }
            "<=" => {
                match operand_a {
                    Value::Float(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Bool(a <= b)); }
                        Value::Int(b) => { self.push(Value::Bool(a <= (b as f64))); }
                        _ => { error!(error); }
                    }}
                    Value::Int(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Bool((a as f64) <= b)); }
                        Value::Int(b) => { self.push(Value::Bool(a <= b)); }
                        _ => { error!(error); }
                    }}
                    _ => { error!(error); }
                }
            }
            "==" => {
                match operand_a {
                    Value::Float(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Bool(a == b)); }
                        Value::Int(b) => { self.push(Value::Bool(a == (b as f64))); }
                        _ => { self.push(Value::Bool(false)); }
                    }}
                    Value::Int(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Bool((a as f64) == b)); }
                        Value::Int(b) => { self.push(Value::Bool(a == b)); }
                        _ => { self.push(Value::Bool(false)); }
                    }}
                    Value::Null => { match operand_b {
                        Value::Null => { self.push(Value::Bool(true)); }
                        _ => { self.push(Value::Bool(false));  }
                    }}
                    Value::Fn(f1) => { match operand_b {
                        Value::Fn(f2) => { self.push(Value::Bool(f1 == f2)); }
                        _ => { self.push(Value::Bool(false)); }
                    }}
                    Value::Bool(a) => { match operand_b {
                        Value::Bool(b) => { self.push(Value::Bool(a == b)); }
                        _ => { self.push(Value::Bool(false)); }
                    }}
                    Value::Instance(a) => { match operand_b {
                        Value::Instance(b) => { self.push(Value::Bool(a == b)); }
                        _ => { self.push(Value::Bool(false)); }
                    }}
                    Value::Type(a) => { match operand_b {
                        Value::Type(b) => { self.push(Value::Bool(a == b))}
                        _ => { self.push(Value::Bool(false)); }
                    }}
                    Value::String(a) => { match operand_b {
                        Value::String(b) => { self.push(Value::Bool(*a == *b)) }
                        _ => { self.push(Value::Bool(false)); }
                    }}
                    Value::Native(a) => { match operand_b {
                        Value::Native(b) => { self.push(Value::Bool(a == b))}
                        _ => { self.push(Value::Bool(false)); }
                    }}
                    Value::Trait(a) => { match operand_b {
                        Value::Trait(b) => { self.push(Value::Bool(a == b))}
                        _ => { self.push(Value::Bool(false)); }
                    }}
                    _ => {
                        self.push(Value::Bool(false));
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
        let error = Error::new(
            address.clone(),
            format!("could not use '{}' for {:?} and {:?}", op, operand_a, operand_b),
            "check your code.".to_string()
        );
        // логика
        match op {
            "and" => {
                match operand_a {
                    Value::Bool(a) => {
                        match operand_b {
                            Value::Bool(b) => { self.push(Value::Bool(a && b)); }
                            _ => { error!(error); }
                        }
                    }
                    _ => { error!(error); }
                }
            }
            "or" => {
                match operand_a {
                    Value::Bool(a) => {
                        match operand_b {
                            Value::Bool(b) => { self.push(Value::Bool(a || b)); }
                            _ => { error!(error); }
                        }
                    }
                    _ => { error!(error); }
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
        let table = memory::alloc_value(Table::new());
        (*table).set_root(root);
        // условие
        self.run(*cond, table)?;
        let bool = self.pop(addr.clone())?;
        // проверка
        if let Value::Bool(b) = bool {
            if b {
                self.run(*body, table)?
            } else {
                if let Option::Some(else_if) = elif {
                    self.run(Chunk::of(*else_if), table)? // todo: chunk::of has high runtime cost!
                }
            }
        } else {
            error!(Error::new(
                addr.clone(),
                format!("condition provided not a bool: {:?}", bool),
                "condition should provide a bool.".to_string()
            ))
        }
        // успех
        Ok(())
    }

    // луп
    #[allow(unused_variables)]
    unsafe fn op_loop(&mut self, addr: Address, body: Box<Chunk>, root: *mut Table) -> Result<(), ControlFlow> {
        // таблица
        let table = memory::alloc_value(Table::new());
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
        // создаём значение функции и добавляем в gc
        let function_value = Value::Fn(function);
        self.gc_register(function_value, table);
        // дефайн функции
        if let Err(e) = (*table).define(addr.clone(), symbol.name.clone(), function_value) {
            error!(e);
        }
        // дефайн функции по full-name
        if symbol.full_name.is_some() {
            if let Err(e) = (*table).define(addr.clone(), symbol.full_name.unwrap(), function_value) {
                error!(e);
            }
        }
        // успех
        Ok(())
    }

    // дефайн типа
    unsafe fn op_define_type(&mut self, addr: Address, symbol: Symbol, body: Box<Chunk>,
                             constructor: Vec<String>, impls: Vec<String>) -> Result<(), ControlFlow> {
        // создаём тип
        let t = memory::alloc_value(
            Type::new(
                symbol.clone(),
                constructor,
                memory::alloc_value(*body),
                impls
            )
        );
        // дефайн типа
        if let Err(e) = (*self.types).define(addr.clone(), symbol.name.clone(), Value::Type(t)) {
            error!(e);
        }
        // дефайн по full-name
        if symbol.full_name.is_some() {
            if let Err(e) = (*self.types).define(addr.clone(), symbol.full_name.unwrap().clone(), Value::Type(t)){
                error!(e);
            }
        }
        // успех
        Ok(())
    }

    // дефайн юнита
    unsafe fn op_define_unit(&mut self, addr: Address, symbol: Symbol,
                             body: Box<Chunk>, table: *mut Table) -> Result<(), ControlFlow> {
        // создаём юнит
        let unit = memory::alloc_value(
            Unit::new(
                symbol.clone(),
                memory::alloc_value(Table::new())
            )
        );
        // добавляем в учет gc
        self.gc_register(Value::Unit(unit), table);
        // рут
        (*(*unit).fields).set_root(self.globals);
        // временный self
        (*(*unit).fields).fields.insert("self".to_string(), Value::Unit(unit));
        // исполняем тело
        self.run(*body, (*unit).fields)?;
        // удаляем временный self
        (*(*unit).fields).fields.remove(&"self".to_string());
        // бинды
        self.bind_functions((*unit).fields, memory::alloc_value(FnOwner::Unit(unit)));
        // дефайн юнита
        if let Err(e) = (*self.units).define(addr.clone(), symbol.name.clone(),
                                             Value::Unit(unit)) {
            error!(e);
        }
        // дефайн по full-name
        if symbol.full_name.is_some() {
            if let Err(e) = (*self.units).define(addr.clone(), symbol.full_name.unwrap().clone(),
                                                 Value::Unit(unit)) {
                error!(e);
            }
        }
        // успех
        Ok(())
    }

    // дефайн тейта
    unsafe fn op_define_trait(&mut self, addr: Address, symbol: Symbol,
                              functions: Vec<TraitFn>) -> Result<(), ControlFlow> {
        // создаём трейт
        let _trait = memory::alloc_value(
            Trait::new(
                symbol.clone(),
                functions
            )
        );
        // дефайн трейта
        if let Err(e) = (*self.traits).define(addr.clone(), symbol.name.clone(),
                                             Value::Trait(_trait)) {
            error!(e);
        }
        // дефайн по full-name
        if symbol.full_name.is_some() {
            if let Err(e) = (*self.traits).define(addr.clone(), symbol.full_name.unwrap().clone(),
                                                 Value::Trait(_trait)) {
                error!(e);
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
                error!(e);
            }
        }
        // если есть
        else {
            // получаем значение
            let previous = self.pop(addr.clone())?;
            // првоеряем
            match previous {
                Value::Instance(instance) => {
                    // исполняем значение
                    self.run(*value, table)?;
                    // получаем значение
                    let operand = self.pop(addr.clone())?;
                    // дефайним
                    if let Err(e) = (*(*instance).fields).define(addr.clone(), name, operand) {
                        error!(e);
                    }
                }
                Value::Unit(unit) => {
                    // исполняем значение
                    self.run(*value, table)?;
                    // получаем значение
                    let operand = self.pop(addr.clone())?;
                    // дефайним
                    if let Err(e) = (*(*unit).fields).define(addr.clone(), name, operand) {
                        error!(e);
                    }
                }
                _ => {
                    error!(Error::new(
                        addr.clone(),
                        format!("{:?} is not a container.", previous),
                        "you can define variable for unit or instance.".to_string()
                    ))
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
                error!(e);
            }
        }
        // если есть
        else {
            // получаем значение
            let previous = self.pop(addr.clone())?;
            // проверяем
            match previous {
                Value::Instance(instance) => {
                    // исполняем значение
                    self.run(*value, table)?;
                    // получаем значение
                    let operand = self.pop(addr.clone())?;
                    // устанавливаем значение
                    if let Err(e) = (*(*instance).fields).set_local(addr.clone(), name, operand) {
                        error!(e);
                    }
                }
                Value::Unit(unit) => {
                    // исполняем значение
                    self.run(*value, table)?;
                    // получаем значение
                    let operand = self.pop(addr.clone())?;
                    // устанавливаем значение
                    if let Err(e) = (*(*unit).fields).set_local(addr.clone(), name, operand) {
                        error!(e);
                    }
                }
                _ => {
                    error!(Error::new(
                        addr.clone(),
                        format!("{:?} is not a container.", previous),
                        "you can define variable for unit or instance.".to_string()
                    ))
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
            let lookup_result;
            if (*table).has(name.clone()) {
                lookup_result = (*table).lookup(addr.clone(), name);
            } else if (*self.types).has(name.clone()) {
                lookup_result = (*self.types).find(addr.clone(), name);
            } else {
                lookup_result = (*self.units).find(addr.clone(), name);
            }
            // проверяем на ошибку
            if let Err(e) = lookup_result {
                // ошибка
                error!(e)
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
            match previous {
                Value::Instance(instance) => {
                    // получаем значение
                    let lookup_result = (*(*instance).fields).find(addr.clone(), name);
                    // проверяем на ошибку
                    if let Err(e) = lookup_result {
                        // ошибка
                        error!(e)
                    }
                    else if let Ok(value) = lookup_result {
                        // пушим в стек
                        if !should_push { return Ok(()) }
                        self.push(value);
                    }
                }
                Value::Unit(unit) => {
                    // получаем значение
                    let lookup_result = (*(*unit).fields).find(addr.clone(), name);
                    // проверяем на ошибку
                    if let Err(e) = lookup_result {
                        // ошибка
                        error!(e)
                    }
                    else if let Ok(value) = lookup_result {
                        // пушим в стек
                        if !should_push { return Ok(()) }
                        self.push(value);
                    }
                }
                _ => {
                    error!(Error::new(
                        addr.clone(),
                        format!("{:?} is not a container.", previous),
                        "you can load variable from unit or instance.".to_string()
                    ))
                }
            }
        }
        // успех
        Ok(())
    }

    // вызов функции
    #[allow(unused_parens)]
    pub unsafe fn call(&mut self, addr: Address, name: String,
                              callable: Value, args: Box<Chunk>,
                              table: *mut Table, should_push: bool) -> Result<(), ControlFlow> {

        // подгрузка аргументов
        unsafe fn pass_arguments(vm: &mut VM, addr: Address, name: String, params_amount: usize,
                                 args: Box<Chunk>, params: Vec<String>, table: *mut Table,
                                 call_table: *mut Table) -> Result<(), ControlFlow> {
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
                    if let Err(e) = (*call_table).define(addr.clone(), param.clone(), operand) {
                        error!(e);
                    }
                }
                Ok(())
            } else {
                error!(Error::new(
                    addr.clone(),
                    format!("invalid args amount: {} to call: {}.", passed_amount, name),
                    format!("expected {} arguments.", params_amount)
                ));
                Ok(())
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
                error!(Error::new(
                    addr.clone(),
                    format!("invalid args amount: {} to call: {}.", passed_amount, name),
                    format!("expected {} arguments.", params_amount)
                ));
                Ok(())
            }
        }

        // проверка на функцию
        if let Value::Fn(function) = callable {
            // создаём таблицу под вызов.
            let call_table = memory::alloc_value(Table::new());
            // рут и self
            if !(*function).owner.is_null() {
                match (*(*function).owner) {
                    FnOwner::Unit(unit) => {
                        (*call_table).set_root((*unit).fields);
                        if let Err(e) = (*call_table).define(
                            addr.clone(), "self".to_string(), Value::Unit(unit)
                        ) {
                            error!(e);
                        }
                    },
                    FnOwner::Instance(instance) => {
                        (*call_table).set_root((*instance).fields);
                        if let Err(e) = (*call_table).define(
                            addr.clone(), "self".to_string(), Value::Instance(instance)
                        ) {
                            error!(e);
                        }
                    }
                }
            } else {
                (*call_table).set_root(self.globals)
            }
            // замыкание
            (*call_table).closure = (*function).closure;
            // загрузка аргументов
            pass_arguments(self, addr, name, (*function).params.len(), args,
                           (*function).params.clone(), table, call_table)?;
            // вызов
            match self.run((*(*function).body).clone(), call_table) {
                Err(e) => {
                    match e {
                        ControlFlow::Return(val) => {
                            if should_push {
                                self.push(val);
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
        else if let Value::Native(function) = callable {
            // создаём таблицу под вызов.
            let call_table = memory::alloc_value(Table::new());
            // рут и self
            if !(*function).owner.is_null() {
                match (*(*function).owner) {
                    FnOwner::Unit(unit) => {
                        (*call_table).set_root((*unit).fields);
                        if let Err(e) = (*call_table).define(
                            addr.clone(), "self".to_string(), Value::Unit(unit)
                        ) {
                            error!(e);
                        }
                    },
                    FnOwner::Instance(instance) => {
                        (*call_table).set_root((*instance).fields);
                        if let Err(e) = (*call_table).define(
                            addr.clone(), "self".to_string(), Value::Instance(instance)
                        ) {
                            error!(e);
                        }
                    }
                }
            } else {
                (*call_table).set_root(self.globals)
            }
            // загрузка аргументов
            load_arguments(self, addr.clone(), name.clone(), (*function).params_amount, args, table)?;
            // вызов
            let native = (*function).function;
            native(self, addr.clone(), should_push, call_table, (*function).owner)?;
            // успех
            Ok(())
        }
        else {
            error!(Error::new(
                addr.clone(),
                format!("{} is not a fn.", name),
                "you can call only fn-s.".to_string()
            ));
            Ok(())
        }
    }

    // загрузка значения переменной
    pub unsafe fn op_call(&mut self, addr: Address, name: String, has_previous: bool,
                                 should_push: bool, args: Box<Chunk>, table: *mut Table) -> Result<(), ControlFlow> {
        // если нет предыдущего
        if !has_previous {
            // получаем значение
            let lookup_result = (*table).lookup(addr.clone(), name.clone());
            // проверяем на ошибку
            if let Err(e) = lookup_result {
                // ошибка
                error!(e)
            }
            else if let Ok(value) = lookup_result {
                // вызываем
                self.call(addr.clone(), name.clone(), value, args.clone(), table, should_push)?;
            }
        }
        // если есть
        else {
            // получаем значение
            let previous = self.pop(addr.clone())?;
            // проверяем
            match previous {
                Value::Instance(instance) => {
                    // получаем значение
                    let lookup_result = (*(*instance).fields).find(addr.clone(), name.clone());
                    // проверяем на ошибку
                    if let Err(e) = lookup_result {
                        // ошибка
                        error!(e)
                    }
                    else if let Ok(value) = lookup_result {
                        // вызываем
                        self.call(addr.clone(), name.clone(), value, args.clone(), table, should_push)?;
                    }
                }
                Value::Unit(unit) => {
                    // получаем значение
                    let lookup_result = (*(*unit).fields).find(addr.clone(), name.clone());
                    // проверяем на ошибку
                    if let Err(e) = lookup_result {
                        // ошибка
                        error!(e)
                    }
                    else if let Ok(value) = lookup_result {
                        // вызываем
                        self.call(addr.clone(), name.clone(), value, args.clone(), table, should_push)?;
                    }
                }
                _ => {
                    error!(Error::new(
                        addr.clone(),
                        format!("couldn't call {} from {:?}.", name.clone(), previous),
                        "you can call fn from unit, instance or foreign.".to_string()
                    ))
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

    // проверка трейтов
    unsafe fn check_traits(&mut self, addr: Address, instance: *mut Instance) {
        // тип инстанса
        let instance_type = (*instance).t;
        // получение трейта
        unsafe fn get_trait(traits: *mut Table, addr: Address, trait_name: String) -> Option<*mut Trait> {
            // трейт
            let trait_result = (*traits).find(addr, trait_name);
            // проверяем результат
            if let Err(e) = trait_result {
                error!(e);
                None
            }
            else if let Ok(trait_value) = trait_result {
                match trait_value {
                    Value::Trait(_trait) => {
                        // перебираем функции
                        return Some(_trait)
                    }
                    _ => {
                        panic!("not a trait in traits table. report to developer.")
                    }
                }
            }
            else {
                return None
            }
        }
        // получение имплементации
        unsafe fn get_impl(table: *mut Table, addr: Address, impl_name: String) -> Option<*mut Function> {
            // трейт
            let fn_result = (*table).lookup(addr, impl_name);
            // проверяем результат
            if let Err(e) = fn_result {
                error!(e);
                None
            }
            else if let Ok(trait_value) = fn_result {
                return match trait_value {
                    Value::Fn(_fn) => {
                        // перебираем функции
                        Some(_fn)
                    }
                    _ => {
                        None
                    }
                }
            }
            else {
                return None
            }
        }
        // проверка
        for trait_name in (*instance_type).impls.clone() {
            // получаем трейт
            let _trait = get_trait(self.traits, addr.clone(), trait_name.clone()).unwrap();
            // проверяем
            for function in (*_trait).functions.clone() {
                // проверяем наличие имплементации
                if (*(*instance).fields).exists(function.name.clone()) {
                    // имплементация
                    let _impl = get_impl((*instance).fields, addr.clone(), function.name.clone());
                    // проверяем
                    if _impl.is_some() {
                        // имплементация
                        let implementation = _impl.unwrap();
                        // проверяем имплементацию
                        if (*implementation).params.len() != function.params_amount {
                            // ошибка
                            error!(Error::new(
                                addr.clone(),
                                format!(
                                    "type {} impls {}, but fn {} has wrong impl.",
                                    (*instance_type).name.name.clone(),
                                    trait_name, function.name.clone()
                                ),
                                format!(
                                    "expected args {}, got {}",
                                    function.params_amount,
                                    (*implementation).params.len()
                                )
                            ));
                        }
                    }
                    else {
                        // ошибка
                        error!(Error::new(
                            addr.clone(),
                            format!(
                                "type {} impls {}, but doesn't impl fn {}({})",
                                (*instance_type).name.name.clone(),
                                trait_name, function.name.clone(),
                                function.params_amount
                            ),
                            format!("implement fn {}", function.name.clone())
                        ));
                    }
                }
                else {
                    // проверяем есть ли дефолтная имплементация
                    if function.default.is_some() {
                        // если есть
                        if let Err(e) = (*(*instance).fields).define(
                            addr.clone(),
                            function.name.clone(),
                            Value::Fn(memory::alloc_value(
                                function.default.clone().unwrap(),
                            ))
                        ) {
                            error!(e);
                        }
                    }
                    // если нет
                    else {
                        // ошибка
                        error!(Error::new(
                            addr.clone(),
                            format!(
                                "type {} impls {}, but doesn't impl fn {}({})",
                                (*instance_type).name.name.clone(), // todo check
                                trait_name, function.name.clone(), // todo check
                                function.params_amount
                            ),
                            format!("implement fn {}", function.name.clone())
                        ));
                    }
                }
            }
        }
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
                        error!(e);
                    }
                }
                Ok(())
            } else {
                error!(Error::new(
                    addr.clone(),
                    format!("invalid args amount: {} to create instance of {}.", passed_amount, name),
                    format!("expected {} arguments.", params_amount)
                ));
                Ok(())
            }
        }
        // ищем тип
        let lookup_result = (*self.types).lookup(addr.clone(), name.clone());
        // проверяем, найден ли
        if let Ok(value) = lookup_result {
            // проверяем тип ли
            match value {
                Value::Type(t) => {
                    // создаём экземпляр
                    let instance = memory::alloc_value(Instance::new(
                        t,
                        memory::alloc_value(Table::new()),
                    ));
                    // добавляем в учет gc
                    self.gc_register(Value::Instance(instance), table);
                    // конструктор
                    pass_constructor(
                        self,
                        addr.clone(),
                        name,
                        (*t).constructor.len(),
                        args,
                        (*t).constructor.clone(),
                        (*instance).fields
                    )?;
                    // рут
                    (*(*instance).fields).set_root(self.globals);
                    // временный self
                    (*(*instance).fields).fields.insert("self".to_string(), Value::Instance(instance));
                    // исполняем тело
                    self.run((*(*t).body).clone(), (*instance).fields)?;
                    // удаляем временный self
                    (*(*instance).fields).fields.remove(&"self".to_string());
                    // проверка трейтов
                    self.check_traits(addr.clone(), instance);
                    // бинды
                    self.bind_functions((*instance).fields, memory::alloc_value(FnOwner::Instance(instance)));
                    // значение экземпляра
                    let instance_value = Value::Instance(
                        instance
                    );
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
            error!(lookup_result.unwrap_err());
            Ok(())
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
            if let Value::Fn(function) = value {
                // устанавливаем замыкание
                (*function).closure = table;
                // успех
                Ok(())
            }
            else {
                // ошибка
                error!(Error::new(
                    addr.clone(),
                    format!("could not make closure for: {}", name.clone()),
                    "not a function.".to_string()
                ));
                Ok(())
            }
        }
        else {
            error!(
                lookup_result.unwrap_err()
            );
            Ok(())
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

    // нативная функция
    unsafe fn op_native(&mut self, addr: Address, name: String) -> Result<(), ControlFlow> {
        // лукап
        let result = (*self.natives).find(addr, name);
        // если нашлась нативная функция
        if let Ok(value) = result {
            self.push(value);
        }
        // если нет
        if let Err(e) = result {
            error!(e);
        }
        // ок
        Ok(())
    }

    // запуск байткода
    #[allow(unused_variables)]
    pub unsafe fn run(&mut self, chunk: Chunk, table: *mut Table) -> Result<(), ControlFlow> {
        for op in chunk.opcodes() {
            match op {
                Opcode::Push { addr, value } => {
                    self.op_push(value, table)?;
                }
                Opcode::Pop { addr } => {
                    self.pop(addr.clone())?;
                }
                Opcode::Bin { addr, op } => {
                    self.op_binary(addr, op.as_str(), table)?;
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
                Opcode::DefineType { addr, name, full_name, body, constructor, impls } => {
                    self.op_define_type(addr, Symbol::new_option(name, full_name), body, constructor, impls)?
                }
                Opcode::DefineUnit { addr, name, full_name, body } => {
                    self.op_define_unit(addr, Symbol::new_option(name, full_name), body, table)?
                }
                Opcode::DefineTrait { addr, name, functions } => {
                    self.op_define_trait(addr, Symbol::by_name(name), functions)?
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
                Opcode::Native { addr, fn_name } => {
                    self.op_native(addr, fn_name)?;
                }
            }
        }
        Ok(())
    }
}

// имплементация для передачи между потоками
unsafe impl Send for VM {}
unsafe impl Sync for VM {}