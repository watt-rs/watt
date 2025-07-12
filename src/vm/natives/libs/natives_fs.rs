// импорты
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::bytecode::OpcodeValue;
use crate::vm::memory::memory;
use crate::vm::natives::natives;
use crate::vm::table::Table;
use crate::vm::values::Value;
use crate::vm::vm::VM;
use crate::{error, vm::natives::libs::utils};
use std::io::{Read, Seek, Write};
use crate::vm::flow::ControlFlow;

// провайд
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    // функции
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@open",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // имя файла
            let filename = &*utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            // если надо пушить
            if should_push {
                // файл
                let file = match std::fs::OpenOptions::new().read(true).open(filename) {
                    Ok(file) => file,
                    Err(_) => {
                        vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
                        return Ok(());
                    }
                };
                // аллокация
                let file = memory::alloc_value(file);
                // пушим
                vm.op_push(OpcodeValue::Raw(Value::Any(file)), table)?;
            }
            // успех
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@create",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // имя файла
            let filename = &*utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            // если надо пушить
            if should_push {
                // файл
                let file = match std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&filename)
                {
                    Ok(file) => file,
                    Err(_e) => {
                        vm.op_push(OpcodeValue::Raw(Value::Null), table)?;

                        return Ok(());
                    }
                };
                // аллокация
                let file = memory::alloc_value(file);
                // добавляем
                vm.op_push(OpcodeValue::Raw(Value::Any(file)), table)?;
            }
            // успех
            Ok(())
        },
    );
    // pop файла из стека
    pub unsafe fn pop_file<'vm>(vm: &'vm mut VM, addr: &Address) -> Result<&'vm mut std::fs::File, ControlFlow> {
        // рав файл
        let raw_file = utils::expect_any(addr.clone(), vm.pop(&addr)?, None);
        // проверка, файл ли
        if !(*raw_file).is::<std::fs::File>() {
            error!(Error::new(
                addr.clone(),
                "internal type in std.fs.File is not a Rust's `std::io::File`!",
                "please file an issue at https://github.com/vyacheslavhere/watt"
            ));
        }
        // возвращаем
        Ok((*raw_file).downcast_mut().unwrap())
    }
    // провайд
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@read_to_string",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // получаем файл
            let file: &mut std::fs::File = pop_file(vm, &addr)?;
            // если надо пушить
            if should_push {
                // строка для чтения
                let mut string = String::new();
                file.read_to_string(&mut string).unwrap();
                // пушим
                vm.op_push(
                    OpcodeValue::Raw(Value::String(memory::alloc_value(string))),
                    table,
                )?;
            }
            // успех
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "fs@write",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // данные для записи
            let data = &*utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            // получаем файл
            let file: &mut std::fs::File = pop_file(vm, &addr)?;
            // если надо пушить
            if should_push {
                // запись
                let value = file.write(data.as_bytes());
                // успешна ли запись
                match value {
                    Ok(_) => { vm.op_push(OpcodeValue::Raw(Value::Null), table)?; }
                    Err(e) => { vm.op_push(
                        OpcodeValue::Int(e.raw_os_error().unwrap_or(0) as _),
                        table,
                    )?; }
                }
            }
            // успех
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@tell",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // получаем файл
            let file: &mut std::fs::File = pop_file(vm, &addr)?;
            // если надо пушить
            if should_push {
                let value = file.stream_position().unwrap_or(0);
                vm.op_push(OpcodeValue::Int(value as _), table)?;
            }
            // успех
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        3,
        "fs@seek",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // whence
            let whence = utils::expect_int(addr.clone(), vm.pop(&addr)?, None);
            // позиция
            let position = utils::expect_int(addr.clone(), vm.pop(&addr)?, None);
            // получаем файл
            let file: &mut std::fs::File = pop_file(vm, &addr)?;
            // seek
            file
                .seek(match whence {
                    1 => std::io::SeekFrom::Current(position as _),
                    2 => std::io::SeekFrom::End(position as _),
                    _ => std::io::SeekFrom::Start(position as _),
                })
                .unwrap();
            // если надо пушить
            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
            }
            // успех
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@mkdir",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // имя директории
            let name = &*utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            // создаём директорию
            let result = std::fs::create_dir(&name);
            // если надо пушить
            if should_push {
                // успешно ли создана директория
                if let Err(e) = result {
                    vm.op_push(
                        OpcodeValue::Int(e.raw_os_error().unwrap_or(0) as _),
                        table,
                    )?;
                } else {
                    vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
                }
            }
            // успех
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@delete_directory",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // имя директории
            let name = &*utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            // удаляем директорию
            let result = std::fs::remove_dir(name);
            // если надо пушить
            if should_push {
                // успешно ли удалена директория
                if let Err(e) = result {
                    vm.op_push(
                        OpcodeValue::Int(e.raw_os_error().unwrap_or(0) as _),
                        table,
                    )?;
                } else {
                    vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
                }
            }
            // успех
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@delete_directory_all",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // имя директории
            let name = &*utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            // удаляем древо директории
            let result = std::fs::remove_dir_all(name);
            // если надо пушить
            if should_push {
                // успешно ли удалено древо директории
                if let Err(e) = result {
                    vm.op_push(
                        OpcodeValue::Int(e.raw_os_error().unwrap_or(0) as _),
                        table,
                    )?;
                } else {
                    vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
                }
            }
            // успех
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@exists",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // путь
            let path = &*utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            // проверка на существование пути
            let result = std::fs::exists(path);
            // если надо пушить
            if should_push {
                if let Ok(data) = result {
                    vm.op_push(OpcodeValue::Raw(Value::Bool(data)), table)?;
                } else {
                    // NDRAEY todo: Change it when I learn to use typeof (return errno)
                    vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
                }
            }
            // успех
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@list",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // путь
            let path = &*utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            // если надо пушить
            if should_push {
                // читаем директорию
                let result = std::fs::read_dir(path);
                // создаем список
                match result {
                    Err(_) => {
                        // NDRAEY todo: Change it when I learn to use typeof (return errno)
                        vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
                    }
                    Ok(data) => {
                        // список
                        let paths: Vec<Value> = data
                            .filter(Result::is_ok)
                            .map(|x| x.unwrap().path().to_string_lossy().to_string())
                            .map(|st| Value::String(memory::alloc_value(st)))
                            .collect();
                        // аллокация
                        let allocated = Value::List(memory::alloc_value(paths));
                        // пушим
                        vm.op_push(OpcodeValue::Raw(allocated), table)?;
                    }
                }
            }
            // успех
            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "fs@is_directory",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // путь
            let path = &*utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            // если надо пушить
            if should_push {
                // читаем директорию
                let result = std::fs::metadata(path);
                // создаем список
                match result {
                    Err(_) => {
                        vm.op_push(OpcodeValue::Raw(Value::Null), table)?;
                    }
                    Ok(data) => {
                        // пушим
                        vm.op_push(OpcodeValue::Bool(data.is_dir()), table)?;
                    }
                }
            }
            // успех
            Ok(())
        }
    );
    // успех
    Ok(())
}
