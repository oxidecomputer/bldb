// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::println;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use crate::uart::Uart;
use alloc::vec::Vec;
use zmodem2::{Read, Write};

use core::result::Result as ZResult;

impl Read for Uart {
    fn read_byte(&mut self) -> ZResult<u8, zmodem2::Error> {
        self.try_getb().map_err(|_| zmodem2::Error::Read)
    }

    fn read(&mut self, dst: &mut [u8]) -> ZResult<u32, zmodem2::Error> {
        let nb = self.try_getbs(dst).map_err(|_| zmodem2::Error::Read)?;
        Ok(nb.try_into().unwrap())
    }
}

impl Write for Uart {
    fn write_byte(&mut self, b: u8) -> ZResult<(), zmodem2::Error> {
        self.try_putb(b).map_err(|_| zmodem2::Error::Write)
    }

    fn write_all(&mut self, bs: &[u8]) -> ZResult<(), zmodem2::Error> {
        self.putbs(bs).map_err(|_| zmodem2::Error::Write)
    }
}

struct SliceVec<'a> {
    buf: &'a mut [u8],
    off: usize,
}

impl<'a> Write for SliceVec<'a> {
    fn write_byte(&mut self, b: u8) -> ZResult<(), zmodem2::Error> {
        let dst = &mut self.buf[self.off..];
        if dst.is_empty() {
            return Err(zmodem2::Error::Write);
        }
        dst[0] = b;
        self.off += 1;
        Ok(())
    }

    fn write_all(&mut self, src: &[u8]) -> ZResult<(), zmodem2::Error> {
        let dst = &mut self.buf[self.off..];
        if dst.len() < src.len() {
            return Err(zmodem2::Error::Write);
        }
        let dst = &mut dst[..src.len()];
        dst.copy_from_slice(src);
        self.off += src.len();
        Ok(())
    }
}

fn rz(uart: &mut Uart, dst: &mut [u8]) -> Result<usize> {
    println!("receiving to {:#x?}", dst.as_ptr());
    let mut state = zmodem2::State::new();
    let mut v = SliceVec { buf: dst, off: 0 };
    while state.stage() != zmodem2::Stage::Done {
        if let Err(e) = zmodem2::receive(uart, &mut v, &mut state) {
            println!("zmodem error: {e:?}");
            return Err(Error::Recv);
        }
    }
    Ok(state.file_size().try_into().unwrap())
}

pub fn run(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: rz <dst addr>,<dst len>");
        error
    };
    let dst = repl::popenv(env)
        .as_slice_mut(&config.page_table, 0)
        .map_err(usage)?
        .unwrap_or_else(|| bldb::xfer_region_init_mut());
    let nrecv = rz(&mut config.cons, dst)?;
    println!("\n\nReceived {nrecv} bytes");
    Ok(Value::Slice(&dst[..nrecv]))
}
