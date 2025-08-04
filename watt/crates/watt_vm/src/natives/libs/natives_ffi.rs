// imports
use crate::bytecode::OpcodeValue;
use crate::memory::gc::Gc;
use crate::memory::memory;
use crate::natives::natives;
use crate::natives::utils;
use crate::table::Table;
use crate::values::Value;
use crate::vm::VM;
use libffi::middle::{Arg, Cif, CodePtr, Type};
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::ffi::c_void;
use watt_common::address::Address;
use watt_common::{error, errors::Error};

/// The FFIValue representation
pub union FFIValue {
    i8: i8,
    u8: u8,
    i16: i16,
    u16: u16,
    i32: i32,
    u32: u32,
    i64: i64,
    u64: u64,
    f32: f32,
    f64: f64,
    isize: isize,
    usize: usize,
    ptr: *const c_void,
    bool: bool,
}
/// Implementation of FFI value
impl FFIValue {
    /// Creates i8 FFIValue from Value
    pub fn i8(address: &Address, value: Value) -> Self {
        match value {
            Value::Int(i) => FFIValue { i8: i as i8 },
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not convert {value} to ffi type i8."),
                    "you can convert only i64 to i8."
                ))
            }
        }
    }
    /// Creates u8 FFIValue from Value
    pub fn u8(address: &Address, value: Value) -> Self {
        match value {
            Value::Int(i) => FFIValue { u8: i as u8 },
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not convert {value} to ffi type u8."),
                    "you can convert only i64 to u8."
                ))
            }
        }
    }
    /// Creates i16 FFIValue from Value
    pub fn i16(address: &Address, value: Value) -> Self {
        match value {
            Value::Int(i) => FFIValue { i16: i as i16 },
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not convert {value} to ffi type i16."),
                    "you can convert only i64 to i16."
                ))
            }
        }
    }
    /// Creates u16 FFIValue from Value
    pub fn u16(address: &Address, value: Value) -> Self {
        match value {
            Value::Int(i) => FFIValue { u16: i as u16 },
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not convert {value} to ffi type u16."),
                    "you can convert only i64 to u16."
                ))
            }
        }
    }
    /// Creates i32 FFIValue from Value
    pub fn i32(address: &Address, value: Value) -> Self {
        match value {
            Value::Int(i) => FFIValue { i32: i as i32 },
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not convert {value} to ffi type i32."),
                    "you can convert only i64 to i32."
                ))
            }
        }
    }
    /// Creates u32 FFIValue from Value
    pub fn u32(address: &Address, value: Value) -> Self {
        match value {
            Value::Int(i) => FFIValue { u32: i as u32 },
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not convert {value} to ffi type u32."),
                    "you can convert only i64 to u32."
                ))
            }
        }
    }
    /// Creates i64 FFIValue from Value
    pub fn i64(address: &Address, value: Value) -> Self {
        match value {
            Value::Int(i) => FFIValue { i64: i },
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not convert {value} to ffi type i64."),
                    "you can only use i64."
                ))
            }
        }
    }
    /// Creates u64 FFIValue from Value
    pub fn u64(address: &Address, value: Value) -> Self {
        match value {
            Value::Int(i) => FFIValue { u64: i as u64 },
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not convert {value} to ffi type u64."),
                    "you can convert only i64 to u64."
                ))
            }
        }
    }
    /// Creates isize FFIValue from Value
    pub fn isize(address: &Address, value: Value) -> Self {
        match value {
            Value::Int(i) => FFIValue { isize: i as isize },
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not convert {value} to ffi type isize."),
                    "you can convert only i64 to isize."
                ))
            }
        }
    }
    /// Creates usize FFIValue from Value
    pub fn usize(address: &Address, value: Value) -> Self {
        match value {
            Value::Int(i) => FFIValue { usize: i as usize },
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not convert {value} to ffi type usize."),
                    "you can convert only i64 to usize."
                ))
            }
        }
    }
    /// Creates f32 FFIValue from Value
    pub fn f32(address: &Address, value: Value) -> Self {
        match value {
            Value::Float(f) => FFIValue { f32: f as f32 },
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not convert {value} to ffi type f32."),
                    "you can convert only f64 to f32."
                ))
            }
        }
    }
    /// Creates f64 FFIValue from Value
    pub fn f64(address: &Address, value: Value) -> Self {
        match value {
            Value::Float(f) => FFIValue { f64: f },
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not convert {value} to ffi type f64."),
                    "you can only use f64."
                ))
            }
        }
    }
    /// Creates bool FFIValue from Value
    pub fn bool(address: &Address, value: Value) -> Self {
        match value {
            Value::Bool(b) => FFIValue { bool: b },
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not convert {value} to ffi type bool."),
                    "you can only use bool."
                ))
            }
        }
    }
    /// Creates ptr FFIValue from Value
    pub fn ptr(address: &Address, value: Value) -> Self {
        match value {
            Value::Any(a) => unsafe {
                FFIValue {
                    ptr: (*a.raw()) as *const c_void,
                }
            },
            Value::String(s) => FFIValue {
                ptr: s.raw() as *const c_void,
            },
            _ => {
                error!(Error::own_text(
                    address.clone(),
                    format!("could not convert {value} to ffi type ptr."),
                    "you can convert: Any, String."
                ))
            }
        }
    }
    /// As arguments
    unsafe fn as_arg(&self, t: &FFIType) -> Arg {
        match t {
            FFIType::I8 => Arg::new(&self.i8),
            FFIType::U8 => Arg::new(&self.u8),
            FFIType::I16 => Arg::new(&self.i16),
            FFIType::U16 => Arg::new(&self.u16),
            FFIType::I32 => Arg::new(&self.i32),
            FFIType::U32 => Arg::new(&self.u32),
            FFIType::I64 => Arg::new(&self.i64),
            FFIType::U64 => Arg::new(&self.u64),
            FFIType::F32 => Arg::new(&self.f32),
            FFIType::F64 => Arg::new(&self.f64),
            FFIType::Pointer => Arg::new(&self.ptr),
            FFIType::Isize => Arg::new(&self.isize),
            FFIType::Usize => Arg::new(&self.usize),
            FFIType::Bool => Arg::new(&self.bool),
            _ => unreachable!(),
        }
    }
}

