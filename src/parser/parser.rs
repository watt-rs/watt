// импорты
use crate::lexer::address::*;
use crate::errors::errors::{Error};
use crate::parser::import::Import;
use crate::lexer::lexer::*;
use crate::parser::ast::*;
use crate::error;
use std::path::PathBuf;

// парсер
pub struct Parser<'file_path, 'prefix> {
    tokens: Vec<Token>,
    current: u128,
    file_path: &'file_path PathBuf,
    full_name_prefix: &'prefix str,
}
// имплементация
#[allow(unused_qualifications)]
impl<'file_path, 'prefix> Parser<'file_path, 'prefix> {
    // новый
    pub fn new(tokens: Vec<Token>, file_path: &'file_path PathBuf, full_name_prefix: &'prefix str) -> Self {
        Parser { tokens, current: 0, file_path, full_name_prefix }
    }
    
    // блок
    fn block(&mut self) -> Result<Node, Error> {
        // список
        let mut nodes: Vec<Node> = Vec::new();
        // до } или конца файла
        while !self.is_at_end() && !self.check(TokenKind::Rbrace) {
            nodes.push(self.statement()?);
        }
        // возвращаем
        Ok(Node::Block {
            body: nodes
        })
    }

    // аргументы
    fn args(&mut self) -> Result<Vec<Node>, Error> {
        // список
        let mut nodes: Vec<Node> = Vec::new();
        // (
        self.consume(TokenKind::Lparen)?;
        // до )
        if !self.check(TokenKind::Rparen) {
            // аргумент
            nodes.push(self.expr()?);
            // через запятую
            while !self.is_at_end() && self.check(TokenKind::Comma) {
                // ,
                self.consume(TokenKind::Comma)?;
                // аргумент
                nodes.push(self.expr()?);
            }
        }
        // )
        self.consume(TokenKind::Rparen)?;
        // возвращаем
        Ok(nodes)
    }

    // параметры
    fn params(&mut self) -> Result<Vec<Token>, Error> {
        // список
        let mut nodes: Vec<Token> = Vec::new();
        // (
        self.consume(TokenKind::Lparen)?;
        // до )
        if !self.check(TokenKind::Rparen) {
            // параметр
            nodes.push(self.consume(TokenKind::Id)?.clone());
            // через запятую
            while !self.is_at_end() && self.check(TokenKind::Comma) {
                // ,
                self.consume(TokenKind::Comma)?;
                // параметр
                nodes.push(self.consume(TokenKind::Id)?.clone());
            }
        }
        // )
        self.consume(TokenKind::Rparen)?;
        // возвращаем
        Ok(nodes)
    }

    // преобразовывает name в full name
    fn to_full_name(&self, tk: Token) -> Token{
        Token::new(
            TokenKind::Text,
            format!("{}:{}", self.full_name_prefix, tk.value),
            tk.address,
        )
    }

    // парсинг new
    fn object_creation_expr(&mut self) -> Result<Node, Error> {
        // new
        self.consume(TokenKind::New)?;
        // имя и аргументы
        let name = self.consume(TokenKind::Id)?.clone();
        let args = self.args()?;
        // возвращаем
        Ok(Node::Instance {
            name,
            constructor: args,
            should_push: true
        })
    }

