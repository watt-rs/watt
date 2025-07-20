// imports
use crate::error;
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::table::Table;
use crate::vm::values::Value;
use crate::vm::vm::VM;
use libffi::middle::{Arg, Cif, CodePtr, Type};
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::ffi::c_void;
use crate::vm::bytecode::OpcodeValue;
use crate::vm::memory::memory;
use crate::vm::natives::libs::utils;
use crate::vm::natives::natives;

/// The FFIType representation
#[derive(Debug)]
enum FFIType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    Void,
    Pointer,
    Isize,
    Usize,
}
/// Implementation of FFI type
impl FFIType {
    /// Creates FFI type from the type name
    pub fn from(type_name: &str) -> Self {
        match type_name {
            "i8" => FFIType::I8,
            "u8" => FFIType::U8,
            "i16" => FFIType::I16,
            "u16" => FFIType::U16,
            "i32" => FFIType::I32,
            "u32" => FFIType::U32,
            "i64" => FFIType::I64,
            "u64" => FFIType::U64,
            "f32" => FFIType::F32,
            "f64" => FFIType::F64,
            "void" => FFIType::Void,
            "ptr" => FFIType::Pointer,
            "isize" => FFIType::Isize,
            "usize" => FFIType::Usize,
            _ => panic!("unknown type {type_name}"),
        }
    }

    /// Converts ffi type to `libffi::middle::Type`
    pub fn to_ffi_type(&self) -> Type {
        match self {
            FFIType::I8 => Type::i8(),
            FFIType::U8 => Type::i16(),
            FFIType::I16 => Type::i16(),
            FFIType::U16 => Type::u16(),
            FFIType::I32 => Type::i32(),
            FFIType::U32 => Type::u32(),
            FFIType::I64 => Type::i64(),
            FFIType::U64 => Type::u64(),
            FFIType::F32 => Type::f32(),
            FFIType::F64 => Type::f64(),
            FFIType::Void => Type::void(),
            FFIType::Pointer => Type::pointer(),
            FFIType::Isize => Type::isize(),
            FFIType::Usize => Type::usize(),
        }
    }
}

/// Converts value to argument
fn value_to_arg(addr: Address, param: &FFIType, value: &Value) -> Result<Arg, Error> {
    // error
    let error = Error::own_text(
        addr,
        format!("could not convert {value:?} to {param:?}"),
        "check for types.",
    );

    // match
    match param {
        FFIType::I8 => match value {
            Value::Int(i) => Ok(Arg::new(&(*i as i8))),
            _ => Err(error),
        },
        FFIType::U8 => match value {
            Value::Int(i) => Ok(Arg::new(&(*i as u8))),
            _ => Err(error),
        },
        FFIType::I16 => match value {
            Value::Int(i) => Ok(Arg::new(&(*i as i16))),
            _ => Err(error),
        },
        FFIType::U16 => match value {
            Value::Int(i) => Ok(Arg::new(&(*i as u8))),
            _ => Err(error),
        },
        FFIType::I32 => match value {
            Value::Int(i) => Ok(Arg::new(&(*i as i32))),
            _ => Err(error),
        },
        FFIType::U32 => match value {
            Value::Int(i) => Ok(Arg::new(&(*i as u32))),
            _ => Err(error),
        },
        FFIType::I64 => match value {
            Value::Int(i) => Ok(Arg::new(i)),
            _ => Err(error),
        },
        FFIType::U64 => match value {
            Value::Int(i) => Ok(Arg::new(&(*i as u64))),
            _ => Err(error),
        },
        FFIType::F32 => match value {
            Value::Float(f) => Ok(Arg::new(&(*f as f32))),
            _ => Err(error),
        },
        FFIType::F64 => match value {
            Value::Float(f) => Ok(Arg::new(f)),
            _ => Err(error),
        },
        FFIType::Void => match value {
            Value::Null => Ok(Arg::new(&())),
            _ => Err(error),
        },
        FFIType::Pointer => match value {
            Value::Type(t) => Ok(Arg::new(t)),
            Value::Fn(f) => Ok(Arg::new(f)),
            Value::Native(n) => Ok(Arg::new(n)),
            Value::Instance(i) => Ok(Arg::new(i)),
            Value::Unit(u) => Ok(Arg::new(u)),
            Value::Trait(t) => Ok(Arg::new(t)),
            Value::List(l) => Ok(Arg::new(l)),
            Value::Any(a) => Ok(Arg::new(a)),
            _ => Err(error),
        },
        FFIType::Isize => match value {
            Value::Int(i) => Ok(Arg::new(&(*i as isize))),
            _ => Err(error),
        },
        FFIType::Usize => match value {
            Value::Int(i) => Ok(Arg::new(&(*i as usize))),
            _ => Err(error),
        },
    }
}

