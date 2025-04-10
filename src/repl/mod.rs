// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::mem;
use crate::mmu;
use crate::println;
use crate::result::{Error, Result};
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::fmt;
use core::ptr;
use core::slice;

mod bits;
mod call;
mod cat;
mod copy;
mod cpuid;
mod ecam;
mod elfinfo;
mod gpio;
mod inflate;
mod iomux;
mod jfmt;
mod list;
mod load;
mod memory;
mod mount;
mod msr;
mod pio;
mod reader;
mod rx;
mod rz;
mod sha;
mod smn;
mod vm;

#[derive(Clone)]
#[allow(dead_code)]
enum Value {
    Nil,
    Slice(&'static [u8]),
    Pair(usize, usize),
    Unsigned(u128),
    Pointer(*mut u8),
    Str(String),
    Cmd(String),
    Sha256([u8; 32]),
    CpuIdResult(x86::cpuid::CpuIdResult),
}

fn unsigned_to_ptr<F, T>(addr: F) -> Result<*const T>
where
    usize: TryFrom<F>,
{
    let addr = usize::try_from(addr).map_err(|_| Error::NumRange)?;
    if !mem::is_canonical(addr) {
        return Err(Error::PtrNonCanon);
    }
    Ok(ptr::with_exposed_provenance::<T>(addr))
}

fn unsigned_to_ptr_mut<F, T>(addr: F) -> Result<*mut T>
where
    usize: TryFrom<F>,
{
    let addr = usize::try_from(addr).map_err(|_| Error::NumRange)?;
    if !mem::is_canonical(addr) {
        return Err(Error::PtrNonCanon);
    }
    Ok(ptr::with_exposed_provenance_mut(addr))
}

impl Value {
    pub fn as_slice(
        &self,
        page_table: &mmu::LoaderPageTable,
        deflen: usize,
    ) -> Result<Option<&'static [u8]>> {
        let (ptr, len) = match self {
            Value::Nil => return Ok(None),
            Value::Slice(slice) => return Ok(Some(*slice)),
            Value::Pair(addr, len) => Ok((unsigned_to_ptr(*addr)?, *len)),
            Value::Unsigned(addr) => Ok((unsigned_to_ptr(*addr)?, deflen)),
            Value::Pointer(ptr) => Ok((ptr.cast_const(), deflen)),
            _ => Err(Error::BadArgs),
        }?;
        if page_table.is_region_readable(mem::page_range_raw(ptr.cast(), len)) {
            Ok(Some(unsafe { slice::from_raw_parts(ptr, len) }))
        } else {
            Err(Error::Unmapped)
        }
    }

    pub fn as_slice_mut(
        &self,
        page_table: &mmu::LoaderPageTable,
        deflen: usize,
    ) -> Result<Option<&'static mut [u8]>> {
        let (ptr, len) = match self {
            Value::Nil => return Ok(None),
            Value::Pair(addr, len) => Ok((unsigned_to_ptr_mut(*addr)?, *len)),
            Value::Unsigned(addr) => Ok((unsigned_to_ptr_mut(*addr)?, deflen)),
            Value::Pointer(ptr) => Ok((*ptr, deflen)),
            _ => Err(Error::BadArgs),
        }?;
        if page_table.is_region_writeable(mem::page_range_raw(ptr.cast(), len))
        {
            unsafe {
                ptr::write_bytes(ptr, 0, len);
            }
            Ok(Some(unsafe { slice::from_raw_parts_mut(ptr, len) }))
        } else {
            Err(Error::Unmapped)
        }
    }

    pub fn as_string(&self) -> Result<String> {
        match self {
            Value::Str(s) => Ok(s.clone()),
            _ => Err(Error::BadArgs),
        }
    }

    pub fn as_num<T: Default + TryFrom<u128>>(&self) -> Result<T> {
        match self {
            Value::Unsigned(num) => {
                T::try_from(*num).map_err(|_| Error::NumRange)
            }
            Value::Pointer(p) => {
                let addr = p.addr() as u128;
                T::try_from(addr).map_err(|_| Error::NumRange)
            }
            _ => Err(Error::BadArgs),
        }
    }

    fn as_ptr<T>(&self) -> Result<*const T> {
        match self {
            Value::Nil => Ok(ptr::null()),
            Value::Slice(slice) => Ok(slice.as_ptr().cast()),
            Value::Pair(addr, _len) => Ok(unsigned_to_ptr(*addr)?),
            Value::Unsigned(addr) => Ok(unsigned_to_ptr(*addr)?),
            Value::Pointer(ptr) => Ok(ptr.cast()),
            _ => Err(Error::BadArgs),
        }
    }

    fn as_pair(&self) -> Result<(u64, usize)> {
        match self {
            &Value::Pair(addr, len) => Ok((addr as u64, len)),
            _ => Err(Error::BadArgs),
        }
    }

    fn as_ptr_len(&self) -> Result<(*const u8, usize)> {
        match self {
            Value::Slice(slice) => Ok((slice.as_ptr(), slice.len())),
            &Value::Pair(addr, len) => Ok((unsigned_to_ptr(addr)?, len)),
            _ => Err(Error::BadArgs),
        }
    }

    fn as_ptr_len_mut(&self) -> Result<(*mut u8, usize)> {
        match self {
            &Value::Pair(addr, len) => Ok((unsigned_to_ptr_mut(addr)?, len)),
            _ => Err(Error::BadArgs),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Nil
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Value {
        Value::Nil
    }
}

impl fmt::Debug for Value {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> core::result::Result<(), fmt::Error> {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Slice(s) => write!(f, "{:#x?},{}", s.as_ptr(), s.len()),
            Self::Pair(a, b) => write!(f, "{:#x},{}", *a, *b),
            Self::Unsigned(u) => write!(f, "{:#x}", *u),
            Self::Pointer(p) => write!(f, "{:#x?}", *p),
            Self::Str(s) => write!(f, "{s}"),
            Self::Cmd(s) => write!(f, "[{s}]"),
            Self::Sha256(hash) => {
                for &b in hash.iter() {
                    write!(f, "{b:02x}")?;
                }
                Ok(())
            }
            Self::CpuIdResult(cpuid) => {
                write!(
                    f,
                    "[{:#x} {:#x} {:#x} {:#x}]",
                    cpuid.eax, cpuid.ebx, cpuid.ecx, cpuid.edx
                )
            }
        }
    }
}

