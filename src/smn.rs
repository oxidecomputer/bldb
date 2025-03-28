// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::pci;
use crate::result::Result;

static SMN_MUTEX: spin::Mutex<()> = spin::Mutex::new(());
const SMN_ADDR_OFFSET: u8 = 0x60;
const SMN_DATA_OFFSET: u8 = 0x64;

pub(crate) fn read(addr: u32) -> Result<u32> {
    let _lock = SMN_MUTEX.lock();
    let value = unsafe {
        pci::cfg::write(
            pci::Bus(0),
            pci::Device::D0,
            pci::Function::F0,
            SMN_ADDR_OFFSET,
            addr,
        )?;
        pci::cfg::read(
            pci::Bus(0),
            pci::Device::D0,
            pci::Function::F0,
            SMN_DATA_OFFSET,
        )
    }?;
    Ok(value)
}

pub(crate) unsafe fn write(addr: u32, data: u32) -> Result<()> {
    let _lock = SMN_MUTEX.lock();
    unsafe {
        pci::cfg::write(
            pci::Bus(0),
            pci::Device::D0,
            pci::Function::F0,
            SMN_ADDR_OFFSET,
            addr,
        )?;
        pci::cfg::write(
            pci::Bus(0),
            pci::Device::D0,
            pci::Function::F0,
            SMN_DATA_OFFSET,
            data,
        )?;
    }
    Ok(())
}
