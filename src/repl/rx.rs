// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::println;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use crate::uart::Uart;
use alloc::vec::Vec;
use xmodem::Xmodem;
use xmodem::io::Error as XError;
use xmodem::io::ErrorKind as XErrorKind;

type XResult<T> = core::result::Result<T, XError>;

impl xmodem::io::Read for Uart {
    fn read(&mut self, dst: &mut [u8]) -> XResult<usize> {
        self.read_exact(dst)?;
        Ok(dst.len())
    }

    fn read_exact(&mut self, dst: &mut [u8]) -> XResult<()> {
        self.try_getbs(dst)
            .map_err(|_| XError::new(XErrorKind::Other, "uart"))?;
        Ok(())
    }
}

impl xmodem::io::Write for Uart {
    fn write(&mut self, bs: &[u8]) -> XResult<usize> {
        self.putbs(bs).map_err(|_| XError::new(XErrorKind::Other, "uart"))?;
        Ok(bs.len())
    }

    fn write_all(&mut self, bs: &[u8]) -> XResult<()> {
        self.putbs(bs).map_err(|_| XError::new(XErrorKind::Other, "uart"))?;
        Ok(())
    }

    fn flush(&mut self) -> XResult<()> {
        Ok(())
    }
}

fn rx(uart: &mut Uart, mut dst: &mut [u8]) -> Result<usize> {
    println!("receiving to {:#x?}", dst.as_ptr());
    let b = uart.getb();
    if b != b'g' {
        println!("Aborted!");
        return Err(Error::Recv);
    }
    let mut xfer = Xmodem::new();
    let nrecv = xfer
        .recv(uart, &mut dst, xmodem::Checksum::CRC16)
        .map_err(|_| Error::Recv)?;
    Ok(nrecv)
}

pub fn run(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: rx <dst addr>,<dst len>");
        error
    };
    let dst = repl::popenv(env)
        .as_slice_mut(&config.page_table, 0)
        .map_err(usage)?
        .unwrap_or_else(|| bldb::xfer_region_init_mut());
    let nrecv = rx(&mut config.cons, dst)?;
    println!("\n\nReceived {nrecv} bytes");
    Ok(Value::Slice(&dst[..nrecv]))
}
