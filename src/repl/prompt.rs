// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::cons;
use crate::println;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use crate::uart;
use alloc::vec::Vec;
use core::time::Duration;

fn cycle(term: &mut uart::Uart, bs: &[u8], timeout: Duration) {
    cons::cycle(term, b"", bs, b"", timeout);
    term.getb();
}

pub(super) fn spinner(
    config: &mut bldb::Config,
    _env: &mut [Value],
) -> Result<Value> {
    cycle(&mut config.cons, b"|/-\\", Duration::from_millis(100));
    Ok(Value::Nil)
}

pub(super) fn pulser(
    config: &mut bldb::Config,
    _env: &mut [Value],
) -> Result<Value> {
    cycle(&mut config.cons, b"oOo.", Duration::from_millis(500));
    Ok(Value::Nil)
}

pub(super) fn mega_pulser(
    config: &mut bldb::Config,
    _env: &mut [Value],
) -> Result<Value> {
    cons::cycle(
        &mut config.cons,
        b"-->",
        b".oO0@0Oo",
        b"<--",
        Duration::from_millis(250),
    );
    Ok(Value::Nil)
}

pub(super) fn prompt(
    config: &mut bldb::Config,
    env: &mut Vec<Value>,
) -> Result<Value> {
    let usage = |error| {
        println!("usage: prompt <tenex | spinner | pulser>");
        error
    };
    let p = repl::popenv(env).as_string().map_err(usage)?;
    match p.as_str() {
        "tenex" => config.prompt = cons::Prompt::Tenex,
        "spinner" => config.prompt = cons::Prompt::Spinner,
        "pulser" => config.prompt = cons::Prompt::Pulser,
        _ => return Err(usage(Error::BadArgs)),
    }
    Ok(Value::Nil)
}
