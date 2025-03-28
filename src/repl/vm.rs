// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::mem;
use crate::mmu;
use crate::println;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use alloc::vec::Vec;

fn check_phys_addr(pair: (u64, usize)) -> Result<(u64, usize)> {
    let (pa, _len) = pair;
    if !mem::is_physical(pa) {
        return Err(Error::NumRange);
    }
    if (pa & mem::P4KA::MASK) != 0 {
        return Err(Error::PageAlign);
    }
    Ok(pair)
}

fn check_virt_range(va: *const (), len: usize) -> Result<*const ()> {
    let addr = va.addr();
    if (addr % mem::V4KA::SIZE) != 0 {
        return Err(Error::PageAlign);
    }
    if (len % mem::V4KA::SIZE) != 0 {
        return Err(Error::PageAlign);
    }
    if !mem::is_canonical_range(addr, addr + len) {
        return Err(Error::PtrNonCanon);
    }
    Ok(va)
}

fn parse_page_attrs(s: &str) -> Result<mem::Attrs> {
    let mut attrs = mem::Attrs::new_rw();
    for attr in s.split(',') {
        match attr {
            "-r" => attrs.set_r(false),
            "r" => attrs.set_r(true),
            "w" => attrs.set_w(true),
            "-w" => attrs.set_w(false),
            "x" => attrs.set_x(true),
            "-x" => attrs.set_x(false),
            "c" => attrs.set_c(true),
            "-c" => attrs.set_c(false),
            "g" => attrs.set_g(true),
            "-g" => attrs.set_g(false),
            _ => return Err(Error::BadArgs),
        }
    }
    Ok(attrs)
}

pub fn map(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: map <phys addr>,<len> <va> <attrs>");
        error
    };
    let (pa, len) =
        repl::popenv(env).as_pair().and_then(check_phys_addr).map_err(usage)?;
    let va = repl::popenv(env)
        .as_ptr::<()>()
        .and_then(|va| check_virt_range(va, len))
        .map_err(usage)?;
    let attrs = repl::popenv(env)
        .as_string()
        .and_then(|s| parse_page_attrs(&s))
        .map_err(usage)?;
    unsafe {
        config.page_table.map_region(
            mem::page_range_raw(va, len),
            attrs,
            mem::P4KA::new(pa),
        )?;
    }
    Ok(Value::Nil)
}

pub fn mapping(
    config: &mut bldb::Config,
    env: &mut Vec<Value>,
) -> Result<Value> {
    let usage = |error| {
        println!("usage: mapping <addr>");
        error
    };
    let ptr = repl::popenv(env).as_ptr::<()>().map_err(usage)?;
    let pte = config.page_table.lookup(ptr);
    let value = match pte {
        None => {
            println!("{ptr:p} is not mapped");
            Value::Nil
        }
        Some(mmu::Entry::Page1G(pte)) => {
            println!("{ptr:p} maps to 1GiB page {pte:#x?}");
            Value::Unsigned(pte.bits().into())
        }
        Some(mmu::Entry::Page2M(pte)) => {
            println!("{ptr:p} maps to 2MiB page {pte:#x?}");
            Value::Unsigned(pte.bits().into())
        }
        Some(mmu::Entry::Page4K(pte)) => {
            println!("{ptr:p} maps to 4KiB page {pte:#x?}");
            Value::Unsigned(pte.bits().into())
        }
    };
    Ok(value)
}

pub fn mappings(
    config: &mut bldb::Config,
    _env: &mut [Value],
) -> Result<Value> {
    config.page_table.dump();
    Ok(Value::Nil)
}

pub fn unmap(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: unmap <addr>,<len>");
        error
    };
    let slice = repl::popenv(env)
        .as_slice_mut(&config.page_table, mem::V4KA::SIZE)
        .and_then(|o| o.ok_or(Error::BadArgs))
        .map_err(usage)?;
    let len = slice.len();
    let ptr = check_virt_range(slice.as_ptr().cast(), len).map_err(usage)?;
    unsafe {
        config.page_table.unmap_range(mem::page_range_raw(ptr, len))?;
    }
    Ok(Value::Nil)
}
