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
    let data = smn::read(smn::Index::Smn0, addr).map_err(usage)?;
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
        smn::write(smn::Index::Smn0, addr, value)?;
    }
    Ok(repl::Value::Nil)
}

pub(super) fn rdsmni(
    _config: &mut bldb::Config,
    env: &mut Vec<repl::Value>,
) -> Result<repl::Value> {
    let usage = |error| {
        println!("usage: rdsmni <index> <addr>");
        error
    };
    let index = repl::popenv(env)
        .as_num::<u8>()
        .and_then(smn::Index::try_from)
        .map_err(usage)?;
    let addr = repl::popenv(env).as_num::<u32>().map_err(usage)?;
    let data = smn::read(index, addr).map_err(usage)?;
    println!("{addr:#x} {data:#x}");
    Ok(repl::Value::Unsigned(data.into()))
}

pub(super) fn wrsmni(
    _config: &mut bldb::Config,
    env: &mut Vec<repl::Value>,
) -> Result<repl::Value> {
    let usage = |error| {
        println!("usage: wrsmni <index> <addr> <value>");
        error
    };
    let index = repl::popenv(env)
        .as_num::<u8>()
        .and_then(smn::Index::try_from)
        .map_err(usage)?;
    let addr = repl::popenv(env).as_num::<u32>().map_err(usage)?;
    let value = repl::popenv(env).as_num::<u32>().map_err(usage)?;
    unsafe {
        smn::write(index, addr, value)?;
    }
    Ok(repl::Value::Nil)
}
