// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Simple hex dump routine.

use crate::bldb;
use crate::io::Read;
use crate::mem;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use crate::{print, println};
use alloc::vec::Vec;
use core::ptr;
use core::slice;

fn hexdump<T: Read + ?Sized>(mut addr: usize, src: &T) -> Result<()> {
    println!(
        "Dumping {s:#016x}..{e:#016x}",
        s = addr,
        e = addr.wrapping_add(src.size())
    );
    const PAD: &str = "";
    let mut len = src.size();
    let mut offset = 0;
    while len > 0 {
        let base = addr & !0b1111;
        let start = addr - base;
        let clen = usize::min(16 - start, len);

        print!("0x{base:016x}:");
        print!("{PAD:>pad$}", pad = start * 3);
        let mut b = 0u8;
        for k in 0..clen {
            let off = offset + k;
            src.read(off as u64, slice::from_mut(&mut b))?;
            print!(" {:02x}", b);
        }
        print!("{PAD:>pad$}", pad = (16 - (clen + start)) * 3);
        print!("{PAD:>start$}");
        print!(" [");
        for k in 0..clen {
            let off = offset + k;
            src.read(off as u64, slice::from_mut(&mut b))?;
            if b.is_ascii_graphic() || b == b' ' {
                print!("{b}", b = b as char);
            } else {
                print!(".");
            }
        }
        println!("]");
        addr = addr.wrapping_add(clen);
        offset = offset.wrapping_add(clen);
        len -= clen;
    }
    Ok(())
}

fn xdfile(config: &bldb::Config, path: &str) -> Result<()> {
    let fs = config.ramdisk.as_ref().ok_or(Error::FsNoRoot)?;
    let file = fs.open(path)?;
    hexdump(0, file.as_ref())
}

struct PtrLenPair(*const u8, usize);

impl Read for PtrLenPair {
    fn read(&self, offset: u64, dst: &mut [u8]) -> Result<usize> {
        use core::cmp;
        let &PtrLenPair(ptr, len) = self;
        let offset = offset.try_into().unwrap();
        if offset >= len {
            return Err(Error::Offset);
        }
        let ptr = ptr.wrapping_add(offset);
        let len = cmp::min(dst.len(), len - offset);
        unsafe {
            ptr::copy(ptr, dst.as_mut_ptr(), len);
        }
        Ok(len)
    }

    fn size(&self) -> usize {
        let &PtrLenPair(_, len) = self;
        len
    }
}

unsafe fn xdmem(config: &bldb::Config, arg: Value) -> Result<()> {
    let (ptr, len) =
        arg.as_ptr_len().and_then(|(ptr, len)| check_pair(config, ptr, len))?;
    let pair = PtrLenPair(ptr, len);
    let addr = ptr.addr();
    hexdump(addr, &pair)
}

pub fn xd(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: xd <addr>,<len>");
        error
    };
    let val = repl::popenv(env);
    if let Value::Str(path) = val {
        xdfile(config, &path)
    } else {
        unsafe { xdmem(config, val) }
    }
    .map_err(usage)?;
    Ok(Value::Nil)
}

fn check_pair_canon(ptr: *const u8, len: usize) -> Result<(*const u8, usize)> {
    let addr = ptr.addr();
    if !mem::is_canonical_range(addr, addr + len) {
        return Err(Error::PtrNonCanon);
    }
    Ok((ptr, len))
}

fn check_pair(
    config: &bldb::Config,
    ptr: *const u8,
    len: usize,
) -> Result<(*const u8, usize)> {
    check_pair_canon(ptr, len).and_then(|(ptr, len)| {
        let range = mem::page_range_raw(ptr.cast(), len);
        if config.page_table.is_region_readable(range) {
            Ok((ptr, len))
        } else {
            Err(Error::Unmapped)
        }
    })
}

fn check_pair_mut(
    config: &bldb::Config,
    ptr: *mut u8,
    len: usize,
) -> Result<(*mut u8, usize)> {
    check_pair_canon(ptr.cast_const(), len)
        .and_then(|(ptr, len)| {
            let range = mem::page_range_raw(ptr.cast(), len);
            if config.page_table.is_region_readable(range) {
                Ok((ptr, len))
            } else {
                Err(Error::Unmapped)
            }
        })
        .map(|(ptr, len)| (ptr.cast_mut(), len))
}

fn check_size(size: usize) -> bool {
    matches!(size, 1 | 2 | 4 | 8 | 16)
}

fn parse_peek_poke_pair(
    config: &bldb::Config,
    value: Value,
) -> Result<(*const u8, usize)> {
    value
        .as_ptr_len()
        .and_then(|(ptr, len)| check_pair(config, ptr, len))
        .and_then(|(ptr, len)| {
            if check_size(len) { Ok((ptr, len)) } else { Err(Error::BadArgs) }
        })
}

fn parse_peek_poke_pair_mut(
    config: &bldb::Config,
    value: Value,
) -> Result<(*mut u8, usize)> {
    value
        .as_ptr_len_mut()
        .and_then(|(ptr, len)| check_pair_mut(config, ptr, len))
        .and_then(|(ptr, len)| {
            if check_size(len) { Ok((ptr, len)) } else { Err(Error::BadArgs) }
        })
}

pub fn read(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: peek <addr>,<len>");
        error
    };
    let (ptr, len) =
        parse_peek_poke_pair(config, repl::popenv(env)).map_err(usage)?;
    let value = match len {
        1 => unsafe { ptr::read::<u8>(ptr).into() },
        2 => unsafe { ptr::read_unaligned::<u16>(ptr.cast()).into() },
        4 => unsafe { ptr::read_unaligned::<u32>(ptr.cast()).into() },
        8 => unsafe { ptr::read_unaligned::<u64>(ptr.cast()).into() },
        16 => unsafe { ptr::read_unaligned::<u128>(ptr.cast()) },
        _ => panic!("impossible length value"),
    };
    println!("{ptr:p} {value:#0pad$x}", pad = 2 * len);
    Ok(Value::Unsigned(value))
}

pub fn write(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: poke <addr>,<len> <value>");
        error
    };
    let (ptr, len) =
        parse_peek_poke_pair_mut(config, repl::popenv(env)).map_err(usage)?;
    let val = repl::popenv(env);
    match len {
        1 => unsafe {
            ptr::write(ptr, val.as_num::<u8>()?);
        },
        2 => unsafe {
            ptr::write_unaligned(ptr.cast(), val.as_num::<u16>()?);
        },
        4 => unsafe {
            ptr::write_unaligned(ptr.cast(), val.as_num::<u32>()?);
        },
        8 => unsafe {
            ptr::write_unaligned(ptr.cast(), val.as_num::<u64>()?);
        },
        16 => unsafe {
            ptr::write_unaligned(ptr.cast(), val.as_num::<u128>()?);
        },
        _ => panic!("impossible length value"),
    }
    Ok(Value::Nil)
}
