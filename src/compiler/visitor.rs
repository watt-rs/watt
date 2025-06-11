// импорты
use crate::parser::import::Import;
use crate::lexer::lexer::*;
use crate::parser::ast::*;
use crate::vm::bytecode::{Chunk, Opcode};
use crate::vm::values::*;
use crate::vm::memory::memory;
use std::collections::VecDeque;
use crate::errors::errors::{Error, ErrorType};

// визитор (компилятор)
pub struct CompileVisitor {
    opcodes: VecDeque<Vec<Opcode>>,
}

// имплементация визитора
#[allow(unused_variables)]
impl CompileVisitor {
    // новый визитор
    pub fn new() -> Self {
        CompileVisitor {
            opcodes: VecDeque::new(),
        }
    }
    // компиляция
    pub fn compile(&mut self, node: Node) -> Result<Chunk, Error> {
        self.push_chunk();
        match self.visit_node(node.clone()) {
            Err(e) => Err(e),
            Ok(_) => Ok(Chunk::new(self.pop_chunk().clone())),
        }
    }

    // пушим чанк
    pub fn push_chunk(&mut self) {
        self.opcodes.push_front(vec![]);
    }

    // попаем чанк
    pub fn pop_chunk(&mut self) -> Vec<Opcode> {
        match self.opcodes.pop_front() {
            Some(v) => v,
            None => panic!("couldn't pop from compiler-visitor stack. report to the developer."),
        }
    }

    // пушим инструкцию
    pub fn push_instr(&mut self, op: Opcode) {
        match self.opcodes.front_mut() {
            Some(v) => v.push(op),
            None => {
                panic!("couldn't push instr to compiler-visitor stack. report to the developer.")
            }
        }
    }

    // попаем инструкцию
    pub fn pop_instr(&mut self) -> Opcode {
        match self.opcodes.front_mut() {
            Some(v) => v.pop().unwrap(),
            None => {
                panic!("couldn't pop instr from compiler-visitor stack. report to the developer.")
            }
        }
    }

    // визит ноды
    pub fn visit_node(&mut self, node: Node) -> Result<(), Error> {
        match node {
            Node::Number { value } => self.visit_number(value),
            Node::String { value } => self.visit_string(value),
            Node::Bool { value } => self.visit_bool(value),
            Node::Bin { left, right, op } => self.visit_binary(left, right, op),
            Node::Unary { value, op } => self.visit_unary(value, op),
            Node::If {
                location,
                logical,
                body,
                elseif,
            } => self.visit_if(location, logical, body, elseif),
            Node::While {
                location,
                logical,
                body,
            } => self.visit_while(location, logical, body),
            Node::Define {
                previous,
                name,
                value,
            } => self.visit_define(previous, name, value),
            Node::Assign {
                previous,
                name,
                value,
            } => self.visit_assign(previous, name, value),
            Node::Get {
                previous,
                name,
                should_push,
            } => self.visit_get(previous, name, should_push),
            Node::Call {
                previous,
                name,
                args,
                should_push,
            } => self.visit_call(previous, name, args, should_push),
            Node::FnDeclaration {
                name,
                full_name,
                params,
                body,
            } => self.visit_fn_decl(name, full_name, params, body),
            Node::AnFnDeclaration { params, body, .. } => self.visit_an_fn_decl(params, body),
            Node::Break { location } => self.visit_break(location),
            Node::Continue { location } => self.visit_continue(location),
            Node::Import { imports } => self.visit_import(imports),
            Node::List { location, values } => self.visit_list(location, values),
            Node::Cond { left, right, op } => self.visit_cond(left, right, op),
            Node::Logical { left, right, op } => self.visit_logical(left, right, op),
            Node::Map { location, values } => self.visit_map(location, values),
            Node::Match {
                location,
                matchable,
                cases,
                default,
            } => self.visit_match(location, matchable, cases, default),
            Node::Native { name, fn_name } => self.visit_native(name, fn_name),
            Node::Instance {
                name,
                constructor,
                should_push,
            } => self.visit_instance(name, constructor, should_push),
            Node::Ret { location, value } => self.visit_return(location, value),
            Node::Null { location } => self.visit_null(location),
            Node::Type {
                name,
                full_name,
                constructor,
                body,
            } => self.visit_type(name, full_name, constructor, body),
            Node::Unit {
                name,
                full_name,
                body,
            } => self.visit_unit(name, full_name, body),
            Node::For {
                iterable,
                variable_name,
                body,
            } => self.visit_for(iterable, variable_name, body),
            Node::Block { body } => self.visit_block(body),
        }
    }

