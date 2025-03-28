// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::iomux;
use crate::println;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use alloc::vec::Vec;

pub(super) fn get(
    config: &mut bldb::Config,
    env: &mut Vec<Value>,
) -> Result<Value> {
    match repl::popenv(env).as_num::<u8>() {
        Err(e) => {
            println!("usage: iomuxget <pin>");
            Err(e)
        }
        Ok(pin) => {
            let value = config.iomux.get_pin(pin);
            println!("pin {pin} is set to IO mux function {value:?}");
            Ok(Value::Unsigned(value as u128))
        }
    }
}

fn parse_func(value: Value) -> Result<iomux::PinFunction> {
    let s = value.as_string()?;
    match s.as_str() {
        "F0" | "f0" => Ok(iomux::PinFunction::F0),
        "F1" | "f1" => Ok(iomux::PinFunction::F1),
        "F2" | "f2" => Ok(iomux::PinFunction::F2),
        "F3" | "f3" => Ok(iomux::PinFunction::F3),
        _ => Err(Error::BadArgs),
    }
}

pub(super) fn set(
    config: &mut bldb::Config,
    env: &mut Vec<Value>,
) -> Result<Value> {
    let usage = |error| {
        println!("usage: iomuxset <pin> <function>");
        error
    };
    let pin = repl::popenv(env).as_num::<u8>().map_err(usage)?;
    let function = parse_func(repl::popenv(env)).map_err(usage)?;
    unsafe {
        config.iomux.set_pin(pin, function);
    }
    Ok(Value::Nil)
}
