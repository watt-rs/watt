use std::{collections::HashMap, path::PathBuf};

/// Данный тип описывает таблицу тестов, которые могут или не могут иметь данные для сравнения
/// Ключ: путь к тесту
/// Значение: Путь к данным для сравнения
pub type TestMap = HashMap<String, Option<String>>;

/// Результаты запуска теста
#[derive(Default)]
pub struct TesterResults {
	/// Количество запущенных тестов
    pub ran: usize,
    /// Количество успешных тестов
    pub ok: usize,
    /// Количество проваленных тестов
    pub fail: usize
}

/// Находит все тесты, опираясь на путь `on` и возвращает список путей данных тестов.
pub fn find_tests(on: &str) -> std::io::Result<Vec<PathBuf>> {
    let mut wt_files = std::fs::read_dir(on)?
        .map(|x| x.unwrap().path())
        .filter(|x| x.extension().map(|k| k.to_str().unwrap()) == Some("wt"))
        .collect::<Vec<PathBuf>>();
    
    wt_files.sort();

    Ok(wt_files)
}

/// Собирает список тестов и ищет их данные для сравнения
/// Если есть данные для теста, то значение становится `Some(<путь к данным>)`
pub fn build_verification_table(on: &str) -> std::io::Result<TestMap> {
    let tests = find_tests(on)?;
    let outputs_path = on.to_string() + "/output/";

    // Выделяем заранее буфер под мапу
    let mut table = HashMap::with_capacity(tests.len());

    for i in tests {
        let output_file: Option<String> = {
            let output_path = outputs_path.clone() + i.file_name().unwrap().to_str().unwrap() + ".stdout";
            
            // Если есть файл с данными, то добавляем его в список.
            if std::fs::exists(&output_path).unwrap_or(false) {
                Some(output_path)
            } else {
                None
            }
        };

        table.insert(i.to_str().unwrap().to_string(), output_file);
    }

    Ok(table)
}

/// Запускает все тесты из таблицы
pub fn run_tests(watt_path: &str, tests_table: &HashMap<String, Option<String>>) -> TesterResults {
    let mut stats = TesterResults::default();

    for (test_file, expected_content_file) in tests_table {
    	// Создаём команду для запуска Watt
        let mut command = std::process::Command::new(watt_path);
        // Добавляем аргумент пути файла для запуска
        let command = command.arg(test_file);

        match command.output() {
        	// Если программа была запущена...
            Ok(data) => {
            	// ...неуспешно, то это провал
                if !data.status.success() {
                    println!("[FAIL] {test_file}");
                    println!("{}", str::from_utf8(&data.stdout).unwrap());
                    
                    stats.fail += 1;
                    stats.ran += 1;

                    continue;
                }

				// Если нет ожидаемых данных, то это хорошо, поскольку программа была обработана нормально
                if expected_content_file.is_none() {
                    println!("[NO OUTPUT - OK] {test_file}");

                    stats.ok += 1;
                } else {
                    let verify_data = std::fs::read(expected_content_file.as_ref().unwrap()).unwrap();

                    if verify_data == data.stdout {
                    	// Если даннве совпали, то тест пройден.
                        println!("[OK] {test_file}");

                        stats.ok += 1;
                    } else {
                        // Ну а если нет, то печатаем отчет об ощибке.
                        println!("[FAIL] {test_file}");
                        println!("Expected:\n---");

                        print!("{}", str::from_utf8(&verify_data).unwrap());
                        println!("\n---");

                        println!("Got:\n---");

                        print!("{}", str::from_utf8(&data.stdout).unwrap());
                        println!("\n---");

                        stats.fail += 1;
                    }
                }
            },
            Err(err) => {
            	// Если Watt не удалось запустить, то это ошибка
            	// TODO: Возможно если Watt не удастся запустить то все тесты будут считаться проваленными, поэтому необходимо остановить процесс тестирования.
                println!("[ERR] {test_file}: {err:?}");

                stats.fail += 1;
            },
        };

        stats.ran += 1;
    }

    assert!(stats.ran == stats.ok + stats.fail);

    stats
}
