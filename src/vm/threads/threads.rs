// потоки
use std::thread::{JoinHandle};
use crate::lexer::address::Address;
use crate::vm::bytecode::Chunk;
use crate::vm::statics::statics;
use crate::vm::table::Table;
use crate::vm::threads::{gil, nonsafe};
use crate::vm::values::{Function, Value};

// поток
#[derive(Debug)]
pub struct VmThread {
    thread: Option<JoinHandle<()>>,
}
// имплементация
impl VmThread {
    // новый поток
    pub fn new() -> Self {
        Self {thread: None}
    }
    // запуск
    pub fn start(&mut self, addr: Address,
                 function: *mut Function, args: Box<Chunk>, table: *mut Table) {
        // нонсейфы
        let nonsafe_table = nonsafe::NonSend::new(table);
        let nonsafe_function = nonsafe::NonSend::new(function);
        let nonsafe_args = nonsafe::NonSend::new(args);
        // запуск потока
        self.thread = Some(std::thread::spawn(move || unsafe {
            let vm_ptr = statics::VM_PTR;
            let _ = (*vm_ptr.unwrap()).call(
                addr.clone(),
                (*nonsafe_function.get()).name.name.clone(),
                Value::Fn(nonsafe_function.get()),
                nonsafe_args.get(),
                nonsafe_table.get(),
                false
            );
            gil::with_gil(||{(*(*vm_ptr.unwrap()).threads).threads_amount -= 1})
        }));
    }
}

// управление потоками
#[derive(Debug)]
pub struct Threads {
    pub threads_amount: u16
}
// имплементация
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
        // цикл
        loop {
            // ожидание колличества потоков
            if self.threads_amount <= 0 {
                break;
            }
        }
    }
}