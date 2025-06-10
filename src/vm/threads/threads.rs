/*
Потоки
 */
use std::thread::{JoinHandle};
use crate::lexer::address::Address;
use crate::vm::bytecode::Chunk;
use crate::vm::table::Table;
use crate::vm::threads::{gil, nonsafe};
use crate::vm::values::{Function, Value};
use crate::VM_PTR;
/*
Поток
 */
#[derive(Debug)]
pub struct VmThread {
    thread: Option<JoinHandle<()>>,
}
impl VmThread {
    // новый поток
    pub fn new() -> Self {
        Self {thread: None}
    }
    // запуск
    pub fn start(&mut self, addr: Address,
                 function: *mut Function, args: Box<Chunk>, table: *mut Table) {
        // табличка
        let nonsafe_table = nonsafe::NonSend::new(table);
        let nonsafe_function = nonsafe::NonSend::new(function);
        let nonsafe_args = nonsafe::NonSend::new(args);
        // поток
        self.thread = Some(std::thread::spawn(move || unsafe {
            let _ = (*VM_PTR.unwrap()).call(
                addr.clone(),
                (*nonsafe_function.get()).name.name.clone(),
                Value::Fn(nonsafe_function.get()),
                nonsafe_args.get(),
                nonsafe_table.get(),
                false
            );
            gil::with_gil(||{(*(*VM_PTR.unwrap()).threads).threads_amount -= 1})
        }));
    }
}

/*
Код для работы с потоками
 */
#[derive(Debug)]
pub struct Threads {
    pub threads_amount: u16
}

impl Threads {
    // новые потоки
    pub fn new() -> Threads {
        Threads {
            threads_amount: 0
        }
    }

    // запуск потока
    pub unsafe fn run_thread(&mut self, addr: Address, function: *mut Function, table: *mut Table, args: Box<Chunk>) {
        gil::with_gil(||{self.threads_amount += 1});
        let mut thread = VmThread::new();
        thread.start(addr, function, args, table);
    }

    // ожидание
    pub unsafe fn wait_finish(&mut self) {
        while self.threads_amount > 0 {
        }
    }
}