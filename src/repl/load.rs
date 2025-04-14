// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::loader;
use crate::println;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use alloc::vec::Vec;

pub fn loadcpio(
    config: &mut bldb::Config,
    env: &mut Vec<Value>,
) -> Result<Value> {
    let usage = |error| {
        println!("usage: loadcpio <src addr>,<len> <path>");
        error
    };
    let cpio = repl::popenv(env)
        .as_slice(&config.page_table, 0)
        .and_then(|o| o.ok_or(Error::BadArgs))
        .map_err(usage)?;
    let path = repl::popenv(env).as_string().map_err(usage)?;
    let src = cpio_reader::iter_files(cpio)
        .find(|entry| entry.name() == path)
        .ok_or(Error::CpioNoFile)?
        .file();
    let entry = loader::load_bytes(&mut config.page_table, src)?;
    Ok(Value::Pointer(entry.cast_mut()))
}

pub fn loadmem(
    config: &mut bldb::Config,
    env: &mut Vec<Value>,
) -> Result<Value> {
    let usage = |error| {
        println!("usage: loadmem <src addr>,<src len>");
        error
    };
    let src = repl::popenv(env)
        .as_slice(&config.page_table, 0)
        .and_then(|o| o.ok_or(Error::BadArgs))
        .map_err(usage)?;
    let entry = loader::load_bytes(&mut config.page_table, src)?;
    crate::println!("Loaded ELF object from memory: entry point {entry:p}");
    Ok(Value::Pointer(entry.cast_mut()))
}

pub fn run(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: load <path>");
        error
    };
    let path = repl::popenv(env).as_string().map_err(usage)?;
    let fs = config.ramdisk.as_ref().ok_or(Error::FsNoRoot)?;
    let kernel = fs.open(&path)?;
    let entry = loader::load_file(&mut config.page_table, kernel.as_ref())?;
    crate::println!("Loaded ELF file: entry point {entry:p}");
    Ok(Value::Pointer(entry.cast_mut()))
}
