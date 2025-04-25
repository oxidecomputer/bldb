// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::pci;
use crate::println;
use crate::repl;
use crate::result::{Error, Result};
use alloc::vec::Vec;

fn parse_bdf(s: &str) -> Result<(pci::Bus, pci::Device, pci::Function)> {
    let mut it = s.split('/');
    let (Some(bus), Some(dev), Some(func), None) =
        (it.next(), it.next(), it.next(), it.next())
    else {
        return Err(Error::BadArgs);
    };
    let bus = pci::Bus(repl::reader::parse_num(bus)?);
    let dev =
        repl::reader::parse_num::<u8>(dev).and_then(pci::Device::try_from)?;
    let func = repl::reader::parse_num::<u8>(func)
        .and_then(pci::Function::try_from)?;
    Ok((bus, dev, func))
}

pub(super) fn read(
    _config: &mut bldb::Config,
    env: &mut Vec<repl::Value>,
) -> Result<repl::Value> {
    let usage = |error| {
        println!("usage: ecamrd b/d/f <offset>");
        error
    };
    let (bus, dev, func) = repl::popenv(env)
        .as_string()
        .and_then(|s| parse_bdf(&s))
        .map_err(usage)?;
    let offset = repl::popenv(env)
        .as_num::<u32>()
        .and_then(pci::ecam::Offset::try_from)
        .map_err(usage)?;
    let data = unsafe { pci::ecam::read::<u32>(bus, dev, func, offset) }
        .map_err(usage)?;
    println!(
        "{b}/{d}/{f} {offset:#x} {data:#x}",
        b = bus.0,
        d = dev as u8,
        f = func as u8,
        offset = offset.addr(),
    );
    Ok(repl::Value::Unsigned(data.into()))
}

pub(super) fn write(
    _config: &mut bldb::Config,
    env: &mut Vec<repl::Value>,
) -> Result<repl::Value> {
    let usage = |error| {
        println!("usage: ecamwr b/d/f <offset> <value>");
        error
    };
    let (bus, dev, func) = repl::popenv(env)
        .as_string()
        .and_then(|s| parse_bdf(&s))
        .map_err(usage)?;
    let offset = repl::popenv(env)
        .as_num::<u32>()
        .and_then(pci::ecam::Offset::try_from)
        .map_err(usage)?;
    let value = repl::popenv(env).as_num::<u32>().map_err(usage)?;
    unsafe {
        pci::ecam::write(bus, dev, func, offset, value)?;
    }
    Ok(repl::Value::Nil)
}
