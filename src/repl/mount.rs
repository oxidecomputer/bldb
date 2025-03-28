// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::println;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use alloc::vec::Vec;

pub fn run(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: mount <ramdisk addr>,<ramdisk len>");
        error
    };
    let val = repl::popenv(env);
    let ramdisk = val
        .as_slice(&config.page_table, 0)
        .and_then(|o| o.ok_or(Error::BadArgs))
        .map_err(usage)?;
    config.mount(ramdisk)?;
    Ok(Value::Nil)
}