    // часть access
    fn access_part(&mut self, previous: Option<Box<Node>>) -> Result<Node, Error> {
        // если есть id
        if self.check(TokenKind::Id) {
            // id
            let identifier = self.consume(TokenKind::Id)?.clone();
            // дефайн ':='
            if self.check(TokenKind::Walrus) {
                self.consume(TokenKind::Walrus)?;
                Ok(Node::Define {
                    previous,
                    name: identifier,
                    value: Box::new(self.expr()?),
                })
            }
            // присваивание '='
            else if self.check(TokenKind::Assign) {
                self.consume(TokenKind::Assign)?;
                Ok(Node::Assign {
                    previous,
                    name: identifier,
                    value: Box::new(self.expr()?),
                })
            }
            // +=, -=, *=, /=
            else if self.check(TokenKind::AssignAdd) ||
                self.check(TokenKind::AssignSub) ||
                self.check(TokenKind::AssignMul) ||
                self.check(TokenKind::AssignDiv) {
                // оператор и локация
                let op;
                let location;
                // парсим
                match self.peek()?.tk_type {
                    TokenKind::AssignSub => {
                        location = self.consume(TokenKind::AssignSub)?.clone();
                        op = "-";
                    }
                    TokenKind::AssignMul => {
                        location = self.consume(TokenKind::AssignMul)?.clone();
                        op = "*";
                    }
                    TokenKind::AssignDiv => {
                        location = self.consume(TokenKind::AssignDiv)?.clone();
                        op = "/";
                    }
                    TokenKind::AssignAdd => {
                        location = self.consume(TokenKind::AssignAdd)?.clone();
                        op = "+";
                    }
                    _ => {
                        panic!("invalid AssignOp tk_type. report to developer.");
                    }
                }
                // нода для получения значения
                let var = Node::Get {
                    previous: previous.clone(),
                    name: identifier.clone(),
                    should_push: true
                };
                // нода для присваивания
                return Ok(Node::Assign {
                    previous,
                    name: identifier,
                    value: Box::new(Node::Bin {
                        left: Box::new(var),
                        right: Box::new(self.expr()?),
                        op: Token::new(
                            TokenKind::Op,
                            op.to_string(),
                            location.address,
                        )
                    }),
                });
            }
            // вызов функции
            else if self.check(TokenKind::Lparen) {
                return Ok(Node::Call {
                    previous,
                    name: identifier,
                    args: self.args()?,
                    should_push: true
                });
            }
            // получение значения переменной
            else {
                return Ok(Node::Get {
                    previous,
                    name: identifier,
                    should_push: true
                })
            }
        }
        // в ином случае - парсинг new
        else {
            Ok(self.object_creation_expr()?)
        }
    }

    // парсинг access
    fn access(&mut self, is_expr: bool) -> Result<Node, Error> {
        // access part
        let mut left = self.access_part(Option::None)?;
        // через точку
        while self.check(TokenKind::Dot) {
            // .
            self.consume(TokenKind::Dot)?;
            // адрес
            let location = self.peek()?.address.clone();
            // access part
            left = self.access_part(Option::Some(Box::new(left)))?;
            // если выражение
            if !is_expr { continue; }
            // если не выражение
            match left {
                Node::Define { .. } => {
                    return Err(Error::new(
                        location,
                        "couldn't use define in expr.",
                        "check your code.",
                    ))
                }
                Node::Assign { .. } => {
                    return Err(Error::new(
                        location,
                        "couldn't use assign in expr.",
                        "check your code.",
                    ))
                }
                _ => {}
            }
        }
        // устанавливаем should push
        left = set_should_push(left, is_expr)?;
        // возвращаем
        Ok(left)
    }

    // парсинг error propagation
    fn error_propagation(&mut self, is_expr: bool) -> Result<Node, Error> {
        // парсинг access
        let mut node = self.access(is_expr)?;
        // проверяем на вопросик
        if self.check(TokenKind::Question) {
            // вопросительный знак
            let question = self.consume(TokenKind::Question)?.clone();
            // нода
            node = Node::ErrorPropagation {
                location: question,
                value: Box::new(node),
                should_push: is_expr,
            }
        }
        // возвращаем
        Ok(node)
    }
    
    // access выражение
    fn access_expr(&mut self) -> Result<Node, Error> {
        if self.check(TokenKind::New) { self.access(true) }
        else { self.error_propagation(true) }
    }

    // access стейтмент
    fn access_stmt(&mut self) -> Result<Node, Error> {
        if self.check(TokenKind::New) { self.access(false) }
        else { self.error_propagation(false) }
    }

    // скобки
    fn grouping_expr(&mut self) -> Result<Node, Error> {
        // (
        self.consume(TokenKind::Lparen)?;
        // выражение
        let expr = self.expr()?;
        // )
        self.consume(TokenKind::Rparen)?;
        // возвращаем
        Ok(expr)
    }

    // анонимная функция
    fn anonymous_fn_expr(&mut self) -> Result<Node, Error> {
        // fun
        let location = self.consume(TokenKind::Fun)?.clone();
        // параметры
        let mut params: Vec<Token> = Vec::new();
        if self.check(TokenKind::Lparen) {
            params = self.params()?;
        }
        // тело
        self.consume(TokenKind::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenKind::Rbrace)?;
        // возвращаем
        Ok(Node::AnFnDeclaration {
            location,
            params,
            body: Box::new(body),
            make_closure: true
        })
    }

