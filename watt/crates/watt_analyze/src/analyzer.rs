// imports
use std::collections::VecDeque;
use watt_ast::ast::{MatchCase, Node};
use watt_common::address::Address;
use watt_common::{error, errors::Error};

/// Analyzer node type
///
/// Used in `analyzer_stack` to push
/// and pop lexical scopes from analyze_stack
///
#[allow(dead_code)]
#[derive(Clone)]
pub enum AnalyzerNode {
    Block,
    If,
    Loop,
    For,
    Fn,
}

/// Semantic analyzer
pub struct Analyzer {
    analyze_stack: VecDeque<AnalyzerNode>,
}
/// Semantic analyzer implementation
impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    /// New analyzer
    pub fn new() -> Self {
        Analyzer {
            analyze_stack: VecDeque::new(),
        }
    }

    /// Analyzes node
    pub fn analyze(&mut self, node: &Node) {
        match node {
            Node::Block { body } => {
                for node in body {
                    self.analyze(node);
                }
            }
            Node::If {
                logical,
                body,
                elseif,
                ..
            } => {
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
            Node::Import { location, .. } => self.analyze_import(&location.address),
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
    }

    /// Checks if analyze_stack has loop in hierarchy
    /// * hierarchy is analyze_stack
    fn hierarchy_has_loop(&mut self) -> bool {
        for node in self.analyze_stack.iter().rev() {
            if let AnalyzerNode::Loop = node {
                return true;
            }
        }
        false
    }

    /// Checks if analyze_stack has fn in hierarchy
    /// * hierarchy is analyze_stack
    fn hierarchy_has_fn(&self) -> bool {
        for node in self.analyze_stack.clone() {
            if let AnalyzerNode::Fn = node {
                return true;
            }
        }
        false
    }

    /// Analyzes if
    pub fn analyze_if(&mut self, body: &Node, logical: &Node, elseif: &Option<Box<Node>>) {
        // push if node to analyzer stack and analyze if
        self.analyze_stack.push_back(AnalyzerNode::If);
        self.analyze(logical);
        self.analyze(body);
        self.analyze_stack.pop_back();
        // analyze else if
        if let Some(else_node) = elseif {
            self.analyze(else_node);
        }
    }

    /// Analyzes match
    pub fn analyze_match(&mut self, cases: &Vec<MatchCase>, default: &Node) {
        // qnalyzing cases
        self.analyze(default);
        for case in cases {
            self.analyze(&case.body);
        }
    }

    /// Analyzing loop while
    fn analyze_while(&mut self, body: &Node, logical: &Node) {
        self.analyze_stack.push_back(AnalyzerNode::Loop);
        self.analyze(logical);
        self.analyze(body);
        self.analyze_stack.pop_back();
    }

    /// Analyzing loop for
    pub fn analyze_for(&mut self, body: &Node, iterable: &Node) {
        self.analyze_stack.push_back(AnalyzerNode::Loop);
        self.analyze(iterable);
        self.analyze(body);
        self.analyze_stack.pop_back();
    }

    /// Analyzing continue
    ///
    /// Checking has_loop_in_hierarchy
    /// raises error, if it's no loop is analyze_stack
    ///
    fn analyze_continue(&mut self, addr: &Address) {
        if self.analyze_stack.is_empty() {
            error!(Error::new(
                addr.clone(),
                "couldn't use continue without loop.",
                "remove this keyword"
            ));
        }
        if !self.hierarchy_has_loop() {
            error!(Error::new(
                addr.clone(),
                "couldn't use continue without loop.",
                "remove this keyword"
            ));
        }
    }

    /// Analyzing break
    ///
    /// Checking has_loop_in_hierarchy
    /// raises error, if it's no loop is analyze_stack
    ///
    fn analyze_break(&mut self, addr: &Address) {
        if self.analyze_stack.is_empty() {
            error!(Error::new(
                addr.clone(),
                "couldn't use break without loop.",
                "remove this keyword"
            ));
        }
        if !self.hierarchy_has_loop() {
            error!(Error::new(
                addr.clone(),
                "couldn't use break without loop.",
                "remove this keyword"
            ));
        }
    }

    /// Analyzing fn declaration
    fn analyze_fn(&mut self, body: &Node) {
        self.analyze_stack.push_back(AnalyzerNode::Fn);
        self.analyze(body);
        self.analyze_stack.pop_back();
    }

    /// Analyzing return
    ///
    /// Checking has_fn_in_hierarchy
    /// raises error, if it's no fn is analyze_stack
    ///
    fn analyze_return(&mut self, addr: &Address) {
        if self.analyze_stack.is_empty() {
            error!(Error::new(
                addr.clone(),
                "couldn't use return without loop.",
                "remove this keyword"
            ));
        }
        if !self.hierarchy_has_fn() {
            error!(Error::new(
                addr.clone(),
                "couldn't use break without loop.",
                "remove this keyword"
            ));
        }
    }

    /// Analyzing import
    ///
    /// Checks if analyze stack is empty, because
    /// imports are only allowed in global table
    ///
    /// If stack isn't empty, raises error
    ///
    fn analyze_import(&self, addr: &Address) {
        if !self.analyze_stack.is_empty() {
            error!(Error::new(
                addr.clone(),
                "couldn't use import in any block.",
                "you can use import only in main scope."
            ))
        }
    }

    /// Error propagation analyze
    ///
    /// Checking has_fn_in_hierarchy
    /// raises error, if it's no fn is analyze_stack
    ///
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
