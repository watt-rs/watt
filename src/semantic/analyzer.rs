// импорты
use crate::parser::ast::Node;
use crate::errors::errors::{Error};
use std::collections::VecDeque;
use crate::error;
use crate::lexer::address::Address;

// нода анализа
#[allow(dead_code)]
#[derive(Clone)]
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
    pub fn analyze(&mut self, node: Node) -> Node {
        match node.clone() {
            Node::Block { body } => {
                self.analyze_block(body);
            }
            Node::If { logical, body, elseif, .. } => {
                self.analyze_if(body, logical, elseif);
            }
            Node::While { logical, body, .. } => {
                self.analyze_while(body, logical);
            }
            Node::FnDeclaration { body, .. } => {
                self.analyze_fn_decl(body);
            }
            Node::Break { location } => { self.analyze_break(location.address); }
            Node::Continue { location } => { self.analyze_continue(location.address); }
            Node::List { .. } => { todo!() }
            Node::Map { .. } => { todo!() }
            Node::Match { .. } => { todo!() }
            Node::Ret { location, .. } => { self.analyze_return(location.address); }
            Node::Type { body, .. } => {
                self.analyze_type_decl( body);
            }
            Node::Unit { body, .. } => {
                self.analyze_unit_decl(body);
            }
            Node::Import { location, .. } => {
                self.analyze_import(location.address)
            }
            Node::ErrorPropagation { location, .. } => {
                self.analyze_error_propagation(location.address);
            }
            _ => {}
        }
        // возвращаем ноду обратно
        node
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
    fn hierarchy_has_fn(&self) -> bool {
        for node in self.analyze_stack.clone() {
            if let AnalyzerNode::Fn = node {
                return true
            }
        }
        false
    }

    // блок
    pub fn analyze_block(&mut self, body: Vec<Node>) {
        // ноды
        for node in body {
            self.analyze(node);
        }
    }

    // иф
    pub fn analyze_if(&mut self, body: Box<Node>, logical: Box<Node>, elseif: Option<Box<Node>>) {
        // пушим
        self.analyze_stack.push_back(AnalyzerNode::Loop);
        self.analyze(*logical);
        self.analyze(*body);
        // попаем
        self.analyze_stack.pop_back();
        // else if
        if let Some(else_node) = elseif {
            self.analyze(*else_node);
        }
    }

    // цикл
    fn analyze_while(&mut self, body: Box<Node>, logical: Box<Node>) {
        // пушим
        self.analyze_stack.push_back(AnalyzerNode::If);
        self.analyze(*logical);
        self.analyze(*body);
        // попаем
        self.analyze_stack.pop_back();
    }

    // continue
    fn analyze_continue(&mut self, addr: Address) {
        // проверяем
        if self.analyze_stack.len() == 0 {
            error!(Error::new(
                addr,
                "couldn't use continue without loop.".to_string(),
                "remove this keyword".to_string()
            ));
            return;
        }
        // проверяем loop
        if !self.hierarchy_has_loop() {
            error!(Error::new(
                addr,
                "couldn't use continue without loop.".to_string(),
                "remove this keyword".to_string()
            ));
        }
    }

    // break
    fn analyze_break(&mut self, addr: Address) {
        // проверяем
        if self.analyze_stack.is_empty() {
            error!(Error::new(
                addr,
                "couldn't use break without loop.".to_string(),
                "remove this keyword".to_string()
            ));
            return;
        }
        // проверяем loop
        if !self.hierarchy_has_loop() {
            error!(Error::new(
                addr,
                "couldn't use break without loop.".to_string(),
                "remove this keyword".to_string()
            ));
        }
    }

    // анализ декларации функции
    fn analyze_fn_decl(&mut self, body: Box<Node>) {
        // пуш в стек
        self.analyze_stack.push_back(AnalyzerNode::Fn);
        self.analyze(*body);
        self.analyze_stack.pop_back();
    }

    // анализ ретурн
    fn analyze_return(&mut self, addr: Address) {
        // проверяем
        if self.analyze_stack.is_empty() {
            error!(Error::new(
                addr,
                "couldn't use return without loop.".to_string(),
                "remove this keyword".to_string()
            ));
            return;
        }
        // проверяем fn
        if !self.hierarchy_has_fn() {
            error!(Error::new(
                addr,
                "couldn't use break without loop.".to_string(),
                "remove this keyword".to_string()
            ));
        }
    }

    // анализ декларации типа
    fn analyze_type_decl(&mut self,  body: Box<Node>) {
        // пуш в стек
        self.analyze(*body);
    }

    // анализ декларации юнита
    fn analyze_unit_decl(&mut self, body: Box<Node>) {
        // пуш в стек
        self.analyze(*body);
    }

    // анализ импорта
    fn analyze_import(&self, addr: Address) {
        // проверка размера стека вложенности
        if self.analyze_stack.len() > 0 {
            error!(Error::new(
                addr,
                "couldn't use import in any block.".to_string(),
                "you can use import only in main scope.".to_string()
            ))
        }
    }

    // анализ error propagation
    fn analyze_error_propagation(&self, addr: Address) {
        // проверка размера стека вложенности
        if !self.hierarchy_has_fn() {
            error!(Error::new(
                addr,
                "couldn't use error propagation outside fn.".to_string(),
                "you can use it only inside functions.".to_string()
            ))
        }
    }    
}