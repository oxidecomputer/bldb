// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::println;
use crate::repl::{self, Value};
use crate::result::Result;
use alloc::vec::Vec;

#[derive(Clone, Copy)]
enum PortSize {
    P8,
    P16,
    P32,
}

impl PortSize {
    fn as_char(self) -> char {
        match self {
            Self::P8 => 'b',
            Self::P16 => 'w',
            Self::P32 => 'l',
        }
    }
}

fn pio_in(port_size: PortSize, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: in{} <port>", port_size.as_char());
        error
    };
    let port = repl::popenv(env).as_num::<u16>().map_err(usage)?;
    let value = match port_size {
        PortSize::P8 => unsafe { x86::io::inb(port).into() },
        PortSize::P16 => unsafe { x86::io::inw(port).into() },
        PortSize::P32 => unsafe { x86::io::inl(port).into() },
    };
    Ok(Value::Unsigned(value))
}

pub fn inb(_config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    pio_in(PortSize::P8, env)
}

pub fn inw(_config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    pio_in(PortSize::P16, env)
}

pub fn inl(_config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    pio_in(PortSize::P32, env)
}

fn pio_out(port_size: PortSize, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: out{} <port> <value>", port_size.as_char());
        error
    };
    let port = repl::popenv(env).as_num::<u16>().map_err(usage)?;
    match port_size {
        PortSize::P8 => repl::popenv(env).as_num().map(|value| unsafe {
            x86::io::outb(port, value);
        }),
        PortSize::P16 => repl::popenv(env).as_num().map(|value| unsafe {
            x86::io::outw(port, value);
        }),
        PortSize::P32 => repl::popenv(env).as_num().map(|value| unsafe {
            x86::io::outl(port, value);
        }),
    }
    .map_err(usage)?;
    Ok(Value::Nil)
}

pub fn outb(_config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    pio_out(PortSize::P8, env)
}

pub fn outw(_config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    pio_out(PortSize::P16, env)
}

pub fn outl(_config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    pio_out(PortSize::P32, env)
}
