// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::loader;
use crate::println;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use alloc::vec::Vec;

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
    let entry = entry.try_into().unwrap();
    Ok(Value::Pointer(src.as_ptr().with_addr(entry).cast_mut()))
}

pub fn run(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    use crate::ramdisk;
    let usage = |error| {
        println!("usage: load <path>");
        error
    };
    let path = repl::popenv(env).as_string().map_err(usage)?;
    let fs = config.ramdisk.as_ref().ok_or(Error::FsNoRoot)?;
    let kernel = ramdisk::open(fs, &path)?;
    let entry = loader::load(&mut config.page_table, &kernel)?;
    let entry = entry.try_into().unwrap();
    Ok(Value::Pointer(fs.data().with_addr(entry).cast_mut()))
}