    // блок
    pub fn visit_block(&mut self, body: Vec<Box<Node>>) -> Result<(), Error> {
        // перебор и компиляция нод
        for node in body {
            self.visit_node(*node)?
        }
        // успех
        Ok(())
    }

    // визит числа
    pub fn visit_number(&mut self, value: Token) -> Result<(), Error> {
        // пуш флоата
        if value.value.contains(".") {
            self.push_instr(Opcode::Push {
                addr: value.address.clone(),
                value: Value::Float(value.value.parse::<f64>().unwrap()),
            });
        }
        // пуш инта
        else {
            self.push_instr(Opcode::Push {
                addr: value.address.clone(),
                value: Value::Int(value.value.parse::<i64>().unwrap()),
            });
        }
        // успех
        Ok(())
    }

    // визит строки
    pub fn visit_string(&mut self, value: Token) -> Result<(), Error> {
        // пуш строки
        self.push_instr(Opcode::Push {
            addr: value.address.clone(),
            value: Value::String(memory::alloc_value(value.value)),
        });
        // успех
        Ok(())
    }

    // визит була
    pub fn visit_bool(&mut self, value: Token) -> Result<(), Error> {
        // пуш бул
        self.push_instr(Opcode::Push {
            addr: value.address.clone(),
            value: Value::Bool(value.value.parse::<bool>().unwrap()),
        });
        // успех
        Ok(())
    }

    // бинарая операция
    pub fn visit_binary(&mut self, left: Box<Node>, right: Box<Node>, op: Token) -> Result<(), Error> {
        // правая часть
        self.visit_node(*right)?;
        // левая часть
        self.visit_node(*left)?;
        // бинарная операция
        self.push_instr(Opcode::Bin {
            addr: op.address.clone(),
            op: op.value,
        });
        // успех
        Ok(())
    }

    // блок if
    pub fn visit_if(
        &mut self,
        location: Token,
        logical: Box<Node>,
        body: Box<Node>,
        elif: Option<Box<Node>>,
    ) -> Result<(), Error> {
        // компиляция if
        // чанк условия
        self.push_chunk();
        self.visit_node(*logical)?;
        let logical_chunk = self.pop_chunk();
        // чанк тела
        self.push_chunk();
        self.visit_node(*body)?;
        let body_chunk = self.pop_chunk();
        // компиляция elif
        let mut elseif: Option<Box<Opcode>> = None;
        // если есть
        if let Some(n) = elif {
            self.visit_node(*n)?;
            elseif = Some(Box::new(self.pop_instr()));
        }
        // возвращаем if
        self.push_instr(Opcode::If {
            addr: location.address.clone(),
            cond: Box::new(Chunk::new(logical_chunk)),
            body: Box::new(Chunk::new(body_chunk)),
            elif: elseif,
        });
        Ok(())
    }

    // блок while
    pub fn visit_while(
        &mut self,
        location: Token,
        logical: Box<Node>,
        body: Box<Node>,
    ) -> Result<(), Error> {
        // чанк логики
        self.push_chunk();
        self.visit_node(*logical)?;
        let logical_chunk = self.pop_chunk();
        // чанк тела
        self.push_chunk();
        self.visit_node(*body)?;
        let body_chunk = self.pop_chunk();
        // опкод условия
        let if_opcode = Opcode::If {
            addr: location.address.clone(),
            cond: Box::new(Chunk::new(logical_chunk)),
            body: Box::new(Chunk::new(body_chunk)),
            elif: Some(Box::new(Opcode::If {
                addr: location.address.clone(),
                cond: Box::new(Chunk::of(Opcode::Push{ addr: location.address.clone(), value: Value::Bool(true) })),
                body: Box::new(Chunk::of(Opcode::EndLoop { addr: location.address.clone(), current_iteration: false })),
                elif: None
            })),
        };
        // цикл
        self.push_instr(Opcode::Loop {
            addr: location.address.clone(),
            body: Box::new(Chunk::of(if_opcode)),
        });
        // успех
        Ok(())
    }

    // дефайн переменной
    pub fn visit_define(
        &mut self,
        previous: Option<Box<Node>>,
        name: Token,
        value: Box<Node>,
    ) -> Result<(), Error> {
        // есть ли предыдущая нода
        let mut has_previous = false;
        // если есть
        if let Some(prev) = previous {
            self.visit_node(*prev)?;
            has_previous = true;
        }
        // чанк для значения
        self.push_chunk();
        self.visit_node(*value)?;
        let value_chunk = self.pop_chunk();
        // дефайн
        self.push_instr(Opcode::Define {
            addr: name.address.clone(),
            name: name.value,
            value: Box::new(Chunk::new(value_chunk)),
            has_previous,
        });
        // успех
        Ok(())
    }

