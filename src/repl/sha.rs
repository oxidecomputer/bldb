// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::println;
use crate::ramdisk;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use alloc::vec::Vec;

pub fn mem(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    use sha2::{Digest, Sha256};
    let usage = |error| {
        println!("usage: sha256mem <addr>,<len>");
        error
    };
    let bs = repl::popenv(env)
        .as_slice(&config.page_table, 0)
        .and_then(|o| o.ok_or(Error::BadArgs))
        .map_err(usage)?;
    let mut sum = Sha256::new();
    sum.update(bs);
    let hash = sum.finalize();
    Ok(Value::Sha256(hash.into()))
}

pub fn run(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let path = match repl::popenv(env) {
        Value::Str(path) => path,
        _ => {
            println!("usage: sha256 file");
            return Err(Error::BadArgs);
        }
    };
    let fs = config.ramdisk.as_ref().ok_or(Error::FsNoRoot)?;
    let hash = ramdisk::sha256(fs.as_ref(), &path)?;
    Ok(Value::Sha256(hash))
}
