// импорты
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::natives::natives;
use crate::vm::table::Table;
use crate::vm::values::{Value};
use crate::vm::vm::VM;

// провайд
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    // функция panic
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "base@panic",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            // текст hint
            let hint = vm.pop(&addr)?;
            // ошибка
            let error = vm.pop(&addr)?;
            // проверяем типы
            if let Value::String(hint_string) = hint {
                if let Value::String(error_string) = error {
                    // ошибка
                    error!(Error::own(
                        addr.clone(),
                        (*error_string).clone(),
                        (*hint_string).clone()
                    ));
                }
                else {
                    error!(Error::own_text(
                        addr.clone(),
                        "error text should be a string.".to_string(),
                        "check your code."
                    ))
                }
            }
            else {
                error!(Error::own_text(
                    addr.clone(),
                    "hint text should be a string.".to_string(),
                    "check your code."
                ))
            }
        }
    );
    // успех
    Ok(())
}