/// FFI function representation
///
/// `ptr`: ptr to the function
/// `cif`: cif of function call
/// `out`: ffi type of out
/// `sign`: function params
/// represented ad vector
/// of ffi types
///
#[derive(Debug)]
struct FFIFn {
    ptr: CodePtr,
    cif: Cif,
    out: FFIType,
    sign: Vec<FFIType>,
}
/// FFI function implementation
impl FFIFn {
    /// Creates new function
    pub fn new(ptr: CodePtr, cif: Cif, out: FFIType, sign: Vec<FFIType>) -> FFIFn {
        FFIFn {
            ptr,
            cif,
            out,
            sign,
        }
    }
}

/// FFI library implementation
struct FFILibrary {
    lib: Library,
    fns: HashMap<String, FFIFn>,
}
/// FFI library implementation
impl FFILibrary {
    /// Creates new library
    pub fn new(lib: Library) -> FFILibrary {
        FFILibrary {
            lib,
            fns: HashMap::new(),
        }
    }

    /// Loads function to cache
    pub unsafe fn load_fn(
        &mut self,
        name: String,
        out: String,
        params: Vec<String>,
    ) -> Result<(), libloading::Error> {
        // loading fn
        let func: Symbol<*mut c_void> = self.lib.get(name.as_bytes())?;

        // params
        let params: Vec<FFIType> = params
            .iter()
            .map(|arg| FFIType::from(arg.as_str()))
            .collect();
        let out = FFIType::from(out.as_str());

        // cif
        let cif_params = params
            .iter()
            .map(|e| e.to_ffi_type())
            .collect::<Vec<Type>>();
        let cif_out = out.to_ffi_type();
        let cif = Cif::new(cif_params, cif_out);

        // inserting fn
        self.fns
            .insert(name, FFIFn::new(CodePtr(*func), cif, out, params));

        Ok(())
    }

    /// Call function from cache
    pub unsafe fn call_fn(
        &mut self,
        vm: &mut VM,
        table: *mut Table,
        addr: Address,
        name: String,
        args: *mut Vec<Value>,
    ) -> Result<Value, Error> {

        // loading fn
        let func = self.fns.get(&name).ok_or_else(|| {
            Error::own_text(
                addr.clone(),
                format!("foreign fn: {name} is not found"),
                "check foreign fn existence.",
            )
        })?;

        // checking arguments amount
        if func.sign.len() != (*args).len() {
            return Err(Error::own(
                addr,
                format!("invalid args amount: {} to call: {}.", (*args).len(), name),
                format!("expected {} arguments.", func.sign.len()),
            ));
        }

        /// Converts arguments
        unsafe fn convert_args(
            addr: Address,
            params: &[FFIType],
            args: *mut Vec<Value>,
        ) -> Vec<Arg> {
            let mut converted: Vec<Arg> = vec![];

            for i in 0..params.len() {
                let param = params.get(i).unwrap();
                let arg = (*args).get(i).unwrap();

                let result = value_to_arg(addr.clone(), param, arg);

                match result {
                    Ok(ok) => {
                        converted.push(ok)
                    }
                    Err(err) => {
                        error!(err);
                    }
                }
            }

            converted
        }

        // calling a fn
        match func.out {
            FFIType::I8 => {
                let result = func
                    .cif
                    .call::<i8>(func.ptr, &convert_args(addr, &func.sign, args));
                Ok(Value::Int(result as i64))
            }
            FFIType::U8 => {
                let result = func
                    .cif
                    .call::<u8>(func.ptr, &convert_args(addr, &func.sign, args));
                Ok(Value::Int(result as i64))
            }
            FFIType::I16 => {
                let result = func
                    .cif
                    .call::<i16>(func.ptr, &convert_args(addr, &func.sign, args));
                Ok(Value::Int(result as i64))
            }
            FFIType::U16 => {
                let result = func
                    .cif
                    .call::<u16>(func.ptr, &convert_args(addr, &func.sign, args));
                Ok(Value::Int(result as i64))
            }
            FFIType::I32 => {
                let result = func
                    .cif
                    .call::<i32>(func.ptr, &convert_args(addr, &func.sign, args));
                Ok(Value::Int(result as i64))
            }
            FFIType::U32 => {
                let result = func
                    .cif
                    .call::<u32>(func.ptr, &convert_args(addr, &func.sign, args));
                Ok(Value::Int(result as i64))
            }
            FFIType::I64 => {
                let result = func
                    .cif
                    .call::<i64>(func.ptr, &convert_args(addr, &func.sign, args));
                Ok(Value::Int(result))
            }
            FFIType::U64 => {
                let result = func
                    .cif
                    .call::<i64>(func.ptr, &convert_args(addr, &func.sign, args));
                Ok(Value::Int(result))
            }
            FFIType::F32 => {
                let result = func
                    .cif
                    .call::<f32>(func.ptr, &convert_args(addr, &func.sign, args));
                Ok(Value::Float(result as f64))
            }
            FFIType::F64 => {
                let result = func
                    .cif
                    .call::<f64>(func.ptr, &convert_args(addr, &func.sign, args));
                Ok(Value::Float(result))
            }
            FFIType::Void => {
                func.cif
                    .call::<()>(func.ptr, &convert_args(addr, &func.sign, args));
                Ok(Value::Null)
            }
            FFIType::Pointer => {
                let result = func
                    .cif
                    .call::<*mut c_void>(func.ptr, &convert_args(addr, &func.sign, args));
                let value = Value::Any(result);
                vm.gc_guard(value);
                vm.gc_register(value, table);
                vm.gc_unguard();
                Ok(value)
            }
            FFIType::Isize => {
                let result = func
                    .cif
                    .call::<isize>(func.ptr, &convert_args(addr, &func.sign, args));
                Ok(Value::Int(result as i64))
            }
            FFIType::Usize => {
                let result = func
                    .cif
                    .call::<usize>(func.ptr, &convert_args(addr, &func.sign, args));
                Ok(Value::Int(result as i64))
            }
        }
    }
}

