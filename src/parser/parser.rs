// импорты
use crate::lexer::address::*;
use crate::errors::errors::{Error};
use crate::parser::import::Import;
use crate::lexer::lexer::*;
use crate::parser::ast::*;
use crate::error;

// парсер
pub struct Parser<'filename> {
    tokens: Vec<Token>,
    current: u128,
    filename: &'filename str,
    full_name_prefix: String,
}
// имплементация
#[allow(unused_qualifications)]
impl<'filename> Parser<'filename> {
    // новый
    pub fn new(tokens: Vec<Token>, filename: &'filename str, full_name_prefix: String) -> Parser<'filename> {
        Parser { tokens, current: 0, filename, full_name_prefix }
    }
    
    // блок
    fn block(&mut self) -> Result<Node, Error> {
        // список
        let mut nodes: Vec<Node> = Vec::new();
        // до } или конца файла
        while !self.is_at_end() && !self.check(TokenType::Rbrace) {
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
        self.consume(TokenType::Lparen)?;
        // до )
        if !self.check(TokenType::Rparen) {
            // аргумент
            nodes.push(self.expr()?);
            // через запятую
            while !self.is_at_end() && self.check(TokenType::Comma) {
                // ,
                self.consume(TokenType::Comma)?;
                // аргумент
                nodes.push(self.expr()?);
            }
        }
        // )
        self.consume(TokenType::Rparen)?;
        // возвращаем
        Ok(nodes)
    }

    // параметры
    fn params(&mut self) -> Result<Vec<Token>, Error> {
        // список
        let mut nodes: Vec<Token> = Vec::new();
        // (
        self.consume(TokenType::Lparen)?;
        // до )
        if !self.check(TokenType::Rparen) {
            // параметр
            nodes.push(self.consume(TokenType::Id)?.clone());
            // через запятую
            while !self.is_at_end() && self.check(TokenType::Comma) {
                // ,
                self.consume(TokenType::Comma)?;
                // параметр
                nodes.push(self.consume(TokenType::Id)?.clone());
            }
        }
        // )
        self.consume(TokenType::Rparen)?;
        // возвращаем
        Ok(nodes)
    }

    // преобразовывает name в full name
    fn to_full_name(&self, tk: Token) -> Token{
        Token::new(
            TokenType::Text,
            format!("{}:{}", self.full_name_prefix, tk.value.clone()),
            tk.address.clone(),
        )
    }

    // парсинг new
    fn object_creation_expr(&mut self) -> Result<Node, Error> {
        // new
        self.consume(TokenType::New)?;
        // имя и аргументы
        let name = self.consume(TokenType::Id)?.clone();
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
        if self.check(TokenType::Id) {
            // id
            let identifier = self.consume(TokenType::Id)?.clone();
            // дефайн ':='
            if self.check(TokenType::Walrus) {
                self.consume(TokenType::Walrus)?;
                Ok(Node::Define {
                    previous,
                    name: identifier,
                    value: Box::new(self.expr()?),
                })
            }
            // присваивание '='
            else if self.check(TokenType::Assign) {
                self.consume(TokenType::Assign)?;
                Ok(Node::Assign {
                    previous,
                    name: identifier,
                    value: Box::new(self.expr()?),
                })
            }
            // +=, -=, *=, /=
            else if self.check(TokenType::AssignAdd) ||
                self.check(TokenType::AssignSub) ||
                self.check(TokenType::AssignMul) ||
                self.check(TokenType::AssignDiv) {
                // оператор и локация
                let op;
                let location;
                // парсим
                match self.peek()?.tk_type {
                    TokenType::AssignSub => {
                        location = self.consume(TokenType::AssignSub)?.clone();
                        op = "-".to_string();
                    }
                    TokenType::AssignMul => {
                        location = self.consume(TokenType::AssignMul)?.clone();
                        op = "*".to_string();
                    }
                    TokenType::AssignDiv => {
                        location = self.consume(TokenType::AssignDiv)?.clone();
                        op = "/".to_string();
                    }
                    TokenType::AssignAdd => {
                        location = self.consume(TokenType::AssignAdd)?.clone();
                        op = "+".to_string();
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
                    previous: previous.clone(),
                    name: identifier.clone(),
                    value: Box::new(Node::Bin {
                        left: Box::new(var),
                        right: Box::new(self.expr()?),
                        op: Token::new(
                            TokenType::Op,
                            op,
                            location.address,
                        )
                    }),
                });
            }
            // вызов функции
            else if self.check(TokenType::Lparen) {
                return Ok(Node::Call {
                    previous,
                    name: identifier.clone(),
                    args: self.args()?,
                    should_push: true
                });
            }
            // получение значения переменной
            else {
                return Ok(Node::Get {
                    previous,
                    name: identifier.clone(),
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
        while self.check(TokenType::Dot) {
            // .
            self.consume(TokenType::Dot)?;
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
                        "couldn't use define in expr.".to_string(),
                        "check your code.".to_string(),
                    ))
                }
                Node::Assign { .. } => {
                    return Err(Error::new(
                        location,
                        "couldn't use assign in expr.".to_string(),
                        "check your code.".to_string(),
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
        if self.check(TokenType::Question) {
            // вопросительный знак
            let question = self.consume(TokenType::Question)?.clone();
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
        if self.check(TokenType::New) { self.access(true) }
        else { self.error_propagation(true) }
    }

    // access стейтмент
    fn access_stmt(&mut self) -> Result<Node, Error> {
        if self.check(TokenType::New) { self.access(false) }
        else { self.error_propagation(false) }
    }

    // скобки
    fn grouping_expr(&mut self) -> Result<Node, Error> {
        // (
        self.consume(TokenType::Lparen)?;
        // выражение
        let expr = self.expr()?;
        // )
        self.consume(TokenType::Rparen)?;
        // возвращаем
        Ok(expr)
    }

    // анонимная функция
    fn anonymous_fn_expr(&mut self) -> Result<Node, Error> {
        // fun
        let location = self.consume(TokenType::Fun)?.clone();
        // параметры
        let mut params: Vec<Token> = Vec::new();
        if self.check(TokenType::Lparen) {
            params = self.params()?;
        }
        // тело
        self.consume(TokenType::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenType::Rbrace)?;
        // возвращаем
        Ok(Node::AnFnDeclaration {
            location,
            params,
            body: Box::new(body)
        })
    }

    // лямбда
    fn lambda_fn_expr(&mut self) -> Result<Node, Error> {
        // lambda
        let location = self.consume(TokenType::Lambda)?.clone();
        // параметры
        let mut params: Vec<Token> = Vec::new();
        if self.check(TokenType::Lparen) {
            params = self.params()?;
        }
        // ->
        self.consume(TokenType::Arrow)?;
        // тело
        let body = self.expr()?;
        // возвращаем
        Ok(Node::AnFnDeclaration {
            location,
            params,
            body: Box::new(body)
        })
    }

    // primary
    fn primary_expr(&mut self) -> Result<Node, Error> {
        // матч
        match self.peek()?.tk_type {
            // access
            TokenType::Id | TokenType::New => {
                Ok(self.access_expr()?)
            }
            // число
            TokenType::Number => {
                Ok(Node::Number {
                    value: self.consume(TokenType::Number)?.clone()
                })
            }
            // текст
            TokenType::Text => {
                Ok(Node::String {
                    value: self.consume(TokenType::Text)?.clone()
                })
            }
            // бул
            TokenType::Bool => {
                Ok(Node::Bool {
                    value: self.consume(TokenType::Bool)?.clone()
                })
            }
            // в скобках
            TokenType::Lparen => {
                Ok(self.grouping_expr()?)
            }
            // мапа
            TokenType::Lbrace => {
                Ok(self.map_expr()?)
            }
            // список
            TokenType::Lbracket => {
                Ok(self.list_expr()?)
            }
            // null
            TokenType::Null => {
                Ok(Node::Null {
                    location: self.consume(TokenType::Null)?.clone()
                })
            }
            // анонимная функция
            TokenType::Fun => {
                Ok(self.anonymous_fn_expr()?)
            }
            // лямбда
            TokenType::Lambda => {
                Ok(self.lambda_fn_expr()?)
            }
            // паттерн матчинг
            TokenType::Match => {
                 todo!()
            }
            // иное
            _ => Err(Error::new(
                self.peek()?.address.clone(),
                format!("invalid token. {:?}:{:?}",
                    self.peek()?.tk_type, self.peek()?.value
                ),
                "check your code.".to_string(),
            ))
        }
    }

    // список
    fn list_expr(&mut self) -> Result<Node, Error> {
        // [
        let location = self.consume(TokenType::Lbracket)?.clone();
        // парсинг тела
        // пустой список
        if self.check(TokenType::Rbracket) {
            self.consume(TokenType::Rbracket)?;
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

            while self.check(TokenType::Comma) {
                self.consume(TokenType::Comma)?;
                nodes.push(self.expr()?);
            }
            
            self.consume(TokenType::Rbracket)?;
            
            Ok(Node::List {
                location,
                values: nodes
            })
        }
    }

    // key : value
    fn key_value_expr(&mut self) -> Result<(Box<Node>, Box<Node>), Error> {
        // ключ
        let l = self.expr()?;
        // :
        self.consume(TokenType::Colon)?;
        // значение
        let r = self.expr()?;
        // возвращаем
        Ok((Box::new(l), Box::new(r)))
    }

    // мапа
    fn map_expr(&mut self) -> Result<Node, Error> {
        // {
        let location = self.consume(TokenType::Lbrace)?.clone();
        // парсинг тела
        // пустая мапа
        if self.check(TokenType::Rbrace) {
            self.consume(TokenType::Rbrace)?;
            Ok(
                Node::Map {
                    location,
                    values: Vec::new()
                }
            )
        }
        // заполненная
        else {
            let mut nodes: Vec<(Box<Node>, Box<Node>)> = Vec::new();
            let key = self.key_value_expr()?;
            nodes.push((key.0, key.1));
            while self.check(TokenType::Comma) {
                self.consume(TokenType::Comma)?;
                let key = self.key_value_expr()?;
                nodes.push((key.0, key.1));
            }
            self.consume(TokenType::Rbrace)?;
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
            Token { tk_type: TokenType::Op, value, .. } if value == "-" || value == "!"  => {
                let op = self.consume(TokenType::Op)?.clone();
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

        while self.check(TokenType::Op) &&
            (self.peek()?.value == "*" || self.peek()?.value == "/") {
            let op = self.consume(TokenType::Op)?.clone();
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

        while self.check(TokenType::Op) &&
            (self.peek()?.value == "+" || self.peek()?.value == "-") {
            let op = self.consume(TokenType::Op)?.clone();
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
        
        if self.check(TokenType::Range) {
            let location = self.consume(TokenType::Range)?.clone();
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

        if self.check(TokenType::Impls) {
            self.consume(TokenType::Impls)?;
            let trait_name = self.consume(TokenType::Id)?.clone();
            left = Node::Impls {
                value: Box::new(left),
                trait_name,
            }
        }

        Ok(left)
    }

    // условие
    fn conditional_expr(&mut self) -> Result<Node, Error> {
        let mut left = self.impls_expr()?;

        // <, >, <=, >=, ==, !=
        if self.check(TokenType::Greater) || self.check(TokenType::Less)
            || self.check(TokenType::LessEq) || self.check(TokenType::GreaterEq)
            || self.check(TokenType::Eq) || self.check(TokenType::NotEq) {
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

    // логическое выражение
    fn logical_expr(&mut self) -> Result<Node, Error> {
        let mut left = self.conditional_expr()?;

        while self.check(TokenType::And) ||
            self.check(TokenType::Or) {
            let op = self.peek()?.clone();

            self.current += 1;

            let right = self.conditional_expr()?;

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
        let location = self.consume(TokenType::Continue)?.clone();
        Ok(Node::Continue {
            location
        })
    }

    // стейтмент break
    fn break_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenType::Break)?.clone();
        Ok(Node::Break {
            location
        })
    }

    // стейтмент return
    fn return_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenType::Ret)?.clone();
        let value = Box::new(self.expr()?);
        Ok(Node::Ret {
            location,
            value
        })
    }

    // один import
    fn single_import(&mut self) -> Result<Import, Error> {
        let name = self.consume(TokenType::Text)?.clone();
        if self.check(TokenType::With) {
            self.consume(TokenType::With)?;
            Ok(Import::new(
                Option::Some(name.address),
                name.value,
                Option::Some(
                    self.consume(TokenType::Text)?.value.clone()
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
        let location = self.consume(TokenType::Import)?.clone();
        // парсинг импортов
        let mut imports = Vec::new();
        if self.check(TokenType::Lparen) {
            self.consume(TokenType::Lparen)?;
            imports.push(self.single_import()?);
            while self.check(TokenType::Comma) {
                self.consume(TokenType::Comma)?;
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
        let location = self.consume(TokenType::While)?.clone();
        let logical = self.expr()?;
        self.consume(TokenType::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenType::Rbrace)?;
        Ok(Node::While {
            location,
            logical: Box::new(logical),
            body: Box::new(body)
        })
    }

    // else
    fn else_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenType::Else)?.clone();
        self.consume(TokenType::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenType::Rbrace)?;
        Ok(Node::If {
            location: location.clone(),
            logical: Box::new(Node::Bool { value: Token::new(
                TokenType::Bool,
                "true".to_string(),
                location.address
            )}),
            body: Box::new(body),
            elseif: None
        })
    }

    // elif
    fn elif_stmt(&mut self) -> Result<Node, Error> {
        let location = self.consume(TokenType::Elif)?.clone();
        let logical = self.expr()?;
        self.consume(TokenType::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenType::Rbrace)?;
        if self.check(TokenType::Elif) {
            Ok(Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: Some(Box::new(self.elif_stmt()?))
            })
        } else if self.check(TokenType::Else) {
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
        let location = self.consume(TokenType::If)?.clone();
        let logical = self.expr()?;
        self.consume(TokenType::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenType::Rbrace)?;
        if self.check(TokenType::Elif) {
            Ok(Node::If {
                location,
                logical: Box::new(logical),
                body: Box::new(body),
                elseif: Some(Box::new(self.elif_stmt()?))
            })
        } else if self.check(TokenType::Else) {
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

    // for
    fn for_stmt(&mut self) -> Result<Node, Error> {
        self.consume(TokenType::For)?;
        let name = self.consume(TokenType::Id)?.clone();
        self.consume(TokenType::In)?;
        let value = self.expr()?;
        self.consume(TokenType::Lbrace)?;
        let body = self.block()?;
        self.consume(TokenType::Rbrace)?;
        Ok(Node::For {
            variable_name: name,
            iterable: Box::new(value),
            body: Box::new(body),
        })
    }

    // определение функции
    fn function_stmt(&mut self) -> Result<Node, Error> {
        // fun
        self.consume(TokenType::Fun)?;
        // имя
        let name = self.consume(TokenType::Id)?.clone();
        // параметры
        let mut params: Vec<Token> = Vec::new();
        if self.check(TokenType::Lparen) {
            params = self.params()?;
        }
        self.consume(TokenType::Lbrace)?;
        // тело
        let body = self.block()?;
        self.consume(TokenType::Rbrace)?;
        // возвращаем
        Ok(Node::FnDeclaration {
            name: name.clone(),
            full_name: Option::Some(
                self.to_full_name(name),
            ),
            params,
            body: Box::new(body)
        })
    }

    // определение типа
    fn type_stmt(&mut self) -> Result<Node, Error> {
        // type
        self.consume(TokenType::Type)?;
        // имя
        let name = self.consume(TokenType::Id)?.clone();
        // параметры
        let mut constructor: Vec<Token> = Vec::new();
        if self.check(TokenType::Lparen) {
            constructor = self.params()?;
        }
        // имплементация трейтов
        let mut impls: Vec<Token> = Vec::new();
        if self.check(TokenType::Impl) {
            // impl
            self.consume(TokenType::Impl)?;
            // парсим
            if !self.check(TokenType::Lbrace) {
                // параметр
                impls.push(self.consume(TokenType::Id)?.clone());
                // через запятую
                while !self.is_at_end() && self.check(TokenType::Comma) {
                    // ,
                    self.consume(TokenType::Comma)?;
                    // параметр
                    impls.push(self.consume(TokenType::Id)?.clone());
                }
            }
        }
        // тело
        self.consume(TokenType::Lbrace)?;
        let mut body = Vec::new();
        while !self.is_at_end() && !self.check(TokenType::Rbrace) {
            let location = self.peek()?.clone();
            let mut node = self.statement()?;
            match node {
                Node::FnDeclaration { name, params, body, .. } => {
                    node = Node::FnDeclaration {
                        name,
                        full_name: None,
                        params,
                        body
                    }
                }
                Node::Native { .. } |
                Node::Get { .. } |
                Node::Define { .. } |
                Node::Assign { .. } => {}
                _ => {
                    return Err(Error::new(
                        location.address,
                        format!("invalid node for type: {:?}:{:?}", location.tk_type, location.value),
                        "check your code.".to_string(),
                    ));
                }
            }
            body.push(node);
        }
        self.consume(TokenType::Rbrace)?;
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

    // определение трэйта
    fn trait_stmt(&mut self) -> Result<Node, Error> {
        // trait
        self.consume(TokenType::Trait)?;
        // имя
        let name = self.consume(TokenType::Id)?.clone();
        // функции
        let mut functions: Vec<TraitNodeFn> = Vec::new();
        // тело
        self.consume(TokenType::Lbrace)?;
        while !self.is_at_end() && !self.check(TokenType::Rbrace) {
            // локация
            let location = self.peek()?.address.clone();
            // функция
            if self.check(TokenType::Fun) {
                // fun
                self.consume(TokenType::Fun)?;
                // имя функции
                let name = self.consume(TokenType::Id)?.clone();
                // параметры
                let mut params: Vec<Token> = Vec::new();
                if self.check(TokenType::Lparen) {
                    params = self.params()?;
                }
                // если есть тело
                if self.check(TokenType::Lbrace) {
                    // тело
                    self.consume(TokenType::Lbrace)?;
                    let body = self.block()?;
                    self.consume(TokenType::Rbrace)?;
                    // добавляем
                    functions.push(TraitNodeFn::new(
                        name,
                        params,
                        Option::Some(
                            Box::new(body)
                        )
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
                    "only fn-s can be declared in trait.".to_string(),
                    "you can create this declaration: 'fn meow(cat) {}'".to_string(),
                ))
            }
        }
        self.consume(TokenType::Rbrace)?;
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
        self.consume(TokenType::Unit)?;
        // имя
        let name = self.consume(TokenType::Id)?.clone();
        // тело
        self.consume(TokenType::Lbrace)?;
        let mut body = Vec::new();
        while !self.is_at_end() && !self.check(TokenType::Rbrace) {
            let location = self.peek()?.clone();
            let mut node = self.statement()?;
            match node {
                Node::FnDeclaration { name, params, body, .. } => {
                    node = Node::FnDeclaration {
                        name,
                        full_name: None,
                        params,
                        body
                    }
                }
                Node::Native { .. } |
                Node::Get { .. } |
                Node::Define { .. } |
                Node::Assign { .. } => {}
                _ => {
                    return Err(Error::new(
                        location.address,
                        format!("invalid node for unit: {:?}:{:?}", location.tk_type, location.value),
                        "check your code.".to_string(),
                    ));
                }
            }
            body.push(node);
        }
        // }
        self.consume(TokenType::Rbrace)?;
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
        self.consume(TokenType::Native)?;
        // имя
        let name = self.consume(TokenType::Id)?.clone();
        // ->
        self.consume(TokenType::Arrow)?;
        // имя нативной функции
        let fn_name = self.consume(TokenType::Text)?.clone();
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
            TokenType::Type => {
                self.type_stmt()
            },
            TokenType::Unit => {
                self.unit_stmt()
            },
            TokenType::If => {
                self.if_stmt()
            },
            TokenType::New | TokenType::Id => {
                self.access_stmt()
            },
            TokenType::Match => {
                todo!()
            },
            TokenType::Continue => {
                self.continue_stmt()
            },
            TokenType::Break => {
                self.break_stmt()
            },
            TokenType::Ret => {
                self.return_stmt()
            },
            TokenType::Fun => {
                self.function_stmt()
            },
            TokenType::Native => {
                self.native_stmt()
            },
            TokenType::Import => {
                self.import_stmt()
            }
            TokenType::For => {
                self.for_stmt()
            }
            TokenType::While => {
                self.while_stmt()
            }
            TokenType::Trait => {
                self.trait_stmt()
            }
            _ => {
                Err(Error::new(
                    tk.address.clone(),
                    format!("unexpected token: {:?}:{tk_value}",
                            tk.tk_type, tk_value=tk.value),
                    "check your code.".to_string(),
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
    fn consume(&mut self, tk_type: TokenType) -> Result<&Token, Error> {
        match self.tokens.get(self.current as usize) {
            Some(tk) => {
                self.current += 1;
                if tk.tk_type == tk_type {
                    Ok(tk)
                } else {
                    Err(Error::new(
                        tk.address.clone(),
                        format!("unexpected token: {:?}:{:?}", tk.tk_type, tk.value),
                        "check your code.".to_string()
                    ))
                }
            },
            None => {
                Err(Error::new(
                    Address::new(
                        0,
                        0,
                        self.filename.to_string(),
                        "eof".to_string()
                    ),
                    "unexpected eof".to_string(),
                    "check your code.".to_string()
                ))
            }
        }
    }

    // check
    fn check(&self, tk_type: TokenType) -> bool {
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
                        self.filename.to_string(),
                        "eof".to_string()
                    ),
                    "unexpected eof".to_string(),
                    "check your code.".to_string()
                ))
            }
        }
    }

    // is_at_end
    fn is_at_end(&self) -> bool {
        self.current as usize >= self.tokens.len()
    }
}
