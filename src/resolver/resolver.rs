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
            ]),
            builtins: vec!["./libs/base.wt".to_string()],
        }
    }

    // билт-ины
    pub fn provide_builtins(&mut self) {
        for builtin in self.builtins.clone() {
            if !self.imported.contains(&builtin.clone()) {
                self.import(None, Import::new(None, builtin, None));
            }
        }
    }

    // ресолвинг
    fn resolve(
        &mut self,
        addr: Option<Address>,
        import: Import
    ) -> Node {
        // файл
        let file;
        // ищем импорт
        if self.libraries.contains_key(&import.name) {
            file = self.libraries.get(&import.name).unwrap().clone();
        } else {
            file = import.name.clone();
        }
        // путь
        let path = PathBuf::from(file.clone());
        // чтение файла
        let code = executor::read_file(addr, path.clone());
        // имя файла
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();
        // компиляция
        let tokens = executor::lex(
            filename.clone(),
            code,
            false,
            false
        );
        let ast = executor::parse(
            filename,
            tokens.unwrap(),
            false,
            false,
            import.full_name
        );
        let analyzed = executor::analyze(
            ast.unwrap()
        );
        // блок результата
        let result: Node;
        // проверяем блок
        if let Node::Block { body } = analyzed {
            // новое тело
            let mut new_body: Vec<Box<Node>> = vec![];
            // добавляем в тело
            for node in body {
                // перебираем
                match *node.clone() {
                    Node::Native { .. } |
                    Node::FnDeclaration { .. } |
                    Node::Type { .. } |
                    Node::Unit { .. } => {
                        new_body.push(node.clone());
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
        import: Import
    ) -> Option<Node> {
        // проверка на наличие импорта, если его нет
        if !self.imported.contains(&import.name.clone()) {
            // ресолвинг
            let node = self.resolve(addr, import.clone());
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