/// The FFI Type representation
#[derive(Debug, Clone)]
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
    String,
    Bool,
}
/// Implementation of FFI type
impl FFIType {
    /// Creates FFI type from the type name
    pub fn from(address: &Address, type_name: &str) -> Self {
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
            "string" => FFIType::String,
            "bool" => FFIType::Bool,
            _ => error!(Error::own_text(
                address.clone(),
                format!("unknown type {type_name}"),
                "available: i8,u8,i16,u16,i32,i64,isize,usize,f32,f43,string,void,ptr,bool"
            )),
        }
    }

    /// Converts ffi type to `libffi::middle::Type`
    pub fn to_ffi_type(&self) -> Type {
        match self {
            FFIType::I8 => Type::i8(),
            FFIType::U8 | FFIType::Bool => Type::u8(),
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
            FFIType::String => Type::pointer(),
        }
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
        address: &Address,
        name: String,
        out: String,
        params: Vec<String>,
    ) -> Result<(), libloading::Error> {
        // loading fn
        let func: Symbol<*mut c_void> = self.lib.get(name.as_bytes())?;

        // params
        let params: Vec<FFIType> = params
            .iter()
            .map(|arg| FFIType::from(address, arg.as_str()))
            .collect();
        let out = FFIType::from(address, out.as_str());

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
    pub unsafe fn call_fn(&mut self, addr: Address, name: String, args: Gc<Vec<Value>>) -> Value {
        // loading fn
        let func = self
            .fns
            .get(&name)
            .ok_or_else(|| {
                error!(Error::own_text(
                    addr.clone(),
                    format!("foreign fn: {name} is not found"),
                    "check foreign fn existence.",
                ));
            })
            .unwrap();

        // checking arguments amount
        if func.sign.len() != (*args).len() {
            error!(Error::own(
                addr,
                format!("invalid args amount: {} to call: {}.", (*args).len(), name),
                format!("expected {} arguments.", func.sign.len()),
            ));
        }

        // converting arguments
        let mut ffi_args: Vec<FFIValue> = Vec::with_capacity(func.sign.len());
        for (index, param) in func.sign.iter().enumerate() {
            let arg = args[index].clone();
            match param {
                FFIType::I8 => ffi_args.push(FFIValue::i8(&addr, arg)),
                FFIType::U8 => ffi_args.push(FFIValue::u8(&addr, arg)),
                FFIType::I16 => ffi_args.push(FFIValue::i16(&addr, arg)),
                FFIType::U16 => ffi_args.push(FFIValue::u16(&addr, arg)),
                FFIType::I32 => ffi_args.push(FFIValue::i32(&addr, arg)),
                FFIType::U32 => ffi_args.push(FFIValue::u32(&addr, arg)),
                FFIType::I64 => ffi_args.push(FFIValue::i64(&addr, arg)),
                FFIType::U64 => ffi_args.push(FFIValue::u64(&addr, arg)),
                FFIType::F32 => ffi_args.push(FFIValue::f32(&addr, arg)),
                FFIType::F64 => ffi_args.push(FFIValue::f64(&addr, arg)),
                FFIType::Void => unreachable!(),
                FFIType::Pointer => ffi_args.push(FFIValue::ptr(&addr, arg)),
                FFIType::Isize => ffi_args.push(FFIValue::isize(&addr, arg)),
                FFIType::Usize => ffi_args.push(FFIValue::usize(&addr, arg)),
                FFIType::String => ffi_args.push(FFIValue::ptr(&addr, arg)),
                FFIType::Bool => ffi_args.push(FFIValue::bool(&addr, arg)),
            }
        }

        // calling arguments
        let call_args: Vec<Arg> = ffi_args
            .iter()
            .enumerate()
            .map(|(i, v)| v.as_arg(&func.sign[i]))
            .collect();

        // calling a fn
        match func.out {
            FFIType::I8 => {
                let result = func.cif.call::<i8>(func.ptr, &call_args);
                Value::Int(result as i64)
            }
            FFIType::U8 => {
                let result = func.cif.call::<u8>(func.ptr, &call_args);
                Value::Int(result as i64)
            }
            FFIType::I16 => {
                let result = func.cif.call::<i16>(func.ptr, &call_args);
                Value::Int(result as i64)
            }
            FFIType::U16 => {
                let result = func.cif.call::<u16>(func.ptr, &call_args);
                Value::Int(result as i64)
            }
            FFIType::I32 => {
                let result = func.cif.call::<i32>(func.ptr, &call_args);
                Value::Int(result as i64)
            }
            FFIType::U32 => {
                let result = func.cif.call::<u32>(func.ptr, &call_args);
                Value::Int(result as i64)
            }
            FFIType::I64 => {
                let result = func.cif.call::<i64>(func.ptr, &call_args);
                Value::Int(result)
            }
            FFIType::U64 => {
                let result = func.cif.call::<i64>(func.ptr, &call_args);
                Value::Int(result)
            }
            FFIType::F32 => {
                let result = func.cif.call::<f32>(func.ptr, &call_args);
                Value::Float(result as f64)
            }
            FFIType::F64 => {
                let result = func.cif.call::<f64>(func.ptr, &call_args);
                Value::Float(result)
            }
            FFIType::Void => {
                func.cif.call::<()>(func.ptr, &call_args);
                Value::Null
            }
            FFIType::Pointer => {
                let result = func.cif.call::<*mut c_void>(func.ptr, &call_args);
                let any_ptr: *mut dyn std::any::Any = result as *mut dyn std::any::Any;
                
                Value::Any(Gc::new(any_ptr))
            }
            FFIType::String => {
                let result = func.cif.call::<*mut c_void>(func.ptr, &call_args);
                
                Value::String(Gc::from_raw(result as *mut String))
            }
            FFIType::Isize => {
                let result = func.cif.call::<isize>(func.ptr, &call_args);
                Value::Int(result as i64)
            }
            FFIType::Usize => {
                let result = func.cif.call::<usize>(func.ptr, &call_args);
                Value::Int(result as i64)
            }
            FFIType::Bool => {
                let result = func.cif.call::<bool>(func.ptr, &call_args);
                Value::Bool(result)
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
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let name = utils::expect_cloned_string(&addr, vm.pop(&addr));
            let path = utils::expect_cloned_string(&addr, vm.pop(&addr));

            if should_push {
                let lib = Library::new(path);
                if let Err(e) = lib {
                    error!(Error::own_text(
                        addr,
                        format!("lib open error: {e:?}"),
                        "check your code"
                    ));
                }
                let unwrapped_lib = lib.unwrap();
                vm.op_push(OpcodeValue::Raw(Value::Any(Gc::new(memory::alloc_value(
                    FFILibrary::new(unwrapped_lib),
                )))))?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        4,
        "ffi@load_fn",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let out = utils::expect_string(&addr, vm.pop(&addr));
            let params = utils::expect_string_list(&addr, vm.pop(&addr));
            let name = utils::expect_string(&addr, vm.pop(&addr));
            let lib = utils::expect_any(&addr, vm.pop(&addr), None);

            if let Some(library) = (*lib).downcast_mut::<FFILibrary>() {
                if let Err(e) = library.load_fn(&addr, (*name).clone(), (*out).clone(), params) {
                    error!(Error::own_text(
                        addr,
                        format!("load fn error: {e:?}"),
                        "check your code"
                    ))
                }
            } else {
                error!(Error::own_text(
                    addr,
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
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let args = utils::expect_list(&addr, vm.pop(&addr));
            let name = utils::expect_string(&addr, vm.pop(&addr));
            let lib = utils::expect_any(&addr, vm.pop(&addr), None);

            if let Some(library) = (*lib).downcast_mut::<FFILibrary>() {
                let result = library.call_fn(addr, (*name).clone(), args);
                if should_push {
                    vm.op_push(OpcodeValue::Raw(result))?;
                }
            } else {
                error!(Error::own_text(
                    addr,
                    format!("not a library: {lib:?}"),
                    "check your code"
                ))
            }
            Ok(())
        },
    );
    Ok(())
}