fn evalcmd(
    config: &mut bldb::Config,
    cmd: &str,
    env: &mut Vec<Value>,
) -> Result<Value> {
    match cmd {
        "call" => call::run(config, env),
        "cat" => cat::run(config, env),
        "copy" => copy::run(config, env),
        "cpuid" => cpuid::run(config, env),
        "ecamrd" => ecam::read(config, env),
        "ecamwr" => ecam::write(config, env),
        "elfinfo" => elfinfo::run(config, env),
        "getbits" => bits::get(config, env),
        "gpioget" => gpio::get(config, env),
        "gpioset" => gpio::set(config, env),
        "hexdump" | "xd" => memory::xd(config, env),
        "iomuxget" => iomux::get(config, env),
        "iomuxset" => iomux::set(config, env),
        "inb" => pio::inb(config, env),
        "inl" => pio::inl(config, env),
        "inflate" => inflate::run(config, env),
        "inw" => pio::inw(config, env),
        "jfmt" => jfmt::run(config, env),
        "load" => load::run(config, env),
        "loadmem" => load::loadmem(config, env),
        "ls" | "list" => list::run(config, env),
        "map" => vm::map(config, env),
        "mapping" => vm::mapping(config, env),
        "mappings" => vm::mappings(config, env),
        "mount" => mount::run(config, env),
        "outb" => pio::outb(config, env),
        "outl" => pio::outl(config, env),
        "outw" => pio::outw(config, env),
        "peek" => memory::read(config, env),
        "poke" => memory::write(config, env),
        "pop" => Ok(pop2(env)),
        "push" => Ok(Value::Nil),
        "rdmsr" => msr::read(config, env),
        "rdsmn" => smn::read(config, env),
        "rx" => rx::run(config, env),
        "rz" => rz::run(config, env),
        "setbits" => bits::set(config, env),
        "sha256" => sha::run(config, env),
        "sha256mem" => sha::mem(config, env),
        "unmap" => vm::unmap(config, env),
        "wrmsr" => msr::write(config, env),
        "wrsmn" => smn::write(config, env),
        _ => Err(Error::NoCommand),
    }
}

fn dup(env: &mut Vec<Value>) -> Value {
    if let Some(v) = env.pop() {
        env.push(v.clone());
        env.push(v.clone());
        v
    } else {
        Value::Nil
    }
}

fn swaptop(env: &mut [Value]) -> Value {
    let len = env.len();
    if len > 1 {
        env.swap(len - 1, len - 2);
        env[len - 1].clone()
    } else {
        Value::Nil
    }
}

fn popenv(env: &mut Vec<Value>) -> Value {
    if let Some(v) = env.pop() { v } else { Value::Nil }
}

fn pop2(env: &mut Vec<Value>) -> Value {
    popenv(env);
    popenv(env)
}

fn eval(
    config: &mut bldb::Config,
    cmd: &reader::Command,
    env: &mut Vec<Value>,
) -> Result<Value> {
    match cmd {
        reader::Command::Push => Ok(dup(env)),
        reader::Command::Swap => Ok(swaptop(env)),
        reader::Command::Cmd(_, tokens) => {
            let mut tokens = tokens.clone();
            while let Some(token) = tokens.pop() {
                match token {
                    reader::Token::Push => {
                        dup(env);
                    }
                    reader::Token::Swap => {
                        swaptop(env);
                    }
                    reader::Token::Term => env.push(Value::Nil),
                    reader::Token::Value(v) => env.push(v),
                }
            }
            let Some(Value::Cmd(cmd)) = env.pop() else {
                return Ok(Value::Nil);
            };
            match evalcmd(config, &cmd, env)? {
                Value::Nil => Ok(Value::Nil),
                v => {
                    env.push(v.clone());
                    Ok(v)
                }
            }
        }
    }
}

pub(crate) fn run(config: &mut bldb::Config) {
    let mut env = Vec::<Value>::new();
    let mut val = Value::default();
    loop {
        match reader::read(config, &mut env, &val) {
            Err(e) => {
                println!("reader: {:?}", e);
                continue;
            }
            Ok(mut cmdstack) => {
                while let Some(cmd) = cmdstack.pop() {
                    match eval(config, &cmd, &mut env) {
                        Err(e) => {
                            println!("eval: '{cmd:?}': {e:?}");
                            env.clear();
                            val = Value::Nil;
                        }
                        Ok(v) => val = v,
                    }
                }
                println!("res: {val:?}");
            }
        }
    }
}
