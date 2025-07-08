// импорты
use crate::parser::ast::{MatchCase, Node};
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
    pub fn analyze<'node>(&mut self, node: &'node Node) -> &'node Node {
        match node {
            Node::Block { body } => {
                for node in body {
                    self.analyze(&node);
                }
            }
            Node::If { logical, body, elseif, .. } => {
                self.analyze_if(body, logical, elseif);
            }
            Node::While { logical, body, .. } => {
                self.analyze_while(body, logical);
            }
            Node::For { iterable, body, .. } => {
                self.analyze_for(body, iterable);
            }
            Node::FnDeclaration { body, .. } => {
                self.analyze_fn(body);
            }
            Node::Break { location } => {
                self.analyze_break(&location.address);
            }
            Node::Continue { location } => {
                self.analyze_continue(&location.address);
            }
            Node::List { values, .. } => {
                for value in values {
                    self.analyze(value);
                }
            }
            Node::Map { values, .. } => {
                for (k, v) in values {
                    self.analyze(k);
                    self.analyze(v);
                }
            }
            Node::Match { cases, default, .. } => { 
                self.analyze_match(cases, default);
            }
            Node::Ret { location, .. } => {
                self.analyze_return(&location.address);
            }
            Node::Type { body, .. } => {
                self.analyze(body);
            }
            Node::Unit { body, .. } => {
                self.analyze(body);
            }
            Node::Import { location, .. } => {
                self.analyze_import(&location.address)
            }
            Node::ErrorPropagation { location, .. } => {
                self.analyze_error_propagation(&location.address);
            }
            Node::Call { args, .. } => {
                for arg in args {
                    self.analyze(arg);
                }
            }
            Node::Define { value, .. } => {
                self.analyze(value);
            }
            Node::Unary { value, .. } => {
                self.analyze(value);
            }
            Node::Bin { left, right, .. } => {
                self.analyze(left);
                self.analyze(right);
            }
            Node::Instance { constructor, .. } => {
                for arg in constructor {
                    self.analyze(arg);
                }
            }
            Node::Assign { value, .. } => {
                self.analyze(value);
            }
            Node::AnFnDeclaration { body, .. } => {
                self.analyze_fn(body);
            }
            Node::Cond { left, right, .. } => {
                self.analyze(left);
                self.analyze(right);
            }
            Node::Logical { left, right, .. } => {
                self.analyze(left);
                self.analyze(right);
            }
            Node::Range { from, to, .. } => {
                self.analyze(from);
                self.analyze(to);
            }
            Node::Impls { value, .. } => {
                self.analyze(value);
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

    // иф
    pub fn analyze_if(&mut self, body: &Node, logical: &Node, elseif: &Option<Box<Node>>) {
        // пушим
        self.analyze_stack.push_back(AnalyzerNode::If);
        self.analyze(logical);
        self.analyze(body);
        // попаем
        self.analyze_stack.pop_back();
        // else if
        if let Some(else_node) = elseif {
            self.analyze(else_node);
        }
    }

    // match
    pub fn analyze_match(&mut self, cases: &Vec<MatchCase>, default: &Node) {
        // анализ
        self.analyze(default);
        for case in cases {
            self.analyze(&*case.body);
        }
    }
    
    // цикл
    fn analyze_while(&mut self, body: &Node, logical: &Node) {
        // пушим
        self.analyze_stack.push_back(AnalyzerNode::Loop);
        self.analyze(logical);
        self.analyze(body);
        // попаем
        self.analyze_stack.pop_back();
    }

    // анализ цикла for
    pub fn analyze_for(&mut self, body: &Node, iterable: &Node) {
        // пушим
        self.analyze_stack.push_back(AnalyzerNode::Loop);
        self.analyze(iterable);
        self.analyze(body);
        // попаем
        self.analyze_stack.pop_back();
    }

    // continue
    fn analyze_continue(&mut self, addr: &Address) {
        // проверяем
        if self.analyze_stack.len() == 0 {
            error!(Error::new(
                addr.clone(),
                "couldn't use continue without loop.",
                "remove this keyword"
            ));
            return;
        }
        // проверяем loop
        if !self.hierarchy_has_loop() {
            error!(Error::new(
                addr.clone(),
                "couldn't use continue without loop.",
                "remove this keyword"
            ));
        }
    }

    // break
    fn analyze_break(&mut self, addr: &Address) {
        // проверяем
        if self.analyze_stack.is_empty() {
            error!(Error::new(
                addr.clone(),
                "couldn't use break without loop.",
                "remove this keyword"
            ));
            return;
        }
        // проверяем loop
        if !self.hierarchy_has_loop() {
            error!(Error::new(
                addr.clone(),
                "couldn't use break without loop.",
                "remove this keyword"
            ));
        }
    }

    // анализ декларации функции
    fn analyze_fn(&mut self, body: &Node) {
        // пуш в стек
        self.analyze_stack.push_back(AnalyzerNode::Fn);
        self.analyze(body);
        self.analyze_stack.pop_back();
    }

    // анализ ретурн
    fn analyze_return(&mut self, addr: &Address) {
        // проверяем
        if self.analyze_stack.is_empty() {
            error!(Error::new(
                addr.clone(),
                "couldn't use return without loop.",
                "remove this keyword"
            ));
            return;
        }
        // проверяем fn
        if !self.hierarchy_has_fn() {
            error!(Error::new(
                addr.clone(),
                "couldn't use break without loop.",
                "remove this keyword"
            ));
        }
    }
    
    // анализ импорта
    fn analyze_import(&self, addr: &Address) {
        // проверка размера стека вложенности
        if self.analyze_stack.len() > 0 {
            error!(Error::new(
                addr.clone(),
                "couldn't use import in any block.",
                "you can use import only in main scope."
            ))
        }
    }

    // анализ error propagation
    fn analyze_error_propagation(&self, addr: &Address) {
        // проверка размера стека вложенности
        if !self.hierarchy_has_fn() {
            error!(Error::new(
                addr.clone(),
                "couldn't use error propagation outside fn.",
                "you can use it only inside functions."
            ))
        }
    }
}