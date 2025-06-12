// импорты
use crate::parser::ast::Node;
use crate::errors::errors::{Error};
use std::collections::VecDeque;
use crate::lexer::address::Address;

// нода анализа
pub enum AnalyzerNode {
    Block,
    If,
    Loop,
    For,
    Fn
}
// семантический анализатор
pub struct Analyzer {
    analyze_stack: VecDeque<AnalyzerNode>,
}
// имплементация
impl Analyzer {
    // новый анализатор
    pub fn new() -> Self {
        Analyzer {
            analyze_stack: VecDeque::new(),
        }
    }

    // анализ ноды
    pub fn analyze(&mut self, node: Node) -> Result<(), Error> {
        match node.clone() {
            Node::Block { body } => {
                self.analyze_block(body)
            }
            Node::Number { .. } => { self.pass() }
            Node::String { .. } => { self.pass() }
            Node::Bool { .. } => { self.pass() }
            Node::Bin { .. } => { self.pass() }
            Node::Unary { .. } => { self.pass() }
            Node::If { logical, body, elseif, .. } => {
                self.analyze_if(body, logical, elseif)
            }
            Node::While { logical, body, .. } => {
                self.analyze_while(body, logical)
            }
            Node::Define { .. } => { self.pass() }
            Node::Assign { .. } => { self.pass() }
            Node::Get { .. } => { self.pass() }
            Node::Call { .. } => { self.pass() }
            Node::FnDeclaration { name, body, .. } => {
                self.analyze_fn_decl(name.address, body)
            }
            Node::AnFnDeclaration { .. } => { self.pass() }
            Node::Break { location } => { self.analyze_break(location.address) }
            Node::Continue { location } => { self.analyze_continue(location.address) }
            Node::Import { .. } => { self.pass() }
            Node::List { .. } => { todo!() }
            Node::Cond { .. } => { self.pass() }
            Node::Logical { .. } => { self.pass() }
            Node::Map { .. } => { todo!() }
            Node::Match { .. } => { todo!() }
            Node::Native { .. } => { todo!() }
            Node::Instance { .. } => { self.pass() }
            Node::Ret { location, .. } => { self.analyze_return(location.address) }
            Node::Null { .. } => { self.pass() }
            Node::Type { name, body, .. } => {
                self.analyze_type_decl(name.address, body)
            }
            Node::Unit { name, body, .. } => {
                self.analyze_unit_decl(name.address, body)
            }
            Node::For { .. } => { self.pass() }
        }
    }

    // проверка, есть ли в иерархии цикл
    fn hierarchy_has_loop(&mut self) -> bool {
        for node in self.analyze_stack.iter().rev() {
            if let AnalyzerNode::Loop = node {
                return true
            }
        }
        false
    }

    // проверка, есть ли в иерархии функция
    fn hierarchy_has_fn(&mut self) -> bool {
        for node in self.analyze_stack.iter().rev() {
            if let AnalyzerNode::Fn = node {
                return true
            }
        }
        false
    }

    // блок
    pub fn analyze_block(&mut self, body: Vec<Box<Node>>) -> Result<(), Error> {
        // ноды
        for node in body {
            self.analyze(*node)?;
        }
        // успех
        Ok(())
    }

    // иф
    pub fn analyze_if(&mut self, body: Box<Node>, logical: Box<Node>, elseif: Option<Box<Node>>) -> Result<(), Error> {
        // пушим
        self.analyze_stack.push_back(AnalyzerNode::Loop);
        self.analyze(*logical)?;
        self.analyze(*body)?;
        // попаем
        self.analyze_stack.pop_back();
        // else if
        if let Some(else_node) = elseif {
            self.analyze(*else_node)?;
        }
        // успех
        Ok(())
    }

    // цикл
    pub fn analyze_while(&mut self, body: Box<Node>, logical: Box<Node>) -> Result<(), Error> {
        // пушим
        self.analyze_stack.push_back(AnalyzerNode::If);
        self.analyze(*logical)?;
        self.analyze(*body)?;
        // попаем
        self.analyze_stack.pop_back();
        // успех
        Ok(())
    }

    // continue
    pub fn analyze_continue(&mut self, addr: Address) -> Result<(), Error> {
        // проверяем
        if self.analyze_stack.len() == 0 {
            return Err(Error::new(
                addr,
                "couldn't use continue without loop.".to_string(),
                "remove this keyword".to_string()
            ))
        }
        // проверяем loop
        if self.hierarchy_has_loop() {
            Ok(())
        } else {
            Err(Error::new(
                addr,
                "couldn't use continue without loop.".to_string(),
                "remove this keyword".to_string()
            ))
        }
    }

    // break
    pub fn analyze_break(&mut self, addr: Address) -> Result<(), Error> {
        // проверяем
        if self.analyze_stack.len() == 0 {
            return Err(Error::new(
                addr,
                "couldn't use break without loop.".to_string(),
                "remove this keyword".to_string()
            ))
        }
        // проверяем loop
        if self.hierarchy_has_loop() {
            Ok(())
        } else {
            Err(Error::new(
                addr,
                "couldn't use break without loop.".to_string(),
                "remove this keyword".to_string()
            ))
        }
    }

    // анализ декларации функции
    #[allow(unused_variables)]
    pub fn analyze_fn_decl(&mut self, addr: Address, body: Box<Node>) -> Result<(), Error> {
        // пуш в стек
        self.analyze_stack.push_back(AnalyzerNode::Fn);
        self.analyze(*body)?;
        self.analyze_stack.pop_back();
        // успех
        Ok(())
    }

    // анализ ретурн
    pub fn analyze_return(&mut self, addr: Address) -> Result<(), Error> {
        // проверяем
        if self.analyze_stack.len() == 0 {
            return Err(Error::new(
                addr,
                "couldn't use return without loop.".to_string(),
                "remove this keyword".to_string()
            ))
        }
        // проверяем fn
        if self.hierarchy_has_fn() {
            Ok(())
        } else {
            Err(Error::new(
                addr,
                "couldn't use break without loop.".to_string(),
                "remove this keyword".to_string()
            ))
        }
    }

    // анализ декларации типа
    #[allow(unused_variables)]
    pub fn analyze_type_decl(&mut self, addr: Address, body: Box<Node>) -> Result<(), Error> {
        // пуш в стек
        self.analyze(*body)?;
        // успех
        Ok(())
    }

    // анализ декларации юнита
    #[allow(unused_variables)]
    pub fn analyze_unit_decl(&mut self, addr: Address, body: Box<Node>) -> Result<(), Error> {
        // пуш в стек
        self.analyze(*body)?;
        // успех
        Ok(())
    }

    // пропуск
    pub fn pass(&self) -> Result<(), Error> {
        Ok(())
    }
}