/// Provides
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "ffi@load",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let name = vm.pop(&addr)?;

            if should_push {
                match name {
                    Value::String(path) => {
                        let lib = Library::new((*path).clone());
                        if let Err(e) = lib {
                            error!(Error::own_text(
                                addr.clone(),
                                format!("lib open error: {e:?}"),
                                "check your code"
                            ));
                        }
                        let unwrapped_lib = lib.unwrap();
                        vm.op_push(
                            OpcodeValue::Raw(Value::Any(memory::alloc_value(
                                FFILibrary::new(unwrapped_lib)
                            ))),
                            table,
                        )?;
                    }
                    _ => error!(Error::own_text(
                        addr.clone(),
                        format!("not a lib path: {name:?}"),
                        "check your code"
                    )),
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        4,
        "ffi@load_fn",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let out = utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            let params = utils::expect_string_list(addr.clone(), vm.pop(&addr)?, None);
            let name = utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            let lib = utils::expect_any(addr.clone(), vm.pop(&addr)?, None);

            if let Some(library) = (*lib).downcast_mut::<FFILibrary>() {
                if let Err(e) = library.load_fn((*name).clone(), (*out).clone(), params) {
                    error!(Error::own_text(
                        addr.clone(),
                        format!("load fn error: {e:?}"),
                        "check your code"
                    ))
                }
            } else {
                error!(Error::own_text(
                    addr.clone(),
                    format!("not a library: {lib:?}"),
                    "check your code"
                ))
            }

            if should_push {
                vm.push(Value::Null);
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        3,
        "ffi@call_fn",
        |vm: &mut VM, addr: Address, should_push: bool, table: *mut Table| {
            let args = utils::expect_list(addr.clone(), vm.pop(&addr)?, None);
            let name = utils::expect_string(addr.clone(), vm.pop(&addr)?, None);
            let lib = utils::expect_any(addr.clone(), vm.pop(&addr)?, None);

            if let Some(library) = (*lib).downcast_mut::<FFILibrary>() {
                let called = library.call_fn(vm, table, addr.clone(), (*name).clone(), args);
                match called {
                    Ok(ok) => {
                        if should_push {
                            vm.op_push(OpcodeValue::Raw(ok), table)?;
                        }
                    }
                    Err(err) => {
                        error!(Error::own_text(
                            addr.clone(),
                            format!("call fn error: {err:?}"),
                            "check your code"
                        ))
                    }
                }

            } else {
                error!(Error::own_text(
                    addr.clone(),
                    format!("not a library: {lib:?}"),
                    "check your code"
                ))
            }
            Ok(())
        },
    );
    Ok(())
}
