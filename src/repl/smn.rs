// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::println;
use crate::repl;
use crate::result::Result;
use crate::smn;
use alloc::vec::Vec;

pub(super) fn read(
    _config: &mut bldb::Config,
    env: &mut Vec<repl::Value>,
) -> Result<repl::Value> {
    let usage = |error| {
        println!("usage: rdsmn <addr>");
        error
    };
    let addr = repl::popenv(env).as_num::<u32>().map_err(usage)?;
    let data = smn::read(addr).map_err(usage)?;
    println!("{addr:#x} {data:#x}");
    Ok(repl::Value::Unsigned(data.into()))
}

pub(super) fn write(
    _config: &mut bldb::Config,
    env: &mut Vec<repl::Value>,
) -> Result<repl::Value> {
    let usage = |error| {
        println!("usage: wrsmn <addr> <value>");
        error
    };
    let addr = repl::popenv(env).as_num::<u32>().map_err(usage)?;
    let value = repl::popenv(env).as_num::<u32>().map_err(usage)?;
    unsafe {
        smn::write(addr, value)?;
    }
    Ok(repl::Value::Nil)
}