    // лямбда
    fn lambda_fn_expr(&mut self) -> Result<Node, Error> {
        // lambda
        let location = self.consume(TokenKind::Lambda)?.clone();
        // параметры
        let mut params: Vec<Token> = Vec::new();
        if self.check(TokenKind::Lparen) {
            params = self.params()?;
        }
        // ->
        self.consume(TokenKind::Arrow)?;
        // тело
        let body = self.expr()?;
        // возвращаем
        Ok(Node::AnFnDeclaration {
            location: location.clone(),
            params,
            body: Box::new(Node::Ret {
                location,
                value: Box::new(body)
            }),
            make_closure: true
        })
    }

    // primary
    fn primary_expr(&mut self) -> Result<Node, Error> {
        // матч
        match self.peek()?.tk_type {
            // access
            TokenKind::Id | TokenKind::New => {
                Ok(self.access_expr()?)
            }
            // число
            TokenKind::Number => {
                Ok(Node::Number {
                    value: self.consume(TokenKind::Number)?.clone()
                })
            }
            // текст
            TokenKind::Text => {
                Ok(Node::String {
                    value: self.consume(TokenKind::Text)?.clone()
                })
            }
            // бул
            TokenKind::Bool => {
                Ok(Node::Bool {
                    value: self.consume(TokenKind::Bool)?.clone()
                })
            }
            // в скобках
            TokenKind::Lparen => {
                Ok(self.grouping_expr()?)
            }
            // мапа
            TokenKind::Lbrace => {
                Ok(self.map_expr()?)
            }
            // список
            TokenKind::Lbracket => {
                Ok(self.list_expr()?)
            }
            // null
            TokenKind::Null => {
                Ok(Node::Null {
                    location: self.consume(TokenKind::Null)?.clone()
                })
            }
            // анонимная функция
            TokenKind::Fun => {
                Ok(self.anonymous_fn_expr()?)
            }
            // лямбда
            TokenKind::Lambda => {
                Ok(self.lambda_fn_expr()?)
            }
            // паттерн матчинг
            TokenKind::Match => {
                Ok(self.match_expr()?)
            }
            // иное
            _ => Err(Error::own_text(
                self.peek()?.address.clone(),
                format!("invalid token. {:?}:{:?}",
                    self.peek()?.tk_type, self.peek()?.value
                ),
                "check your code."
            ))
        }
    }

    // match выражение
    fn match_expr(&mut self) -> Result<Node, Error> {
        // локация
        let location = self.consume(TokenKind::Match)?.clone();
        // matchable значение
        let matchable = self.expr()?;
        // список кейсов
        let mut cases = vec![];
        // дефолтный кейс
        let default;
        // заворачивание в лямбду
        fn make_lambda(location: Token, body: Node) -> Node {
            Node::Block {
                body: vec![
                    Node::Define {
                        previous: None,
                        name: Token::new(
                            TokenKind::Id,
                            "@match_lambda".to_string(),
                            location.address.clone()
                        ),
                        value: Box::new(Node::AnFnDeclaration {
                            location: location.clone(),
                            params: vec![],
                            body: Box::new(body),
                            make_closure: false,
                        })
                    },
                    Node::Call {
                        previous: None,
                        name: Token::new(
                            TokenKind::Id,
                            "@match_lambda".to_string(),
                            location.address.clone()
                        ),
                        args: vec![],
                        should_push: true
                    }
                ]
            }
        }
        // {
        self.consume(TokenKind::Lbrace)?;
        // кейсы
        while self.check(TokenKind::Case) {
            // case
            self.consume(TokenKind::Case)?;
            // значение
            let value = self.expr()?;
            // если ->
            if self.check(TokenKind::Arrow) {
                // ->
                self.consume(TokenKind::Arrow)?;
                // добавляем кейс
                cases.push(MatchCase::new(
                    Box::new(value),
                    Box::new(self.expr()?),
                ));
            }
            // если тело в фигурных в скобках
            else if self.check(TokenKind::Lbrace) {
                // тело лямбды
                self.consume(TokenKind::Lbrace)?;
                let body = self.block()?;
                self.consume(TokenKind::Rbrace)?;
                // заворачиваем в лямбду
                cases.push(MatchCase::new(
                    Box::new(value),
                    Box::new(make_lambda(location.clone(), body))
                ));
            }
            // в ином случае
            else {
                return Err(Error::new(
                    location.address.clone(),
                    "expected arrow or brace after case value",
                    "check your code"
                ))
            }
        }
        // дефолтный кейс
        self.consume(TokenKind::Default)?;
        // если ->
        if self.check(TokenKind::Arrow) {
            // ->
            self.consume(TokenKind::Arrow)?;
            // дефолтный кейс
            default = Box::new(self.expr()?);
        }
        // если тело в фигурных в скобках
        else if self.check(TokenKind::Lbrace) {
            // тело лямбды
            self.consume(TokenKind::Lbrace)?;
            let body = self.block()?;
            self.consume(TokenKind::Rbrace)?;
            // заворачиваем в лямбду
            default = Box::new(make_lambda(location.clone(), body))
        }
        // в ином случае
        else {
            // ошибка
            return Err(Error::new(
                location.address.clone(),
                "expected arrow or brace after case value",
                "check your code"
            ))
        }
        // }
        self.consume(TokenKind::Rbrace)?;
        // возвращаем
        Ok(Node::Match {
            location,
            matchable: Box::new(matchable),
            cases,
            default
        })
    }

