// imports
use crate::bytecode::OpcodeValue;
use crate::memory::gc::Gc;
use crate::natives::natives;
use crate::natives::utils;
use crate::table::Table;
use crate::vm::VM;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use md5::Md5;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};
use watt_common::address::Address;
use watt_common::{error, errors::Error};

/// Provides
#[allow(unused_variables)]
pub unsafe fn provide(built_in_address: &Address, vm: &mut VM) -> Result<(), Error> {
    // functions
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "crypto@b64_encode",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let to_encode = utils::expect_cloned_string(&addr, vm.pop(&addr));

            if should_push {
                vm.op_push(OpcodeValue::String(BASE64_STANDARD.encode(to_encode)))?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "crypto@b64_decode",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let to_decode = utils::expect_cloned_string(&addr, vm.pop(&addr));

            if should_push {
                match BASE64_STANDARD.decode(to_decode.clone()) {
                    Ok(decoded) => match String::from_utf8(decoded.clone()) {
                        Ok(decoded_string) => {
                            vm.op_push(OpcodeValue::String(decoded_string))?;
                        }
                        Err(e) => {
                            error!(Error::own(
                                addr.clone(),
                                format!("failed to decode b64 string, bytes: {decoded:?}"),
                                format!("error: {e:?}")
                            ))
                        }
                    },
                    Err(e) => {
                        error!(Error::own_hint(
                            addr.clone(),
                            "failed to decode b64 string",
                            format!("error: {e:?}")
                        ))
                    }
                }
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "crypto@sha256",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let to_crypto = utils::expect_cloned_string(&addr, vm.pop(&addr));

            if should_push {
                vm.op_push(OpcodeValue::String(hex::encode(Sha256::digest(to_crypto))))?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "crypto@sha224",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let to_crypto = utils::expect_cloned_string(&addr, vm.pop(&addr));

            if should_push {
                vm.op_push(OpcodeValue::String(hex::encode(Sha224::digest(to_crypto))))?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "crypto@sha512",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let to_crypto = utils::expect_cloned_string(&addr, vm.pop(&addr));

            if should_push {
                vm.op_push(OpcodeValue::String(hex::encode(Sha512::digest(to_crypto))))?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "crypto@sha384",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let to_crypto = utils::expect_cloned_string(&addr, vm.pop(&addr));

            if should_push {
                vm.op_push(OpcodeValue::String(hex::encode(Sha384::digest(to_crypto))))?;
            }

            Ok(())
        },
    );
    natives::provide(
        vm,
        built_in_address.clone(),
        1,
        "crypto@md5",
        |vm: &mut VM, addr: Address, should_push: bool, table: Gc<Table>| {
            let to_crypto = utils::expect_cloned_string(&addr, vm.pop(&addr));

            if should_push {
                vm.op_push(OpcodeValue::String(hex::encode(Md5::digest(to_crypto))),)?;
            }

            Ok(())
        },
    );
    Ok(())
}
