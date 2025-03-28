// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::println;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use alloc::vec::Vec;

pub fn run(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    use crate::loader;
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