    // вызов функции
    pub fn visit_call(
        &mut self,
        previous: Option<Box<Node>>,
        name: Token,
        args: Vec<Box<Node>>,
        should_push: bool,
    ) -> Result<(), Error> {
        // есть ли предыдущая нода
        let mut has_previous = false;
        // если есть
        if let Some(prev) = previous {
            self.visit_node(*prev)?;
            has_previous = true;
        }
        // чанка аргументов
        self.push_chunk();
        self.visit_block(args)?;
        let args_chunk = self.pop_chunk();
        // вызов
        self.push_instr(Opcode::Call {
            addr: name.address.clone(),
            name: name.value,
            args: Box::new(Chunk::new(args_chunk)),
            has_previous,
            should_push,
        });
        // успех
        Ok(())
    }

    // дефайн функции
    pub fn visit_fn_decl(
        &mut self,
        name: Token,
        full_name: Option<Token>,
        parameters: Vec<Token>,
        body: Box<Node>,
    ) -> Result<(), Error> {
        // полное имя
        let full_name = match full_name {
            Some(n) => Some(n.value),
            None => None,
        };
        // параметры
        let mut params = Vec::new();
        for param in parameters {
            params.push(param.value);
        }
        // чанк тела
        self.push_chunk();
        self.visit_node(*body)?;
        self.visit_node(Node::Ret {
            location: name.clone(),
            value: Box::new(Node::Null {
                location: name.clone(),
            })
        })?;
        let chunk = self.pop_chunk();
        // дефайн функции
        self.push_instr(Opcode::DefineFn {
            addr: name.address.clone(),
            name: name.value.clone(),
            full_name,
            params,
            body: Box::new(Chunk::new(chunk)),
        });
        // замыкание
        self.push_instr(Opcode::Closure {
            addr: name.address.clone(),
            name: name.value

        });
        // успех
        Ok(())
    }

    // визит break
    pub fn visit_break(&mut self, location: Token) -> Result<(), Error> {
        // завершения цикла
        self.push_instr(Opcode::EndLoop {
            addr: location.address.clone(),
            current_iteration: false,
        });
        // успех
        Ok(())
    }

    // визит continue
    pub fn visit_continue(&mut self, location: Token) -> Result<(), Error> {
        // скип итерации цикла
        self.push_instr(Opcode::EndLoop {
            addr: location.address.clone(),
            current_iteration: true,
        });
        // успех
        Ok(())
    }

    // todo: import
    pub fn visit_import(&mut self, imports: Vec<Import>) -> Result<(), Error> {
        todo!()
    }

    // todo: list
    pub fn visit_list(&mut self, location: Token, list: Box<Vec<Box<Node>>>) -> Result<(), Error> {
        todo!()
    }

    // todo: map
    pub fn visit_map(
        &mut self,
        location: Token,
        map: Box<Vec<(Box<Node>, Box<Node>)>>,
    ) -> Result<(), Error> {
        todo!()
    }

    // todo: for
    pub fn visit_for(
        &mut self,
        iterable: Box<Node>,
        variable_name: Token,
        body: Box<Node>,
    ) -> Result<(), Error> {
        todo!()
    }

    // todo: match
    pub fn visit_match(
        &mut self,
        location: Token,
        matchable: Box<Node>,
        cases: Vec<Box<Node>>,
        default: Box<Node>,
    ) -> Result<(), Error> {
        todo!()
    }

    // todo: anonymous fn
    pub fn visit_an_fn_decl(&mut self, args: Vec<Token>, body: Box<Node>) -> Result<(), Error> {
        todo!()
    }

    // todo: native
    pub fn visit_native(&mut self, name: Token, fn_name: Token) -> Result<(), Error> {
        todo!()
    }

    // унарная операция
    pub fn visit_unary(&mut self, value: Box<Node>, op: Token) -> Result<(), Error> {
        // перебираем оператор
        match op.value.as_str() {
            // оператор -
            "-" => self.push_instr(Opcode::Neg {
                addr: op.address.clone(),
            }),
            // оператор !
            "!" => self.push_instr(Opcode::Bang {
                addr: op.address.clone(),
            }),
            // неизвестный оператор
            _ => {
                return Err(Error::new(
                    ErrorType::Compilation,
                    op.address,
                    format!("undefined unary op: {:?}", op.value),
                    "available: -, !".to_string(),
                ))
            }
        }
        // успех
        Ok(())
    }