    // список
    fn list_expr(&mut self) -> Result<Node, Error> {
        // [
        let location = self.consume(TokenKind::Lbracket)?.clone();
        // парсинг тела
        // пустой список
        if self.check(TokenKind::Rbracket) {
            self.consume(TokenKind::Rbracket)?;
            Ok(
                Node::List {
                    location,
                    values: Vec::new()
                }
            )
        }
        // заполненный
        else {
            let mut nodes: Vec<Node> = vec![self.expr()?];

            while self.check(TokenKind::Comma) {
                self.consume(TokenKind::Comma)?;
                nodes.push(self.expr()?);
            }
            
            self.consume(TokenKind::Rbracket)?;
            
            Ok(Node::List {
                location,
                values: nodes
            })
        }
    }

    // key : value
    fn key_value_expr(&mut self) -> Result<(Node, Node), Error> {
        // ключ
        let l = self.expr()?;
        // :
        self.consume(TokenKind::Colon)?;
        // значение
        let r = self.expr()?;
        // возвращаем
        Ok((l, r))
    }

    // мапа
    fn map_expr(&mut self) -> Result<Node, Error> {
        // {
        let location = self.consume(TokenKind::Lbrace)?.clone();
        // парсинг тела
        // пустая мапа
        if self.check(TokenKind::Rbrace) {
            self.consume(TokenKind::Rbrace)?;
            Ok(
                Node::Map {
                    location,
                    values: Vec::new()
                }
            )
        }
        // заполненная
        else {
            let mut nodes: Vec<(Node, Node)> = Vec::new();
            let key = self.key_value_expr()?;
            nodes.push((key.0, key.1));
            while self.check(TokenKind::Comma) {
                self.consume(TokenKind::Comma)?;
                let key = self.key_value_expr()?;
                nodes.push((key.0, key.1));
            }
            self.consume(TokenKind::Rbrace)?;
            Ok(Node::Map {
                location,
                values: nodes
            })
        }
    }

    // унарная операция
    fn unary_expr(&mut self) -> Result<Node, Error> {
        let tk = self.peek()?;

        match tk {
            Token { tk_type, value, .. }
            if (tk_type == &TokenKind::Op && value == "-") || (tk_type == &TokenKind::Bang) => {
                let op = self.consume(*tk_type)?.clone();
                
                Ok(Node::Unary {
                    op,
                    value: Box::new(self.primary_expr()?)
                })
            }
            _ => {
                Ok(self.primary_expr()?)
            }
        }
    }

    // бинарные операции умножения, деления
    fn multiplicative_expr(&mut self) -> Result<Node, Error> {
        let mut left = self.unary_expr()?;

        while self.check(TokenKind::Op) && (
            self.peek()?.value == "*" || 
            self.peek()?.value == "&" || 
            self.peek()?.value == "|" || 
            self.peek()?.value == "^" || 
            self.peek()?.value == "/" || 
            self.peek()?.value == "%") 
        {
            let op = self.consume(TokenKind::Op)?.clone();
            let right = self.unary_expr()?;
            left = Node::Bin {
                left: Box::new(left),
                right: Box::new(right),
                op
            }
        }

        Ok(left)
    }

    // бинарные операции сложения, вычитания
    fn additive_expr(&mut self) -> Result<Node, Error> {
        let mut left = self.multiplicative_expr()?;

        while self.check(TokenKind::Op) &&
            (self.peek()?.value == "+" || self.peek()?.value == "-") {
            let op = self.consume(TokenKind::Op)?.clone();
            let right = self.multiplicative_expr()?;
            left = Node::Bin {
                left: Box::new(left),
                right: Box::new(right),
                op
            }
        }

        Ok(left)
    }

