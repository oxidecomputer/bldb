// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::println;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use alloc::vec::Vec;
use core::convert::TryFrom;

fn msr_from_str(name: &str) -> Option<u32> {
    const MAP: &[(&str, u32)] = &[
        ("IA32_APIC_BASE", x86::msr::IA32_APIC_BASE),
        ("IA32_EFER", x86::msr::IA32_EFER),
        ("IA32_STAR", x86::msr::IA32_STAR),
        ("IA32_LSTAR", x86::msr::IA32_LSTAR),
        ("IA32_CSTAR", x86::msr::IA32_CSTAR),
        ("IA32_FMASK", x86::msr::IA32_FMASK),
        ("IA32_FS_BASE", x86::msr::IA32_FS_BASE),
        ("IA32_GS_BASE", x86::msr::IA32_GS_BASE),
        ("IA32_KERNEL_GSBASE", x86::msr::IA32_KERNEL_GSBASE),
    ];
    for &(mname, value) in MAP.iter() {
        if name == mname {
            return Some(value);
        }
    }
    None
}

fn value_to_msr(val: Value) -> Result<u32> {
    match val {
        Value::Str(name) => msr_from_str(&name).ok_or(Error::BadArgs),
        Value::Unsigned(num) => u32::try_from(num).map_err(|_| Error::NumRange),
        _ => Err(Error::BadArgs),
    }
}
pub fn write(
    _config: &mut bldb::Config,
    env: &mut Vec<Value>,
) -> Result<Value> {
    let usage = |error| {
        println!("usage: wrmsr <msr>, <value>");
        error
    };
    let msr = value_to_msr(repl::popenv(env)).map_err(usage)?;
    let value = repl::popenv(env).as_num().map_err(usage)?;
    unsafe {
        x86::msr::wrmsr(msr, value);
    }
    Ok(Value::Nil)
}

pub fn read(_config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: rdmsr <msr>");
        error
    };
    let msr = value_to_msr(repl::popenv(env)).map_err(usage)?;
    let val = unsafe { x86::msr::rdmsr(msr) };
    Ok(Value::Unsigned(val.into()))
}