    // визит типа
    pub fn visit_type(
        &mut self,
        name: Token,
        full_name: Option<Token>,
        constructor: Vec<Token>,
        body: Box<Node>,
    ) -> Result<(), Error> {
        // полное имя
        let full_name = match full_name {
            Some(name) => Some(name.value),
            None => None,
        };
        // конструктор
        let mut constructor_params = Vec::new();
        for param in constructor {
            constructor_params.push(param.value);
        }
        // тело типа
        self.push_chunk();
        self.visit_node(*body)?;
        let chunk = self.pop_chunk();
        // дефайн типа
        self.push_instr(Opcode::DefineType {
            addr: name.address.clone(),
            name: name.value,
            full_name,
            constructor: constructor_params,
            body: Box::new(Chunk::new(chunk)),
        });
        // успех
        Ok(())
    }

    // визит юнита
    pub fn visit_unit(
        &mut self,
        name: Token,
        full_name: Option<Token>,
        body: Box<Node>,
    ) -> Result<(), Error> {
        // полное имя
        let full_name = match full_name {
            Some(name) => Some(name.value),
            None => None,
        };
        // тело юнита
        self.push_chunk();
        self.visit_node(*body)?;
        let chunk = self.pop_chunk();
        // дефайн юнита
        self.push_instr(Opcode::DefineUnit {
            addr: name.address.clone(),
            name: name.value,
            full_name,
            body: Box::new(Chunk::new(chunk)),
        });
        // успех
        Ok(())
    }

    // визит условия
    pub fn visit_cond(
        &mut self,
        left: Box<Node>,
        right: Box<Node>,
        op: Token,
    ) -> Result<(), Error> {
        // правая и левая ноды
        self.visit_node(*right)?;
        self.visit_node(*left)?;
        // условие
        self.push_instr(Opcode::Cond {
            addr: op.address.clone(),
            op: op.value,
        });
        // успех
        Ok(())
    }

    // визит логическово выражения
    pub fn visit_logical(
        &mut self,
        left: Box<Node>,
        right: Box<Node>,
        op: Token,
    ) -> Result<(), Error> {
        // правая и левая ноды
        self.visit_node(*right)?;
        self.visit_node(*left)?;
        // логический опкод
        self.push_instr(Opcode::Logic {
            addr: op.address.clone(),
            op: op.value,
        });
        // успех
        Ok(())
    }

    // визит возврата значения
    pub fn visit_return(&mut self, location: Token, value: Box<Node>) -> Result<(), Error> {
        // чанк значения
        self.push_chunk();
        self.visit_node(*value)?;
        let chunk = self.pop_chunk();
        // ретурн
        self.push_instr(Opcode::Ret {
            addr: location.address.clone(),
            value: Box::new(Chunk::new(chunk)),
        });
        // успех
        Ok(())
    }

    // нулл значение
    pub fn visit_null(&mut self, location: Token) -> Result<(), Error> {
        // нулл значение
        self.push_instr(Opcode::Push {
            addr: location.address.clone(),
            value: Value::Null,
        });
        // успех
        Ok(())
    }

    // визит инстанса
    pub fn visit_instance(
        &mut self,
        name: Token,
        constructor: Vec<Box<Node>>,
        should_push: bool,
    ) -> Result<(), Error> {
        // конструктор
        self.push_chunk();
        for arg in constructor {
            self.visit_node(*arg)?;
        }
        let constructor_args = self.pop_chunk();
        // инстанс
        self.push_instr(Opcode::Instance {
            addr: name.address.clone(),
            name: name.value,
            args: Box::new(Chunk::new(constructor_args)),
            should_push,
        });
        // успех
        Ok(())
    }

    // установка значения переменной
    pub fn visit_assign(
        &mut self,
        previous: Option<Box<Node>>,
        name: Token,
        value: Box<Node>,
    ) -> Result<(), Error> {
        // есть ли предыдущая нода
        let mut has_previous = false;
        if let Some(prev) = previous {
            self.visit_node(*prev)?;
            has_previous = true;
        }
        // чанк значения
        self.push_chunk();
        self.visit_node(*value)?;
        let chunk = self.pop_chunk();
        // установка значения переменной
        self.push_instr(Opcode::Set {
            addr: name.address.clone(),
            name: name.value,
            value: Box::new(Chunk::new(chunk)),
            has_previous,
        });
        // успех
        Ok(())
    }

    // получение значения переменной
    pub fn visit_get(
        &mut self,
        previous: Option<Box<Node>>,
        name: Token,
        should_push: bool,
    ) -> Result<(), Error> {
        // есть ли предыдущая нода
        let mut has_previous = false;
        if let Some(prev) = previous {
            self.visit_node(*prev)?;
            has_previous = true;
        }
        // загрузка переменной
        self.push_instr(Opcode::Load {
            addr: name.address.clone(),
            name: name.value,
            has_previous,
            should_push,
        });
        // успех
        Ok(())
    }
}