    // выражение range
    fn range_expr(&mut self) -> Result<Node, Error> {
        let mut left = self.additive_expr()?;
        
        if self.check(TokenKind::Range) {
            let location = self.consume(TokenKind::Range)?.clone();
            let right = self.additive_expr()?;
            left = Node::Range {
                location,
                from: Box::new(left),
                to: Box::new(right)
            }
        }
        
        Ok(left)
    }
    
    // impls
    fn impls_expr(&mut self) -> Result<Node, Error> {
        let mut left = self.range_expr()?;

        if self.check(TokenKind::Impls) {
            self.consume(TokenKind::Impls)?;
            let trait_name = self.consume(TokenKind::Id)?.clone();
            left = Node::Impls {
                value: Box::new(left),
                trait_name,
            }
        }

        Ok(left)
    }
    
    // сравнение >=, <=, ==, !=
    fn compare_expr(&mut self) -> Result<Node, Error> {
        let mut left = self.impls_expr()?;

        // <, >, <=, >=
        if self.check(TokenKind::Greater) || self.check(TokenKind::Less)
            || self.check(TokenKind::LessEq) || self.check(TokenKind::GreaterEq) {
            let op = self.peek()?.clone();
            self.current += 1;
            let right = self.impls_expr()?;
            left = Node::Cond {
                left: Box::new(left),
                right: Box::new(right),
                op
            };
        }

        Ok(left)
    }
    
    // проверка на равность ==, !=
    fn equality_expr(&mut self) -> Result<Node, Error> {
        let mut left = self.compare_expr()?;

        // ==, !=
        if self.check(TokenKind::Eq) || self.check(TokenKind::NotEq) {
            let op = self.peek()?.clone();
            self.current += 1;
            let right = self.compare_expr()?;
            left = Node::Cond {
                left: Box::new(left),
                right: Box::new(right),
                op
            };
        }

        Ok(left)
    }

    // логическое выражение
    fn logical_expr(&mut self) -> Result<Node, Error> {
        let mut left = self.equality_expr()?;

        while self.check(TokenKind::And) ||
            self.check(TokenKind::Or) {
            let op = self.peek()?.clone();

            self.current += 1;

            let right = self.equality_expr()?;

            left = Node::Logical {
                left: Box::new(left),
                right: Box::new(right),
                op
            };
        }

        Ok(left)
    }

    // выражение
    fn expr(&mut self) -> Result<Node, Error> {
        self.logical_expr()
    }

