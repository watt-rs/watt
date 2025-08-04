/// imports
use crate::bytecode::OpcodeValue;
use crate::flow::ControlFlow;
use crate::memory::gc::Gc;
use crate::memory::memory;
use crate::natives::natives;
use crate::natives::utils;
use crate::table::Table;
use crate::values::Value;
use crate::vm::VM;
use watt_common::address::Address;
use watt_common::{error, errors::Error};

/// Gets request from stack
unsafe fn pop_request(vm: &mut VM, addr: &Address) -> Result<minreq::Request, ControlFlow> {
    // getting a raw request
    let raw_request = utils::expect_any(addr, vm.pop(addr), None);

    if !(*raw_request).is::<minreq::Request>() {
        error!(Error::new(
            addr.clone(),
            "internal builder in std.net.Request is not a `minreq::Request`!",
            "please file an issue at https://github.com/vyacheslavhere/watt"
        ));
    }

    Ok((*raw_request)
        .downcast_mut::<minreq::Request>()
        .unwrap()
        .clone())
}

/// Gets response from stack
unsafe fn pop_response(vm: &mut VM, addr: &Address) -> Result<minreq::Response, ControlFlow> {
    // getting a raw request
    let raw_request = utils::expect_any(addr, vm.pop(addr), None);

    if !(*raw_request).is::<minreq::Response>() {
        error!(Error::new(
            addr.clone(),
            "internal builder in std.net.Response is not a `minreq::Response`!",
            "please file an issue at https://github.com/vyacheslavhere/watt"
        ));
    }

    Ok((*raw_request)
        .downcast_mut::<minreq::Response>()
        .unwrap()
        .clone())
}

/// Provides
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "net@get",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let url = utils::expect_cloned_string(&addr, vm.pop(&addr));
            let requst = memory::alloc_value(minreq::get(url));

            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(requst))))?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "net@post",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let url = utils::expect_cloned_string(&addr, vm.pop(&addr));
            let requst = memory::alloc_value(minreq::post(url));

            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(requst))))?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "net@put",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let url = utils::expect_cloned_string(&addr, vm.pop(&addr));
            let requst = memory::alloc_value(minreq::put(url));

            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(requst))))?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "net@options",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let url = utils::expect_cloned_string(&addr, vm.pop(&addr));
            let requst = memory::alloc_value(minreq::options(url));

            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(requst))))?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "net@delete",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let url = utils::expect_cloned_string(&addr, vm.pop(&addr));
            let requst = memory::alloc_value(minreq::delete(url));

            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(requst))))?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "net@patch",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let url = utils::expect_cloned_string(&addr, vm.pop(&addr));
            let requst = memory::alloc_value(minreq::patch(url));

            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(requst))))?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "net@head",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let url = utils::expect_cloned_string(&addr, vm.pop(&addr));
            let requst = memory::alloc_value(minreq::head(url));

            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(requst))))?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        3,
        "net@header",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let value = utils::expect_cloned_string(&addr, vm.pop(&addr));
            let key = utils::expect_cloned_string(&addr, vm.pop(&addr));
            let cloned_request: minreq::Request = pop_request(vm, &addr)?;
            let request = memory::alloc_value(cloned_request.with_header(key, value));

            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(request))))?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        2,
        "net@body",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let data = utils::expect_cloned_string(&addr, vm.pop(&addr));
            let cloned_request: minreq::Request = pop_request(vm, &addr)?;
            let request = memory::alloc_value(cloned_request.with_body(data));

            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(request))))?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "net@send",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let request: minreq::Request = pop_request(vm, &addr)?;
            let result: Result<minreq::Response, minreq::Error> = request.send();

            if should_push {
                match result {
                    Ok(ok) => vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(
                        memory::alloc_value(ok),
                    ))))?,
                    Err(err) => error!(Error::own_text(
                        addr.clone(),
                        format!("failed the request: {err}"),
                        "check your request."
                    )),
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "net@response_status",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let response: minreq::Response = pop_response(vm, &addr)?;

            if should_push {
                vm.op_push(OpcodeValue::Int(response.status_code as i64))?
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "net@response_headers",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let response: minreq::Response = pop_response(vm, &addr)?;

            if should_push {
                let mut headers = String::from("{");

                for (key, value) in &response.headers {
                    headers.push_str(format!("\"{key}\":\"{value}\",",).as_str());
                }

                if !response.headers.is_empty() {
                    headers.pop();
                }

                headers.push('}');

                vm.op_push(OpcodeValue::String(headers))?
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "net@response_utf8",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let response: minreq::Response = pop_response(vm, &addr)?;

            if should_push {
                match response.as_str() {
                    Ok(ok) => vm.op_push(OpcodeValue::String(ok.to_string()))?,
                    Err(err) => error!(Error::own_text(
                        addr.clone(),
                        format!("failed response decoding as utf-8: {err}"),
                        "check your request."
                    )),
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "net@response_bytes",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let response: minreq::Response = pop_response(vm, &addr)?;

            if should_push {
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    response.as_bytes().to_vec(),
                )))))?;
            }

            Ok(())
        },
    );
    Ok(())
}
