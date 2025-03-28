// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::mem;
use crate::println;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;

pub type Thunk = unsafe extern "C" fn(
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    r8: u64,
    r9: u64,
) -> u64;

// Parses the rip from the top element of the environment stack.
// We try our best to validate it, ensuring that it is canonical
// and that at least two bytes at the given address lie within a
// mapped range.  However, without examining the target
// instruction, it's difficult to ensure that it is fully
// mapped; it is possible that the instruction we jump to is
// right up against a page boundary, and the instruction could
// span across that into an unmapped page.  We choose a region
// size of two because that is the length of the shortest `jmp`
// instruction.
fn parse_rip(config: &bldb::Config, value: Value) -> Result<u64> {
    let rip = value.as_num::<u64>()?;
    let urip = rip as usize;
    if !mem::is_canonical(urip) {
        return Err(Error::PtrNonCanon);
    }
    let range = mem::page_range_raw(core::ptr::without_provenance(urip), 2);
    if !config.page_table.is_region_mapped(range, mem::Attrs::new_x()) {
        return Err(Error::Unmapped);
    }
    Ok(rip)
}

fn callargs(config: &bldb::Config, env: &mut Vec<Value>) -> Result<Vec<u64>> {
    let rip = parse_rip(config, repl::popenv(env))?;
    let mut args = vec![rip];
    for _ in 0..6 {
        match repl::popenv(env) {
            Value::Nil => break,
            Value::Slice(slice) => {
                args.push(slice.as_ptr().addr() as u64);
                args.push(slice.len() as u64);
            }
            Value::Pair(a, b) => {
                args.push(a as u64);
                args.push(b as u64);
            }
            Value::Unsigned(a) => {
                args.push(u64::try_from(a).map_err(|_| Error::NumRange)?);
            }
            _ => return Err(Error::BadArgs),
        }
    }
    Ok(args)
}

pub fn run(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: call <rip> [up to six args]");
        error
    };
    let args = callargs(config, env).map_err(usage)?;
    let rip = args[0];
    let thunk = unsafe { core::mem::transmute::<u64, Thunk>(rip) };
    let rdi = if args.len() > 1 { args[1] } else { 0 };
    let rsi = if args.len() > 2 { args[2] } else { 0 };
    let rdx = if args.len() > 3 { args[3] } else { 0 };
    let rcx = if args.len() > 4 { args[4] } else { 0 };
    let r8 = if args.len() > 5 { args[5] } else { 0 };
    let r9 = if args.len() > 6 { args[6] } else { 0 };
    let rax = unsafe { thunk(rdi, rsi, rdx, rcx, r8, r9) };
    println!("call returned {rax:x}");
    Ok(Value::Unsigned(rax.into()))
}