    // стейтмент continue
    fn continue_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenKind::Continue)?.clone();
        Ok(Node::Continue {
            location
        })
    }

    // стейтмент break
    fn break_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenKind::Break)?.clone();
        Ok(Node::Break {
            location
        })
    }

    // стейтмент return
    fn return_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenKind::Ret)?.clone();
        let value = Box::new(self.expr()?);
        Ok(Node::Ret {
            location,
            value
        })
    }

    // один import
    fn single_import(&mut self) -> Result<Import, Error> {
        let name = self.consume(TokenKind::Text)?.clone();
        if self.check(TokenKind::With) {
            self.consume(TokenKind::With)?;
            Ok(Import::new(
                Option::Some(name.address),
                name.value,
                Option::Some(
                    self.consume(TokenKind::Text)?.value.clone()
                )
            ))
        } else {
            Ok(Import::new(
                Option::Some(name.address),
                name.value,
                Option::None
            ))
        }
    }

    // стейтмент импорт
    fn import_stmt(&mut self) -> Result<Node, Error> {
        // локация
        let location = self.consume(TokenKind::Import)?.clone();
        // парсинг импортов
        let mut imports = Vec::new();
        if self.check(TokenKind::Lparen) {
            self.consume(TokenKind::Lparen)?;
            imports.push(self.single_import()?);
            while self.check(TokenKind::Comma) {
                self.consume(TokenKind::Comma)?;
                imports.push(self.single_import()?);
            }
        }
        else {
            imports.push(self.single_import()?);
        }
        // возвращаем
        Ok(Node::Import {
            location,
            imports
        })
    }

    // стейтмент while
    fn while_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenKind::While)?.clone();
        let logical = self.expr()?;
        self.consume(TokenKind::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenKind::Rbrace)?;
        Ok(Node::While {
            location,
            logical: Box::new(logical),
            body: Box::new(body)
        })
    }

    // else
    fn else_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenKind::Else)?.clone();
        self.consume(TokenKind::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenKind::Rbrace)?;
        Ok(Node::If {
            location: location.clone(),
            logical: Box::new(Node::Bool { value: Token::new(
                TokenKind::Bool,
                "true".to_string(),
                location.address
            )}),
            body: Box::new(body),
            elseif: None
        })
    }

    // elif
    fn elif_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenKind::Elif)?.clone();
        let logical = self.expr()?;
        self.consume(TokenKind::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenKind::Rbrace)?;
        if self.check(TokenKind::Elif) {
            Ok(Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: Some(Box::new(self.elif_stmt()?))
            })
        } else if self.check(TokenKind::Else) {
            Ok(Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: Some(Box::new(self.else_stmt()?))
            })
        } else {
            Ok(Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: None
            })
        }
    }

    // if
    fn if_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenKind::If)?.clone();
        let logical = self.expr()?;
        self.consume(TokenKind::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenKind::Rbrace)?;
        if self.check(TokenKind::Elif) {
            Ok(Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: Some(Box::new(self.elif_stmt()?))
            })
        } else if self.check(TokenKind::Else) {
            Ok(Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: Some(Box::new(self.else_stmt()?))
            })
        } else {
            Ok(Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: None
            })
        }
    }

    // стейтмент match
    fn match_stmt(&mut self) -> Result<Node, Error> {
        // локация
        let location = self.consume(TokenKind::Match)?.clone();
        // matchable значение
        let matchable = self.expr()?;
        // список кейсов
        let mut cases = vec![];
        // дефолтный кейс
        let default;
        // {
        self.consume(TokenKind::Lbrace)?;
        // кейсы
        while self.check(TokenKind::Case) {
            // case
            self.consume(TokenKind::Case)?;
            // значение
            let value = self.expr()?;
            // если ->
            if self.check(TokenKind::Arrow) {
                // ->
                self.consume(TokenKind::Arrow)?;
                // добавляем кейс
                cases.push(MatchCase::new(
                    Box::new(value),
                    Box::new(self.statement()?),
                ))
            }
            // если тело в фигурных скобках
            else if self.check(TokenKind::Lbrace) {
                // парсим тело
                self.consume(TokenKind::Lbrace)?;
                let body = self.block()?;
                self.consume(TokenKind::Rbrace)?;
                // добавляем кейс
                cases.push(MatchCase::new(
                    Box::new(value),
                    Box::new(body),
                ))
            }
            // в ином случае
            else {
                return Err(Error::new(
                    location.address.clone(),
                    "expected arrow or brace after case value",
                    "check your code"
                ))
            }
        }
        // дефолтный кейс
        self.consume(TokenKind::Default)?;
        // если ->
        if self.check(TokenKind::Arrow) {
            // ->
            self.consume(TokenKind::Arrow)?;
            // дефолтный кейс
            default = Box::new(self.statement()?);
        }
        // если тело в фигурных скобках
        else if self.check(TokenKind::Lbrace) {
            // парсим тело
            self.consume(TokenKind::Lbrace)?;
            let body = self.block()?;
            self.consume(TokenKind::Rbrace)?;
            // дефолтный кейс
            default = Box::new(body);
        }
        // в ином случае
        else {
            // ошибка
            return Err(Error::new(
                location.address.clone(),
                "expected arrow or brace after case value",
                "check your code"
            ))
        }
        // }
        self.consume(TokenKind::Rbrace)?;
        // возвращаем
        Ok(Node::Match {
            location,
            matchable: Box::new(matchable),
            cases,
            default
        })
    }

    // for
    fn for_stmt(&mut self) -> Result<Node, Error> {
        self.consume(TokenKind::For)?;
        let name = self.consume(TokenKind::Id)?.clone();
        self.consume(TokenKind::In)?;
        let value = self.expr()?;
        self.consume(TokenKind::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenKind::Rbrace)?;
        Ok(Node::For {
            variable_name: name,
            iterable: Box::new(value),
            body: Box::new(body),
        })
    }

    // определение функции
    fn function_stmt(&mut self) -> Result<Node, Error> {
        // fun
        self.consume(TokenKind::Fun)?;
        // имя
        let name = self.consume(TokenKind::Id)?.clone();
        // параметры
        let mut params: Vec<Token> = Vec::new();
        if self.check(TokenKind::Lparen) {
            params = self.params()?;
        }
        self.consume(TokenKind::Lbrace)?;
        // тело
        let body = self.block()?;
        self.consume(TokenKind::Rbrace)?;
        // возвращаем
        Ok(Node::FnDeclaration {
            name: name.clone(),
            full_name: Option::Some(
                self.to_full_name(name),
            ),
            params,
            body: Box::new(body),
            make_closure: true
        })
    }

    // определение типа
    fn type_stmt(&mut self) -> Result<Node, Error> {
        // type
        self.consume(TokenKind::Type)?;
        // имя
        let name = self.consume(TokenKind::Id)?.clone();
        // параметры
        let mut constructor: Vec<Token> = Vec::new();
        if self.check(TokenKind::Lparen) {
            constructor = self.params()?;
        }
        // имплементация трейтов
        let mut impls: Vec<Token> = Vec::new();
        if self.check(TokenKind::Impl) {
            // impl
            self.consume(TokenKind::Impl)?;
            // парсим
            if !self.check(TokenKind::Lbrace) {
                // параметр
                impls.push(self.consume(TokenKind::Id)?.clone());
                // через запятую
                while !self.is_at_end() && self.check(TokenKind::Comma) {
                    // ,
                    self.consume(TokenKind::Comma)?;
                    // параметр
                    impls.push(self.consume(TokenKind::Id)?.clone());
                }
            }
        }
        // тело
        self.consume(TokenKind::Lbrace)?;
        let mut body = Vec::new();
        while !self.is_at_end() && !self.check(TokenKind::Rbrace) {
            let location = self.peek()?.clone();
            let mut node = self.statement()?;
            match node {
                Node::FnDeclaration { name, params, body, .. } => {
                    node = Node::FnDeclaration {
                        name,
                        full_name: None,
                        params,
                        body,
                        make_closure: false
                    }
                }
                Node::Native { .. } |
                Node::Get { .. } |
                Node::Define { .. } |
                Node::Assign { .. } => {}
                _ => {
                    return Err(Error::own_text(
                        location.address,
                        format!("invalid node for type: {:?}:{:?}", location.tk_type, location.value),
                        "check your code.",
                    ));
                }
            }
            body.push(node);
        }
        self.consume(TokenKind::Rbrace)?;
        // возвращаем
        Ok(Node::Type {
            name: name.clone(),
            full_name: Some(self.to_full_name(name)),
            constructor,
            body: Box::new(Node::Block {
                body
            }),
            impls
        })
    }

    // определение трейта
    fn trait_stmt(&mut self) -> Result<Node, Error> {
        // trait
        self.consume(TokenKind::Trait)?;
        // имя
        let name = self.consume(TokenKind::Id)?.clone();
        // функции
        let mut functions: Vec<TraitNodeFn> = Vec::new();
        // тело
        self.consume(TokenKind::Lbrace)?;
        while !self.is_at_end() && !self.check(TokenKind::Rbrace) {
            // локация
            let location = self.peek()?.address.clone();
            // функция
            if self.check(TokenKind::Fun) {
                // fun
                self.consume(TokenKind::Fun)?;
                // имя функции
                let name = self.consume(TokenKind::Id)?.clone();
                // параметры
                let mut params: Vec<Token> = Vec::new();
                if self.check(TokenKind::Lparen) {
                    params = self.params()?;
                }
                // если есть тело
                if self.check(TokenKind::Lbrace) {
                    // тело
                    self.consume(TokenKind::Lbrace)?;
                    let body = self.block()?;
                    self.consume(TokenKind::Rbrace)?;
                    // добавляем
                    functions.push(TraitNodeFn::new(
                        name,
                        params,
                        Option::Some(Box::new(body))
                    ))
                }
                else {
                    // добавляем
                    functions.push(TraitNodeFn::new(
                        name,
                        params,
                        Option::None
                    ))
                }
            }
            // в ином случае
            else {
                error!(Error::new(
                    location,
                    "only fn-s can be declared in trait.",
                    "you can create this declaration: 'fn meow(cat)'",
                ))
            }
        }
        self.consume(TokenKind::Rbrace)?;
        // возвращаем
        Ok(Node::Trait {
            name: name.clone(),
            full_name: Some(self.to_full_name(name)),
            functions,
        })
    }

    // определение юнита
    fn unit_stmt(&mut self) -> Result<Node, Error> {
        // unit
        self.consume(TokenKind::Unit)?;
        // имя
        let name = self.consume(TokenKind::Id)?.clone();
        // тело
        self.consume(TokenKind::Lbrace)?;
        let mut body = Vec::new();
        while !self.is_at_end() && !self.check(TokenKind::Rbrace) {
            let location = self.peek()?.clone();
            let mut node = self.statement()?;
            match node {
                Node::FnDeclaration { name, params, body, .. } => {
                    node = Node::FnDeclaration {
                        name,
                        full_name: None,
                        params,
                        body,
                        make_closure: false
                    }
                }
                Node::Native { .. } |
                Node::Get { .. } |
                Node::Define { .. } |
                Node::Assign { .. } => {}
                _ => {
                    return Err(Error::own_text(
                        location.address,
                        format!("invalid node for unit: {:?}:{:?}", location.tk_type, location.value),
                        "check your code.",
                    ));
                }
            }
            body.push(node);
        }
        // }
        self.consume(TokenKind::Rbrace)?;
        // возвращаем
        Ok(Node::Unit {
            name: name.clone(),
            full_name: Some(self.to_full_name(name)),
            body: Box::new(Node::Block {
                body
            })
        })
    }

    // определение нативной функции
    fn native_stmt(&mut self) -> Result<Node, Error> {
        // native
        self.consume(TokenKind::Native)?;
        // имя
        let name = self.consume(TokenKind::Id)?.clone();
        // ->
        self.consume(TokenKind::Arrow)?;
        // имя нативной функции
        let fn_name = self.consume(TokenKind::Text)?.clone();
        // возвращаем
        Ok(Node::Native {
            name,
            fn_name
        })
    }

    // стейтмент
    fn statement(&mut self) -> Result<Node, Error> {
        let tk = self.peek()?;
        match tk.tk_type {
            TokenKind::Type => {
                self.type_stmt()
            },
            TokenKind::Unit => {
                self.unit_stmt()
            },
            TokenKind::If => {
                self.if_stmt()
            },
            TokenKind::New | TokenKind::Id => {
                self.access_stmt()
            },
            TokenKind::Match => {
                self.match_stmt()
            },
            TokenKind::Continue => {
                self.continue_stmt()
            },
            TokenKind::Break => {
                self.break_stmt()
            },
            TokenKind::Ret => {
                self.return_stmt()
            },
            TokenKind::Fun => {
                self.function_stmt()
            },
            TokenKind::Native => {
                self.native_stmt()
            },
            TokenKind::Import => {
                self.import_stmt()
            }
            TokenKind::For => {
                self.for_stmt()
            }
            TokenKind::While => {
                self.while_stmt()
            }
            TokenKind::Trait => {
                self.trait_stmt()
            }
            _ => {
                Err(Error::own_text(
                    tk.address.clone(),
                    format!("unexpected stmt token: {:?}:{}", tk.tk_type, tk.value),
                    "check your code.",
                ))
            }
        }
    }

    // парсинг
    pub fn parse(&mut self) -> Result<Node, Error> {
        self.block()
    }

    /*
     вспомогательные функции
     */

    // consume
    fn consume(&mut self, tk_type: TokenKind) -> Result<&Token, Error> {
        match self.tokens.get(self.current as usize) {
            Some(tk) => {
                self.current += 1;
                if tk.tk_type == tk_type {
                    Ok(tk)
                } else {
                    Err(Error::own_text(
                        tk.address.clone(),
                        format!("unexpected token: '{:?}:{}', expected: '{tk_type:?}'", tk.tk_type, tk.value),
                        "check your code."
                    ))
                }
            },
            None => {
                Err(Error::new(
                    Address::new(
                        0,
                        0,
                        self.file_path.clone()
                    ),
                    "unexpected eof",
                    "check your code."
                ))
            }
        }
    }

    // check
    fn check(&self, tk_type: TokenKind) -> bool {
        match self.tokens.get(self.current as usize) {
            Some(tk) => {
                if tk.tk_type == tk_type {
                    true
                } else {
                    false
                }
            },
            None => {
                false
            }
        }
    }

    // peek
    fn peek(&self) -> Result<&Token, Error> {
        match self.tokens.get(self.current as usize) {
            Some(tk) => {
                Ok(tk)
            },
            None => {
                Err(Error::new(
                    Address::new(
                        0,
                        0,
                        self.file_path.clone(),
                    ),
                    "unexpected eof",
                    "check your code."
                ))
            }
        }
    }

    // is_at_end
    fn is_at_end(&self) -> bool {
        self.current as usize >= self.tokens.len()
    }
}
