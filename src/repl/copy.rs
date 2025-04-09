// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::println;
use crate::ramdisk;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use alloc::vec::Vec;

pub fn run(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: copy <file> <dst addr>,<dst len>");
        error
    };
    let path = repl::popenv(env).as_string().map_err(usage)?;
    let fs = config.ramdisk.as_ref().ok_or(Error::FsNoRoot)?;
    let dst = repl::popenv(env)
        .as_slice_mut(&config.page_table, 0)
        .and_then(|o| o.ok_or(Error::BadArgs))
        .map_err(usage)?;
    let len = ramdisk::copy(fs, &path, dst)?;
    Ok(Value::Slice(&dst[..len]))
}
