// импорт
use crate::executor::executor;
use crate::parser::import::Import;
use std::collections::HashMap;
use std::path::PathBuf;
use crate::lexer::address::Address;
use crate::parser::ast::Node;

// ресолвер импортов
pub struct ImportsResolver {
    imported: Vec<String>,
    libraries: HashMap<String, String>,
    builtins: Vec<String>
}
// имплементация
#[allow(unused_qualifications)]
impl ImportsResolver {
    // новый
    pub fn new() -> ImportsResolver {
        ImportsResolver {
            imported: vec![],
            libraries: HashMap::from([
                ("std.io".to_string(), "./libs/std/std_io.wt".to_string()),
                ("std.gc".to_string(), "./libs/std/std_gc.wt".to_string()),
                ("std.errors".to_string(), "./libs/std/std_errors.wt".to_string()),
            ]),
            builtins: vec!["./libs/base.wt".to_string()],
        }
    }

    // импорт билт-инов
    pub fn import_builtins(&mut self) -> Vec<Node> {
        // ноды
        let mut nodes = vec![];
        // перебираем билт-ины
        for builtin in self.builtins.clone() {
            if !self.imported.contains(&builtin) {
                // нода
                let node_option = self.import(
                    None,
                    &Import::new(None, builtin.to_string(), None)
                );
                // импортируем
                if let Some(node) = node_option {
                    nodes.push(node);
                }
            }
        }
        // возвращаем
        nodes
    }

    // ресолвинг
    fn resolve(
        &mut self,
        addr: Option<Address>,
        import: &Import
    ) -> Node {
        // ищем импорт
        let file: &str = if self.libraries.contains_key(&import.name) {
            self.libraries.get(&import.name).unwrap()
        } else {
            &import.name
        };
        // путь
        let path = PathBuf::from(file);
        // чтение файла
        let code = executor::read_file(addr, &path);
        // имя файла
        let filename = path.file_name().unwrap().to_str().unwrap();
        // компиляция
        let tokens = executor::lex(
            filename,
            &code,
            false,
            false
        );
        let ast = executor::parse(
            filename,
            tokens.unwrap(),
            false,
            false,
            &import.full_name
        );
        let mut analyzed = executor::analyze(
            ast.unwrap()
        );
        // блок результата
        let result: Node;
        // проверяем блок
        if let Node::Block { body } = &mut analyzed {
            // новое тело
            let mut new_body: Vec<Node> = vec![];
            // добавляем в тело
            
            while let Some(node) = body.pop() {
                // перебираем
                match node {
                    Node::Native { .. } |
                    Node::FnDeclaration { .. } |
                    Node::Type { .. } |
                    Node::Unit { .. } |
                    Node::Trait { .. } => {
                        new_body.push(node);
                    }
                    _ => {}
                }
            }
            // результат
            result = Node::Block { body: new_body };
        }
        // в случае ошибки
        else {
            // ошибка
            panic!("parser returned non-block node as result. report to the developer.");
        }
        // возвращаем
        result
    }

    // импорт
    pub fn import(
        &mut self,
        addr: Option<Address>,
        import: &Import
    ) -> Option<Node> {
        // проверка на наличие импорта, если его нет
        if !self.imported.contains(&import.name) {
            // ресолвинг
            let node = self.resolve(addr, import);
            // импротируем
            self.imported.push(import.name.clone());
            // возвращаем
            Option::Some(node)
        }
        // ничего
        else {
            Option::None
        }
    }
}
