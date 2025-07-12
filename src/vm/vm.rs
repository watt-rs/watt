// импорты
use scopeguard::defer;
use crate::error;
use crate::errors::errors::{Error};
use crate::lexer::address::Address;
use crate::vm::bytecode::{Chunk, Opcode, OpcodeValue};
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
    pub stack: Vec<Value>,
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
            stack: Vec::new(),
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
        self.stack.push(value);
    }

    // поп
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

    // очистка
    pub unsafe fn cleanup(&mut self) {
        // освобождаем все значения через gc
        (*self.gc).cleanup();
        // высвобождаем gc
        memory::free_value(self.gc);
        // высвобождаем типы
        (*self.types).free_fields();
        // высвобождаем трэйты
        (*self.traits).free_fields();
        // высвобождаем таблицы
        memory::free_value(self.traits);
        memory::free_value(self.types);
        memory::free_value(self.natives);
        memory::free_value(self.units);
        memory::free_value(self.globals);
    }

    // очистка мусора
    pub unsafe fn gc_invoke(&mut self, table: *mut Table) {
        // собираем мусор
        (*self.gc).collect_garbage(self, table);
    }

    // добавление в учет сборщика мусора
    pub unsafe fn gc_register(&mut self, value: Value, table: *mut Table) {
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

    // защита значения от очистки
    pub unsafe fn gc_guard(&mut self, value: Value) {
        // добавляем объект
        (*self.gc).push_guard(value);
    }

    // удаление защиты последнего значения от очистки
    pub unsafe fn gc_unguard(&mut self) {
        // добавляем объект
        (*self.gc).pop_guard();
    }

    // пуш в стек
    pub unsafe fn op_push(&mut self, value: OpcodeValue, table: *mut Table) -> Result<(), ControlFlow> {
        // проверяем значение
        match value {
            OpcodeValue::Int(int) => { self.push(Value::Int(int)); }
            OpcodeValue::Float(float) => { self.push(Value::Float(float)); }
            OpcodeValue::Bool(bool) => { self.push(Value::Bool(bool)); }
            OpcodeValue::String(string) => {
                // строка
                let new_string = Value::String(
                    memory::alloc_value(
                        string
                    )
                );
                // пушим
                self.push(new_string);
                // добавляем в gc
                self.gc_register(new_string, table);
            }
            OpcodeValue::Raw(raw) => {
                match raw {
                    Value::Instance(_) | Value::Fn(_) |
                    Value::Native(_) | Value::String(_) |
                    Value::Unit(_) | Value::List(_) |
                    Value::Any(_) => {
                        // пушим
                        self.push(raw);
                        // добавляем в gc
                        self.gc_register(raw, table);
                    }
                    _ => {
                        // пушим
                        self.push(raw);
                    }
                }
            }
        }
        // успех
        Ok(())
    }
    
    // бинарная операция
    unsafe fn op_binary(&mut self, address: Address, op: &str, table: *mut Table) -> Result<(), ControlFlow> {
        // два операнда
        let operand_a = self.pop(&address)?;
        let operand_b = self.pop(&address)?;
        // ошибки
        let invalid_op_error = || {
            error!(Error::own_text(
                address.clone(),
                format!("could not use '{}' with {:?} and {:?}", op, operand_a, operand_b),
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

        // бинарная операция
        match op {
            "+" => {
                match operand_a {
                    Value::Float(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float(a + b)); }
                        Value::Int(b) => { self.push(Value::Float(a + (b as f64))); }
                        Value::String(b) => {
                            let string = Value::String(
                                memory::alloc_value(format!("{}{}", a, *b))
                            );
                            self.push(string);
                            self.gc_register(string, table);
                        }
                        _ => { invalid_op_error(); }
                    }}
                    Value::Int(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float((a as f64) + b)); }
                        Value::Int(b) => { self.push(Value::Int(a + b)); }
                        Value::String(b) => {
                            let string = Value::String(
                                memory::alloc_value(format!("{}{}", a, *b))
                            );
                            self.push(string);
                            self.gc_register(string, table);
                        }
                        _ => { invalid_op_error(); }
                    }}
                    Value::String(a) => {
                        let string = Value::String(
                            memory::alloc_value(format!("{}{:?}", *a, operand_b))
                        );
                        self.push(string);
                        self.gc_register(string, table);
                    }
                    _ => {
                        if let Value::String(_) = operand_b {
                            let string = Value::String(
                                memory::alloc_value(format!("{:?}{:?}", operand_a, operand_b))
                            );
                            self.push(string);
                            self.gc_register(string, table);
                        }
                        else {
                            invalid_op_error();
                        }
                    }
                }
            }
            "-" => {
                match operand_a {
                    Value::Float(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float(a - b)); }
                        Value::Int(b) => { self.push(Value::Float(a - (b as f64))); }
                        _ => { invalid_op_error(); }
                    }}
                    Value::Int(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float((a as f64) - b)); }
                        Value::Int(b) => { self.push(Value::Int(a - b)); }
                        _ => { invalid_op_error(); }
                    }}
                    _ => { invalid_op_error(); }
                }
            }
            "*" => {
                match operand_a {
                    Value::Float(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float(a * b)); }
                        Value::Int(b) => { self.push(Value::Float(a * (b as f64))); }
                        _ => { invalid_op_error(); }
                    }}
                    Value::Int(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float((a as f64) * b)); }
                        Value::Int(b) => { self.push(Value::Int(a * b)); }
                        _ => { invalid_op_error(); }
                    }}
                    _ => { invalid_op_error(); }
                }
            }
            "/" => {
                match operand_a {
                    Value::Float(a) => { match operand_b {
                        Value::Float(b) => {
                            // проверка на деление на 0
                            if b == 0f64 { division_error(); }
                            // деление
                            self.push(Value::Float(a / b));
                        }
                        Value::Int(b) => {
                            // проверка на деление на 0
                            if b == 0 { division_error(); }
                            // деление
                            self.push(Value::Float(a / (b as f64)));
                        }
                        _ => { invalid_op_error(); }
                    }}
                    Value::Int(a) => { match operand_b {
                        Value::Float(b) => {
                            // проверка на деление на 0
                            if b == 0f64 { division_error(); }
                            // деление
                            self.push(Value::Float((a as f64) / b));
                        }
                        Value::Int(b) => {
                            // проверка на деление на 0
                            if b == 0 { division_error(); }
                            // деление
                            if a % b == 0 { self.push(Value::Int(a / b)); }
                            else { self.push(Value::Float(a as f64 / b as f64)) }
                        }
                        _ => { invalid_op_error(); }
                    }}
                    _ => { invalid_op_error(); }
                }
            }
            "%" => {
                match operand_a {
                    Value::Float(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float(a % b)); }
                        Value::Int(b) => { self.push(Value::Float(a % (b as f64))); }
                        _ => { invalid_op_error(); }
                    }}
                    Value::Int(a) => { match operand_b {
                        Value::Float(b) => { self.push(Value::Float((a as f64) % b)); }
                        Value::Int(b) => { self.push(Value::Int(a % b)); }
                        _ => { invalid_op_error(); }
                    }}
                    _ => { invalid_op_error(); }
                }
            }
            "&" => {
                match operand_a {
                    Value::Int(a) => { match operand_b {
                        Value::Int(b) => { self.push(Value::Int(a & b)); }
                        _ => { invalid_op_error(); }
                    }}
                    _ => { invalid_op_error(); }
                }
            }
            "|" => {
                match operand_a {
                    Value::Int(a) => { match operand_b {
                        Value::Int(b) => { self.push(Value::Int(a | b)); }
                        _ => { invalid_op_error(); }
                    }}
                    _ => { invalid_op_error(); }
                }
            }
            "^" => {
                match operand_a {
                    Value::Int(a) => { match operand_b {
                        Value::Int(b) => { self.push(Value::Int(a ^ b)); }
                        _ => { invalid_op_error(); }
                    }}
                    _ => { invalid_op_error(); }
                }
            }
            _ => { panic!("operator = {} is not found.", op)}
        }
        Ok(())
    }

    // негэйт
    unsafe fn op_negate(&mut self, address: &Address) -> Result<(), ControlFlow> {
        // операнд
        let operand = self.pop(&address)?;
        // негэйт
        match operand {
            Value::Float(a) => {
                self.push(Value::Float(-a));
            }
            Value::Int(a) => {
                self.push(Value::Int(-a));
            }
            _ => {
                // ошибка
                error!(Error::own_text(
                    address.clone(),
                    format!("could not use 'negate' for {:?}", operand),
                    "check your code."
                ));
            }
        }
        // успех
        Ok(())
    }

    // бэнг
    unsafe fn op_bang(&mut self, address: &Address) -> Result<(), ControlFlow> {
        // операнд
        let operand = self.pop(address)?;
        // бэнг
        match operand {
            Value::Bool(b) => {
                self.push(Value::Bool(!b));
            }
            _ => { 
                error!(Error::own_text(
                    address.clone(),
                    format!("could not use 'bang' for {:?}", operand),
                    "check your code."
                ));
            }
        }
        // успех
        Ok(())
    }

    // условие
    unsafe fn op_conditional(&mut self, address: &Address, op: &str) -> Result<(), ControlFlow> {
        // операнды
        let operand_a = self.pop(&address)?;
        let operand_b = self.pop(&address)?;
        let error = Error::own_text(
            address.clone(),
            format!("could not use '{}' for {:?} and {:?}", op, operand_a, operand_b),
            "check your code."
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
                    Value::String(a) => { match operand_b {
                        Value::String(b) => { self.push(Value::Bool(*a > *b)); }
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
                    Value::String(a) => { match operand_b {
                        Value::String(b) => { self.push(Value::Bool(*a < *b)); }
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
                    Value::String(a) => { match operand_b {
                        Value::String(b) => { self.push(Value::Bool(*a >= *b)); }
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
                    Value::String(a) => { match operand_b {
                        Value::String(b) => { self.push(Value::Bool(*a <= *b)); }
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
            "!=" => {
                // операнды
                self.push(operand_b);
                self.push(operand_a);
                // выполняем ==
                self.op_conditional(&address, "==")?;
                // инверсируем
                self.op_bang(&address)?;
            }
            _ => { panic!("operator {} is not found.", op)}
        }
        // успех
        Ok(())
    }

    // логика
    unsafe fn op_logical(&mut self, address: &Address, op: &str) -> Result<(), ControlFlow> {
        // операнды
        let operand_a = self.pop(&address)?;
        let operand_b = self.pop(&address)?;
        let error = Error::own_text(
            address.clone(),
            format!("could not use '{}' for {:?} and {:?}", op, operand_a, operand_b),
            "check your code."
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
    unsafe fn op_if(&mut self, addr: &Address, cond: Chunk, body: Chunk,
                    elif: Option<Chunk>, root: *mut Table) -> Result<(), ControlFlow> {
        // таблица
        let table = memory::alloc_value(Table::new());
        (*table).set_root(root);
        // высвобождение
        defer! {
            // высвобождение таблицы
            memory::free_value(table);
        }
        // условие
        self.run_owned(cond, table)?;
        let bool = self.pop(&addr)?;
        // проверка
        if let Value::Bool(b) = bool {
            if b {
                self.run_owned(body, table)?
            } else {
                if let Option::Some(else_if) = elif {
                    self.run_owned(else_if, table)?
                }
            }
        } else {
            error!(Error::own_text(
                addr.clone(),
                format!("condition provided not a bool: {:?}", bool),
                "condition should provide a bool."
            ))
        }
        // успех
        Ok(())
    }

    // луп
    #[allow(unused_variables)]
    unsafe fn op_loop(&mut self, addr: &Address, body: Chunk, root: *mut Table) -> Result<(), ControlFlow> {
        // таблица
        let table = memory::alloc_value(Table::new());
        (*table).set_root(root);
        // высвобождение
        defer! {
            // высвобождение таблицы
            memory::free_value(table);
        }
        // проверка
        loop {
            if let Err(e) = self.run(&body, table) {
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
    unsafe fn op_define_fn(&mut self, addr: &Address, symbol: Symbol, body: Chunk,
                        params: Vec<String>, make_closure: bool, table: *mut Table) -> Result<(), ControlFlow> {
        // создаём функцию
        let function = memory::alloc_value(
            Function::new(
                symbol.clone(),
                memory::alloc_value(body),
                params
            )
        );
        // если надо создать замыкание
        if make_closure {
            // создаём замыкание
            let closure = memory::alloc_value(Table::new());
            // копируем поля
            (*closure).fields = (*table).fields.clone();
            (*closure).closure = (*table).closure.clone();
            // устанавливаем замыкание
            (*function).closure = closure;
        }
        // создаём значение функции
        let function_value = Value::Fn(function);
        // защищаем значение
        self.gc_guard(function_value);
        // добавляем в gc
        self.gc_register(function_value, table);
        // дефайн функции
        if let Err(e) = (*table).define(&addr, &symbol.name, function_value) {
            error!(e);
        }
        // дефайн функции по full-name
        if symbol.full_name.is_some() {
            if let Err(e) = (*table).define(&addr, symbol.full_name.as_ref().unwrap(), function_value) {
                error!(e);
            }
        }
        // удаляем защиту
        self.gc_unguard();
        // успех
        Ok(())
    }

    // создание анонимной функции и пуш её в стек
    unsafe fn op_anonymous_fn(&mut self, body: &Chunk, params: &Vec<String>,
                              make_closure: bool, table: *mut Table) -> Result<(), ControlFlow> {
        // создаём функцию
        let function = memory::alloc_value(
            Function::new(
                Symbol::by_name("$lambda".to_string()),
                memory::alloc_value(body.clone()),
                params.clone()
            )
        );
        // если надо создать замыкание
        if make_closure {
            // создаём замыкание
            let closure = memory::alloc_value(Table::new());
            // копируем поля
            (*closure).fields = (*table).fields.clone();
            (*closure).closure = (*table).closure.clone();
            // устанавливаем замыкание
            (*function).closure = closure;
        }
        // создаём значение функции
        let function_value = Value::Fn(function);
        // пушим в стек
        self.push(function_value);
        // регистрируем в gc
        self.gc_register(function_value, table);
        // успех
        Ok(())
    }

    // бинды функций
    unsafe fn bind_functions(&mut self, table: *mut Table, owner: FnOwner) {
        // биндим
        for val in (*table).fields.values() {
            if let Value::Fn(function) = *val {
                (*function).owner = Some(owner.clone());
            }
        }
    }

    // дефайн типа
    unsafe fn op_define_type(&mut self, addr: &Address, symbol: Symbol, body: Chunk,
                             constructor: Vec<String>, impls: Vec<String>) -> Result<(), ControlFlow> {
        // создаём тип
        let t = memory::alloc_value(
            Type::new(
                symbol.clone(),
                constructor,
                memory::alloc_value(body),
                impls
            )
        );
        // дефайн типа
        if let Err(e) = (*self.types).define(&addr, &symbol.name, Value::Type(t)) {
            error!(e);
        }
        // дефайн по full-name
        if symbol.full_name.is_some() {
            if let Err(e) = (*self.types).define(&addr, symbol.full_name.as_ref().unwrap(), Value::Type(t)){
                error!(e);
            }
        }
        // успех
        Ok(())
    }

    // дефайн юнита
    unsafe fn op_define_unit(&mut self, addr: &Address, symbol: Symbol,
                             body: Chunk, table: *mut Table) -> Result<(), ControlFlow> {
        // создаём юнит
        let unit = memory::alloc_value(
            Unit::new(
                symbol.clone(),
                memory::alloc_value(Table::new())
            )
        );
        // значение юнита
        let unit_value = Value::Unit(unit);
        // защищаем значение
        self.gc_guard(unit_value);
        // добавляем в учет gc
        self.gc_register(unit_value, table);
        // рут
        (*(*unit).fields).set_root(self.globals);
        // временный parent
        (*(*unit).fields).parent = table;
        // временный self
        (*(*unit).fields).fields.insert("self".to_string(), Value::Unit(unit));
        // исполняем тело
        self.run_owned(body, (*unit).fields)?;
        // удаляем временный self
        (*(*unit).fields).fields.remove("self");
        // бинды
        self.bind_functions((*unit).fields, FnOwner::Unit(unit));
        // init функция
        let init_fn = "init";
        if (*(*unit).fields).exists(&init_fn) {
            // пушим юнит
            self.push(unit_value);
            // вызываем
            self.op_call(addr, init_fn, true, false, Chunk::new(vec![]), table)?
        }
        // дефайн юнита
        if let Err(e) = (*self.units).define(addr, &symbol.name, unit_value) {
            error!(e);
        }
        // дефайн по full-name
        if symbol.full_name.is_some() {
            if let Err(e) = (*self.units).define(addr, symbol.full_name.as_ref().unwrap(), unit_value) {
                error!(e);
            }
        }
        // удаляем временный parent
        (*(*unit).fields).parent = std::ptr::null_mut();
        // удаляем защиту
        self.gc_unguard();
        // успех
        Ok(())
    }

    // дефайн тейта
    unsafe fn op_define_trait(&mut self, addr: &Address, symbol: Symbol, functions: Vec<TraitFn>)
    -> Result<(), ControlFlow> {
        // создаём трейт
        let _trait = memory::alloc_value(
            Trait::new(
                symbol.clone(),
                functions
            )
        );
        // дефайн трейта
        if let Err(e) = (*self.traits).define(&addr, &symbol.name, Value::Trait(_trait)) {
            error!(e);
        }
        // дефайн по full-name
        if symbol.full_name.is_some() {
            if let Err(e) = (*self.traits).define(&addr, symbol.full_name.as_ref().unwrap(), Value::Trait(_trait)) {
                error!(e);
            }
        }
        // успех
        Ok(())
    }

    // дефайн
    unsafe fn op_define(&mut self, addr: &Address, name: &str, has_previous: bool,
                        value: Chunk, table: *mut Table) -> Result<(), ControlFlow> {
        // если нет предыдущего
        if !has_previous {
            // исполняем значение
            self.run_owned(value, table)?;
            // получаем значение
            let operand = self.pop(&addr)?;
            // дефайним
            if let Err(e) = (*table).define(&addr, &name, operand) {
                error!(e);
            }
        }
        // если есть
        else {
            // получаем значение
            let previous = self.pop(&addr)?;
            // првоеряем
            match previous {
                Value::Instance(instance) => {
                    // исполняем значение
                    self.run_owned(value, table)?;
                    // получаем значение
                    let operand = self.pop(&addr)?;
                    // дефайним
                    if let Err(e) = (*(*instance).fields).define(&addr, &name, operand) {
                        error!(e);
                    }
                }
                Value::Unit(unit) => {
                    // исполняем значение
                    self.run_owned(value, table)?;
                    // получаем значение
                    let operand = self.pop(&addr)?;
                    // дефайним
                    if let Err(e) = (*(*unit).fields).define(&addr, &name, operand) {
                        error!(e);
                    }
                }
                _ => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("{:?} is not a container.", previous),
                        "you can define variable for unit or instance."
                    ))
                }
            }
        }
        // успех
        Ok(())
    }

    // установка значения переменной
    unsafe fn op_set(&mut self, addr: &Address, name: &str, has_previous: bool,
                        value: Chunk, table: *mut Table) -> Result<(), ControlFlow> {
        // если нет предыдущего
        if !has_previous {
            // исполняем значение
            self.run_owned(value, table)?;
            // получаем значение
            let operand = self.pop(&addr)?;
            // дефайним
            if let Err(e) = (*table).set(addr, name, operand) {
                error!(e);
            }
        }
        // если есть
        else {
            // получаем значение
            let previous = self.pop(&addr)?;
            // проверяем
            match previous {
                Value::Instance(instance) => {
                    // исполняем значение
                    self.run_owned(value, table)?;
                    // получаем значение
                    let operand = self.pop(&addr)?;
                    // устанавливаем значение
                    if let Err(e) = (*(*instance).fields).set_local(&addr, name, operand) {
                        error!(e);
                    }
                }
                Value::Unit(unit) => {
                    // исполняем значение
                    self.run_owned(value, table)?;
                    // получаем значение
                    let operand = self.pop(&addr)?;
                    // устанавливаем значение
                    if let Err(e) = (*(*unit).fields).set_local(&addr, name, operand) {
                        error!(e);
                    }
                }
                _ => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("{:?} is not a container.", previous),
                        "you can define variable for unit or instance."
                    ))
                }
            }
        }
        // успех
        Ok(())
    }

    // загрузка значения переменной
    unsafe fn op_load(&mut self, addr: &Address, name: &str, has_previous: bool,
                      should_push: bool, table: *mut Table) -> Result<(), ControlFlow> {
        // если нет предыдущего
        if !has_previous {
            // получаем значение
            let lookup_result;
            if (*table).has(name) {
                lookup_result = (*table).lookup(addr, &name);
            } else if (*self.types).has(name) {
                lookup_result = (*self.types).find(addr, &name);
            } else {
                lookup_result = (*self.units).find(addr, &name);
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
            let previous = self.pop(&addr)?;
            // проверяем
            match previous {
                Value::Instance(instance) => {
                    // получаем значение
                    let lookup_result = (*(*instance).fields).find(addr, &name);
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
                    let lookup_result = (*(*unit).fields).find(addr, &name);
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
                    error!(Error::own_text(
                        addr.clone(),
                        format!("{:?} is not a container.", previous),
                        "you can load variable from unit or instance."
                    ))
                }
            }
        }
        // успех
        Ok(())
    }

    // вызов функции
    #[allow(unused_parens)]
    pub unsafe fn call(&mut self, addr: &Address, name: &str,
                              callable: Value, args: Chunk,
                              table: *mut Table, should_push: bool) -> Result<(), ControlFlow> {

        // подгрузка аргументов
        unsafe fn pass_arguments(vm: &mut VM, addr: &Address, name: &str, params_amount: usize,
                                 args: Chunk, params: Vec<String>, table: *mut Table,
                                 call_table: *mut Table) -> Result<(), ControlFlow> {
            // фиксируем размер стека
            let prev_size = vm.stack.len();
            // загрузка аргументов
            vm.run_owned(args, table)?;
            // фиксируем новый размер стека
            let new_size = vm.stack.len();
            // количество переданных аргументов
            let passed_amount = new_size-prev_size;
            // проверяем количество аргументов и параметров
            // если совпало
            if passed_amount == params_amount {
                // проходимся по реверсированным параметрам
                for param in params.iter().rev() {
                    // получаем аргумент из стека
                    let operand = vm.pop(&addr)?;
                    // устанавливаем в таблице
                    if let Err(e) = (*call_table).define(addr, &param, operand) {
                        error!(e);
                    }
                }
                Ok(())
            }
            // если не совпало
            else {
                error!(Error::own(
                    addr.clone(),
                    format!(
                        "invalid args amount: {} to call: {}. stack: {:?}",
                        passed_amount, name, vm.stack
                    ),
                    format!("expected {} arguments.", params_amount)
                ));
            }
        }

        // только загрузка аргументов
        unsafe fn load_arguments(vm: &mut VM, addr: &Address, name: &str, params_amount: usize,
                                 args: Chunk, table: *mut Table) -> Result<(), ControlFlow> {
            // фиксируем размер стека
            let prev_size = vm.stack.len();
            // загрузка аргументов
            vm.run_owned(args, table)?;
            // фиксируем новый размер стека
            let new_size = vm.stack.len();
            // количество переданных аргументов
            let passed_amount = new_size-prev_size;
            // проверяем
            if passed_amount == params_amount {
                Ok(())
            } else {
                error!(Error::own(
                    addr.clone(),
                    format!(
                        "invalid args amount: {} to call: {}. stack: {:?}",
                        passed_amount, name, vm.stack
                    ),
                    format!("expected {} arguments.", params_amount)
                ));
            }
        }

        // проверка на функцию
        if let Value::Fn(function) = callable {
            // создаём таблицу под вызов.
            let call_table = memory::alloc_value(Table::new());
            // parent таблица
            (*call_table).parent = table;
            // замыкание
            (*call_table).closure = (*function).closure;
            // высвобождение
            defer! {
                // высвобождение таблицы
                memory::free_value(call_table);
            }
            // рут и self
            if (*function).owner.is_some() {
                match (*function).owner.clone().unwrap() {
                    FnOwner::Unit(unit) => {
                        (*call_table).set_root((*unit).fields);
                        if let Err(e) = (*call_table).define(
                            &addr, "self", Value::Unit(unit)
                        ) {
                            error!(e);
                        }
                    },
                    FnOwner::Instance(instance) => {
                        (*call_table).set_root((*instance).fields);
                        if let Err(e) = (*call_table).define(
                            &addr, "self", Value::Instance(instance)
                        ) {
                            error!(e);
                        }
                    }
                }
            } else {
                (*call_table).set_root(self.globals)
            }
            // загрузка аргументов
            pass_arguments(self, addr, name, (*function).params.len(), args,
                           (*function).params.clone(), table, call_table)?;
            
            let body = &*(*function).body;

            // вызов
            match self.run(body, call_table) {
                // если поймали control flow
                Err(e) => {
                    return match e {
                        // если поймали return
                        ControlFlow::Return(val) => {
                            // пушим
                            if should_push {
                                self.push(val);
                            }
                            // успех
                            Ok(())
                        },
                        // если другая ошибка
                        _ => {
                            // пробрасываем
                            Err(e)
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
            // parent таблица
            (*call_table).parent = table;
            // высвобождение
            defer! {
                // высвобождение таблицы
                memory::free_value(call_table);
            }
            // рут
            (*call_table).set_root(self.globals);
            // загрузка аргументов
            load_arguments(self, &addr, &name, (*function).params_amount, args, table)?;
            // вызов
            let native = (*function).function;
            native(self, addr.clone(), should_push, call_table)?;
            // успех
            Ok(())
        }
        else {
            error!(Error::own_text(
                addr.clone(),
                format!("{name} is not a fn."),
                "you can call only fn-s."
            ));
        }
    }

    // загрузка значения переменной
    pub unsafe fn op_call(&mut self, addr: &Address, name: &str, has_previous: bool,
                                 should_push: bool, args: Chunk, table: *mut Table) -> Result<(), ControlFlow> {
        // если нет предыдущего
        if !has_previous {
            // получаем значение
            let lookup_result = (*table).lookup(&addr, &name);
            // проверяем на ошибку
            if let Err(e) = lookup_result {
                // ошибка
                error!(e)
            }
            else if let Ok(value) = lookup_result {
                // вызываем
                self.call(addr, &name, value, args, table, should_push)?;
            }
        }
        // если есть
        else {
            // получаем значение
            let previous = self.pop(&addr)?;
            // проверяем
            match previous {
                Value::Instance(instance) => {
                    // получаем значение
                    let lookup_result = (*(*instance).fields).find(&addr, &name);
                    // проверяем на ошибку
                    if let Err(e) = lookup_result {
                        // ошибка
                        error!(e)
                    }
                    else if let Ok(value) = lookup_result {
                        // вызываем
                        self.call(addr, &name, value, args, table, should_push)?;
                    }
                }
                Value::Unit(unit) => {
                    // получаем значение
                    let lookup_result = (*(*unit).fields).find(&addr, &name);
                    // проверяем на ошибку
                    if let Err(e) = lookup_result {
                        // ошибка
                        error!(e)
                    }
                    else if let Ok(value) = lookup_result {
                        // вызываем
                        self.call(addr, &name, value, args, table, should_push)?;
                    }
                }
                _ => {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("couldn't call {} from {:?}.", name, previous),
                        "you can call fn from unit, instance or foreign."
                    ))
                }
            }
        }
        // успех
        Ok(())
    }

    // дублирование значения в стеке
    unsafe fn op_duplicate(&mut self, addr: &Address) -> Result<(), ControlFlow> {
        // операнд
        let operand = self.pop(&addr)?;
        // пушим
        self.push(operand);
        self.push(operand);
        // успех
        Ok(())
    }

    // проверка трейтов
    unsafe fn check_traits(&mut self, addr: &Address, instance: *mut Instance, table: *mut Table) {
        // тип инстанса
        let instance_type = (*instance).t;
        // получение трейта
        unsafe fn get_trait(traits: *mut Table, addr: &Address, trait_name: &str) -> Option<*mut Trait> {
            // трейт
            let trait_result = (*traits).find(addr, trait_name);
            // проверяем результат
            if let Err(e) = trait_result {
                error!(e);
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
        unsafe fn get_impl(table: *mut Table, addr: &Address, impl_name: &str) -> Option<*mut Function> {
            // трейт
            let fn_result = (*table).lookup(addr, impl_name);
            // проверяем результат
            if let Err(e) = fn_result {
                error!(e);
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
        for trait_name in &(*instance_type).impls {
            // получаем трейт
            let _trait = get_trait(self.traits, &addr, trait_name).unwrap();
            // проверяем
            for function in &(*_trait).functions {
                // проверяем наличие имплементации
                if (*(*instance).fields).exists(&function.name) {
                    // имплементация
                    let _impl = get_impl((*instance).fields, addr, &function.name);
                    // проверяем
                    if _impl.is_some() {
                        // имплементация
                        let implementation = _impl.unwrap();
                        // проверяем имплементацию
                        if (*implementation).params.len() != function.params_amount {
                            // ошибка
                            error!(Error::own(
                                addr.clone(),
                                format!(
                                    "type {} impls {}, but fn {} has wrong impl.",
                                    (*instance_type).name.name,
                                    trait_name, function.name
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
                        error!(Error::own(
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
                        // реализация
                        let default_impl = function.default.as_ref().unwrap();
                        // функция
                        let default_fn = Value::Fn(memory::alloc_value(
                            Function::new(
                                Symbol::by_name(function.name.clone()),
                                memory::alloc_value(default_impl.chunk.clone()),
                                default_impl.params.clone(),
                            ),
                        ));
                        // защита gc
                        self.gc_guard(default_fn);
                        // добавляем в gc
                        self.gc_register(default_fn, table);
                        // удаляем защиту gc
                        self.gc_unguard();
                        // если есть
                        if let Err(e) = (*(*instance).fields).define(
                            &addr,
                            &function.name,
                            default_fn
                        ) {
                            error!(e);
                        }
                    }
                    // если нет
                    else {
                        // ошибка
                        error!(Error::own(
                            addr.clone(),
                            format!(
                                "type {} impls {}, but doesn't impl fn {}({})",
                                (*instance_type).name.name, // todo check
                                trait_name, function.name, // todo check
                                function.params_amount
                            ),
                            format!("implement fn {}", function.name)
                        ));
                    }
                }
            }
        }
    }

    // созедание экземпляра типа
    unsafe fn op_instance(&mut self, addr: &Address, name: &str,
                          args: Chunk, should_push: bool, table: *mut Table) -> Result<(), ControlFlow> {

        // подгрузка конструктора
        unsafe fn pass_constructor(vm: &mut VM, addr: &Address, name: &str, params_amount: usize,
                                 args: Chunk, params: Vec<String>, table: *mut Table,
                                   fields_table: *mut Table) -> Result<(), ControlFlow> {
            // фиксируем размер стека
            let prev_size = vm.stack.len();
            // загрузка аргументов
            vm.run_owned(args, table)?;
            // фиксируем новый размер стека
            let new_size = vm.stack.len();
            // количество переданных аргументов
            let passed_amount = new_size-prev_size;
            // проверяем
            if passed_amount == params_amount {
                // проходимся по реверсированным параметрам
                for param in params.iter().rev() {
                    // получаем аргумент из стека
                    let operand = vm.pop(&addr)?;
                    // устанавливаем в таблице
                    if let Err(e) = (*fields_table).define(&addr, &param, operand) {
                        error!(e);
                    }
                }
                Ok(())
            } else {
                error!(Error::own(
                    addr.clone(),
                    format!("invalid args amount: {} to create instance of {}.", passed_amount, name),
                    format!("expected {} arguments.", params_amount)
                ));
            }
        }
        // ищем тип
        let lookup_result = (*self.types).lookup(addr, name);
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
                    // значение экземпляра
                    let instance_value = Value::Instance(instance);
                    // добавляем в защиту gc
                    self.gc_guard(instance_value);
                    // добавляем в учет gc
                    self.gc_register(Value::Instance(instance), table);
                    // конструктор
                    pass_constructor(
                        self,
                        &addr,
                        name,
                        (*t).constructor.len(),
                        args,
                        (*t).constructor.clone(),
                        table,
                        (*instance).fields
                    )?;
                    // рут
                    (*(*instance).fields).set_root(self.globals);
                    // временный parent
                    (*(*instance).fields).parent = table;
                    // временный self
                    (*(*instance).fields).fields.insert("self".to_string(), Value::Instance(instance));
                    // исполняем тело
                    self.run(&*(*t).body, (*instance).fields)?;
                    // удаляем временный self
                    (*(*instance).fields).fields.remove("self");
                    // проверка трейтов
                    self.check_traits(addr, instance, table);
                    // бинды
                    self.bind_functions((*instance).fields, FnOwner::Instance(instance));
                    // init функция
                    let init_fn = "init";
                    if (*(*instance).fields).exists(init_fn) {
                        // пушим инстанс
                        self.push(instance_value);
                        // вызываем
                        self.op_call(addr, &init_fn, true, false, Chunk::new(vec![]), table)?
                    }
                    // пушим
                    if should_push {
                        self.push(instance_value);
                    }
                    // удаляем временный parent
                    (*(*instance).fields).parent = std::ptr::null_mut();
                    // удаляем защиту gc
                    self.gc_unguard();
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
        }
    }

    // окончание цикла
    #[allow(unused_variables)]
    unsafe fn op_endloop(&mut self, addr: &Address, current_iteration: bool) -> Result<(), ControlFlow> {
        if current_iteration {
            Err(ControlFlow::Continue)
        } else {
            Err(ControlFlow::Break)
        }
    }

    // возврат значения из функции
    unsafe fn op_return(&mut self, addr: &Address, value: Chunk, table: *mut Table) -> Result<(), ControlFlow> {
        // выполняем
        self.run_owned(value, table)?;
        let value = self.pop(addr)?;
        // возвращаем
        Err(ControlFlow::Return(value))
    }

    // нативная функция
    unsafe fn op_native(&mut self, addr: &Address, name: &str) -> Result<(), ControlFlow> {
        // лукап
        let result = (*self.natives).find(addr, &name);
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

    // "пробрасывание" ошибок
    unsafe fn op_error_propagation(&mut self, addr: &Address, value: Chunk, table: *mut Table) -> Result<(), ControlFlow> {
        // выполняем
        self.run_owned(value, table)?;
        // значение
        let value = self.pop(addr)?;
        // вызов is_ok
        unsafe fn call_is_ok(vm: &mut VM, addr: &Address, instance: *mut Instance) -> Result<bool, ControlFlow> {
            // пробуем получить is_ok
            let lookup_result = (*(*instance).fields).find(&addr, "is_ok");
            // если успешно
            if let Ok(callable) = lookup_result {
                // проверяем, функция ли
                if let Value::Fn(function) = callable {
                    // проверяем количество аргументов
                    if (*function).params.len() != 0 {
                        error!(Error::own_text(
                            addr.clone(),
                            format!("is_ok takes {} params", (*function).params.len()),
                            "is_ok should take 0 params."
                        ));
                    }
                }
                // если нет
                else {
                    error!(Error::new(
                            addr.clone(),
                            "is_ok is not a fn.",
                            "is_ok should be fn."
                        ));
                }
                // вызываем
                vm.call(
                    &addr, "is_ok", callable,
                    Chunk::new(vec![]),
                    memory::alloc_value(Table::new()),
                    true
                )?;
                // получаем значение
                let is_ok = vm.pop(addr)?;
                // проверяем, бул ли
                return if let Value::Bool(boolean) = is_ok {
                    Ok(boolean)
                } else {
                    error!(Error::own(
                        addr.clone(),
                        "is_ok should return a bool.".to_string(),
                        format!("it returned: {is_ok:?}")
                    ));
                }
            }
            // если ошибка
            else if let Err(e) = lookup_result {
                error!(e);
            }
            // dead code
            Ok(false)
        }
        // вызов unwrap
        unsafe fn call_unwrap(vm: &mut VM, addr: &Address, instance: *mut Instance) -> Result<(), ControlFlow> {
            // пробуем получить is_ok
            let lookup_result = (*(*instance).fields).find(addr, "unwrap");
            // если успешно
            match lookup_result {
                Ok(callable) => {
                    // проверяем, функция ли
                    if let Value::Fn(function) = callable {
                        // проверяем количество аргументов
                        if (*function).params.len() != 0 {
                            error!(Error::own_text(
                                addr.clone(),
                                format!("unwrap takes {} params", (*function).params.len()),
                                "unwrap should take 0 params."
                            ));
                        }
                    }
                    // если нет
                    else {
                        error!(Error::new(
                            addr.clone(),
                            "unwrap is not a fn.",
                            "unwrap should be fn."
                        ));
                    }
                    // вызываем
                    vm.call(
                        &addr, "unwrap", callable,
                        Chunk::new(vec![]),
                        memory::alloc_value(Table::new()),
                        true
                    )?;
                    // успех
                    Ok(())
                },
                Err(e) => {
                    // ошибка
                    error!(e);
                }
            }
        }
        // проверяем тип значения
        if let Value::Instance(instance) = value {
            // вызов is_ok
            let is_ok = call_is_ok(
                self,
                &addr,
                instance,
            )?;
            // проверяем is_ok
            // если есть ошибка
            if !is_ok {
                // возвращаем обратно
                return Err(
                    ControlFlow::Return(value)
                );
            }
            // если нет ошибки
            else {
                // вызываем unwrap
                call_unwrap(
                    self,
                    addr,
                    instance
                )?;
            }
        }
        // если неверный тип значения - ошибка
        else {
            error!(Error::own_text(
                addr.clone(),
                format!("could not use error propagation with {:?}.", value),
                "requires instance of type that impls .is_ok() and .unwrap() fn-s."
            ))
        }
        // успех
        Ok(())
    }

    // проверка имплементации трейта
    unsafe fn op_impls(&mut self, addr: &Address, value: Chunk,
                       trait_name: &str, table: *mut Table) -> Result<(), ControlFlow> {
        // выполняем
        self.run_owned(value, table)?;
        // значение
        let value = self.pop(&addr)?;
        // проверка, экземпляр ли класс значение
        if let Value::Instance(instance) = value {
            // ищем трейт
            let lookup_result = (*self.traits).lookup(&addr, &trait_name);
            // если нашли
            if let Ok(trait_value) = lookup_result {
                // проверяем, трейт ли
                match trait_value {
                    // если трейт
                    Value::Trait(_trait) => {
                        // список имплементаций
                        let impls = &(*(*instance).t).impls;
                        // имена трейта
                        let name = &(*_trait).name.name;
                        let full_name_option = &(*_trait).name.full_name;
                        // если есть полное имя
                        if let Some(full_name) = full_name_option {
                            // пушим бул, есть ли трейт в имплементациях
                            self.push(Value::Bool(
                                impls.contains(&name) || impls.contains(&full_name),
                            ));
                        }
                        // если нет
                        else {
                            // пушим бул, есть ли трейт в имплементациях
                            self.push(Value::Bool(
                                impls.contains(&name),
                            ));
                        }
                    }
                    // если нет
                    _ => {
                        panic!("not a trait in traits table. report to developer.")
                    }
                }
            }
            // если трейта не существует
            else if let Err(e) = lookup_result {
                error!(e);
            }
        }
        else {
            error!(Error::own_text(
                addr.clone(),
                format!("could not use impls with {:?}.", value),
                "impls op requires instance."
            ))
        }
        // успех
        Ok(())
    }

    // удаление локальной переменной
    #[allow(unused_variables)]
    unsafe fn op_delete_local(&self, addr: &Address, name: &String, table: *mut Table) {
        (*table).fields.remove(name);
    }

    pub unsafe fn run_owned(&mut self, chunk: Chunk, table: *mut Table) -> Result<(), ControlFlow> { 
        for op in chunk.into_opcodes() {
            match op {
                // chkd
                Opcode::Push { value, .. } => {
                    self.op_push(value, table)?;
                }
                // chkd
                Opcode::Pop { addr } => {
                    self.pop(&addr)?;
                }
                // chkd
                Opcode::Bin { addr, op } => {
                    self.op_binary(addr, &op, table)?;
                }
                // chkd
                Opcode::Neg { addr } => {
                    self.op_negate(&addr)?;
                }
                // chkd
                Opcode::Bang { addr } => {
                    self.op_bang(&addr)?;
                }
                // chkd
                Opcode::Cond { addr, op } => {
                    self.op_conditional(&addr, &op)?;
                }
                // chkd
                Opcode::Logic { addr, op } => {
                    self.op_logical(&addr, &op)?
                }
                // chkd
                Opcode::If { addr, cond, body, elif } => {
                    self.op_if(&addr, cond, body, elif, table)?;
                }
                // later chk
                Opcode::Loop { addr, body } => {
                    self.op_loop(&addr, body, table)?;
                }
                // chkd
                Opcode::DefineFn {
                    addr,
                    name,
                    full_name,
                    body,
                    params,
                    make_closure
                } => {
                    self.op_define_fn(
                        &addr,
                        Symbol::new_option(name, full_name),
                        body,
                        params,
                        make_closure,
                        table
                    )?;
                }
                Opcode::AnonymousFn {
                    addr,
                    body,
                    params,
                    make_closure
                } => {
                    self.op_anonymous_fn(
                        &body,
                        &params,
                        make_closure,
                        table
                    )?;
                }
                // chkd
                Opcode::DefineType {
                    addr,
                    name,
                    full_name,
                    body,
                    constructor,
                    impls
                } => {
                    self.op_define_type(
                        &addr,
                        Symbol::new_option(name, full_name),
                        body,
                        constructor,
                        impls
                    )?;
                }
                // chkd
                Opcode::DefineUnit { addr, name, full_name, body } => {
                    self.op_define_unit(
                        &addr, 
                        Symbol::new_option(name, full_name),
                        body,
                        table
                    )?
                }
                // chkd
                Opcode::DefineTrait {
                    addr,
                    name,
                    full_name,
                    functions
                } => {
                    self.op_define_trait(
                        &addr,
                        Symbol::new_option(name, full_name),
                        functions
                    )?
                }
                // chkd
                Opcode::Define { addr, name, value, has_previous} => {
                    self.op_define(&addr, &name, has_previous, value, table)?;
                }
                // chkd
                Opcode::Set { addr, name, value, has_previous } => {
                    self.op_set(&addr, &name, has_previous, value, table)?;
                }
                // chkd
                Opcode::Load { addr, name, has_previous, should_push } => {
                    self.op_load(&addr, &name, has_previous, should_push, table)?;
                }
                Opcode::Call {
                    addr,
                    name,
                    has_previous,
                    should_push,
                    args
                } => {
                    self.op_call(&addr, &name, has_previous, should_push, args, table)?
                }
                Opcode::Duplicate { addr } => {
                    self.op_duplicate(&addr)?;
                }
                Opcode::Instance { addr, name, args, should_push } => {
                    self.op_instance(&addr, &name, args, should_push, table)?;
                }
                Opcode::EndLoop { addr, current_iteration } => {
                    self.op_endloop(&addr, current_iteration)?;
                }
                Opcode::Ret { addr, value } => {
                    self.op_return(&addr, value, table)?;
                }
                Opcode::Native { addr, fn_name } => {
                    self.op_native(&addr, &fn_name)?;
                }
                Opcode::ErrorPropagation { addr, value } => {
                    self.op_error_propagation(&addr, value, table)?;
                }
                Opcode::Impls { addr, value, trait_name } => {
                    self.op_impls(&addr, value, &trait_name, table)?;
                }
                Opcode::DeleteLocal { addr, name } => {
                    self.op_delete_local(&addr, &name, table)
                }
                _ => todo!("{:?}", op)
            }
        }

        Ok(())
    }

    // запуск байткода
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
                    self.op_binary(addr.clone(), &op, table)?;
                }
                Opcode::Neg { addr } => {
                    self.op_negate(addr)?;
                }
                Opcode::Bang { addr } => {
                    self.op_bang(&addr)?;
                }
                Opcode::Cond { addr, op } => {
                    self.op_conditional(&addr, &op)?;
                }
                Opcode::Logic { addr, op } => {
                    self.op_logical(addr, &op)?
                }
                Opcode::If { addr, cond, body, elif } => {
                    self.op_if(addr, cond.clone(), body.clone(), elif.clone(), table)?;
                }
                Opcode::Loop { addr, body } => {
                    self.op_loop(addr, body.clone(), table)?;
                }
                Opcode::DefineFn {
                    addr,
                    name,
                    full_name,
                    body,
                    params,
                    make_closure
                } => {
                    self.op_define_fn(
                        addr,
                        Symbol::new_option(name.clone(), full_name.clone()),
                        body.clone(),
                        params.clone(),
                        *make_closure,
                        table
                    )?;
                }
                Opcode::AnonymousFn {
                    addr,
                    body,
                    params,
                    make_closure
                } => {
                    self.op_anonymous_fn(
                        body,
                        params,
                        *make_closure,
                        table
                    )?;
                }
                Opcode::DefineType {
                    addr,
                    name,
                    full_name,
                    body,
                    constructor,
                    impls
                } => {
                    self.op_define_type(
                        addr,
                        Symbol::new_option(name.clone(), full_name.clone()),
                        body.clone(),
                        constructor.clone(),
                        impls.clone()
                    )?
                }
                Opcode::DefineUnit { addr, name, full_name, body } => {
                    self.op_define_unit(addr, Symbol::new_option(name.clone(), full_name.clone()), body.clone(), table)?
                }
                Opcode::DefineTrait {
                    addr,
                    name,
                    full_name,
                    functions
                } => {
                    self.op_define_trait(
                        addr,
                        Symbol::new_option(name.clone(), full_name.clone()),
                        functions.clone()
                    )?
                }
                Opcode::Define { addr, name, value, has_previous} => {
                    self.op_define(addr, name, *has_previous, value.clone(), table)?;
                }
                Opcode::Set { addr, name, value, has_previous } => {
                    self.op_set(addr, name, *has_previous, value.clone(), table)?;
                }
                Opcode::Load { addr, name, has_previous, should_push } => {
                    self.op_load(addr, name, *has_previous, *should_push, table)?;
                }
                Opcode::Call {
                    addr,
                    name,
                    has_previous,
                    should_push,
                    args
                } => {
                    self.op_call(addr, name, *has_previous, *should_push, args.clone(), table)?
                }
                Opcode::Duplicate { addr } => {
                    self.op_duplicate(addr)?;
                }
                Opcode::Instance { addr, name, args, should_push } => {
                    self.op_instance(addr, name, args.clone(), *should_push, table)?;
                }
                Opcode::EndLoop { addr, current_iteration } => {
                    self.op_endloop(addr, *current_iteration)?;
                }
                Opcode::Ret { addr, value } => {
                    self.op_return(addr, value.clone(), table)?;
                }
                Opcode::Native { addr, fn_name } => {
                    self.op_native(addr, fn_name)?;
                }
                Opcode::ErrorPropagation { addr, value } => {
                    self.op_error_propagation(addr, value.clone(), table)?;
                }
                Opcode::Impls { addr, value, trait_name } => {
                    self.op_impls(addr, value.clone(), trait_name, table)?;
                }
                Opcode::DeleteLocal { addr, name } => {
                    self.op_delete_local(addr, name, table)
                }
            }
        }
        Ok(())
    }
}

// имплементация для передачи между потоками
unsafe impl Send for VM {}
unsafe impl Sync for VM {}
