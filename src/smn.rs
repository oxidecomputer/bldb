// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::pci;
use crate::result::{Error, Result};
use core::convert::TryFrom;
use spin::Mutex;

pub(crate) enum Index {
    Smn0 = 0,
    Smn1 = 1,
    Smn2 = 2,
    Smn3 = 3,
    Smn4 = 4,
    Smn5 = 5,
    Smn6 = 6,
}

impl TryFrom<u8> for Index {
    type Error = Error;
    fn try_from(f: u8) -> Result<Self> {
        match f {
            0 => Ok(Index::Smn0),
            1 => Ok(Index::Smn1),
            2 => Ok(Index::Smn2),
            3 => Ok(Index::Smn3),
            4 => Ok(Index::Smn4),
            5 => Ok(Index::Smn5),
            6 => Ok(Index::Smn6),
            _ => Err(Error::NumRange),
        }
    }
}

const NSMN: usize = Index::Smn6 as usize + 1;

static ADDR_DATA_PAIRS: [spin::Mutex<(u8, u8)>; NSMN] = [
    Mutex::new((0x60, 0x64)),
    Mutex::new((0xA0, 0xA4)),
    Mutex::new((0xB8, 0xBC)),
    Mutex::new((0xC4, 0xC8)),
    Mutex::new((0xD0, 0xD4)),
    Mutex::new((0xE0, 0xE4)),
    Mutex::new((0xF8, 0xFC)),
];

pub(crate) fn read(k: Index, addr: u32) -> Result<u32> {
    let pair = ADDR_DATA_PAIRS[k as usize].lock();
    let (addr_off, data_off) = *pair;
    let value = unsafe {
        pci::cfg::write(
            pci::Bus(0),
            pci::Device::D0,
            pci::Function::F0,
            addr_off,
            addr,
        )?;
        pci::cfg::read(
            pci::Bus(0),
            pci::Device::D0,
            pci::Function::F0,
            data_off,
        )
    }?;
    Ok(value)
}

pub(crate) unsafe fn write(k: Index, addr: u32, data: u32) -> Result<()> {
    let pair = ADDR_DATA_PAIRS[k as usize].lock();
    let (addr_off, data_off) = *pair;
    unsafe {
        pci::cfg::write(
            pci::Bus(0),
            pci::Device::D0,
            pci::Function::F0,
            addr_off,
            addr,
        )?;
        pci::cfg::write(
            pci::Bus(0),
            pci::Device::D0,
            pci::Function::F0,
            data_off,
            data,
        )?;
    }
    Ok(())
}
