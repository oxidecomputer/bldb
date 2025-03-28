// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::cpuid;
use crate::println;
use crate::repl;
use crate::result::Result;
use alloc::vec::Vec;

fn as_num(value: repl::Value) -> Result<u32> {
    match value {
        repl::Value::Nil => Ok(0),
        _ => value.as_num(),
    }
}

pub(super) fn run(
    _config: &bldb::Config,
    env: &mut Vec<repl::Value>,
) -> Result<repl::Value> {
    let usage = |error| {
        println!("usage: cpuid <leaf> [<subleaf>]");
        error
    };
    let leaf = repl::popenv(env).as_num().map_err(usage)?;
    let subleaf = as_num(repl::popenv(env)).map_err(usage)?;
    let res = cpuid::cpuid(leaf, subleaf);
    println!("{res:#?}");
    Ok(repl::Value::CpuIdResult(res))
}
