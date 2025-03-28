// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::result::{Error, Result};
use core::convert::TryFrom;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Bus(pub u8);

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub(crate) enum Device {
    D0 = 0,
    D1 = 1,
    D2 = 2,
    D3 = 3,
    D4 = 4,
    D5 = 5,
    D6 = 6,
    D7 = 7,
    D8 = 8,
    D9 = 9,
    D10 = 10,
    D11 = 11,
    D12 = 12,
    D13 = 13,
    D14 = 14,
    D15 = 15,
    D16 = 16,
    D17 = 17,
    D18 = 18,
    D19 = 19,
    D20 = 20,
    D21 = 21,
    D22 = 22,
    D23 = 23,
    D24 = 24,
    D25 = 25,
    D26 = 26,
    D27 = 27,
    D28 = 28,
    D29 = 29,
    D30 = 30,
    D31 = 31,
}

impl TryFrom<u8> for Device {
    type Error = Error;
    fn try_from(f: u8) -> Result<Self> {
        match f {
            0 => Ok(Device::D0),
            1 => Ok(Device::D1),
            2 => Ok(Device::D2),
            3 => Ok(Device::D3),
            4 => Ok(Device::D4),
            5 => Ok(Device::D5),
            6 => Ok(Device::D6),
            7 => Ok(Device::D7),
            8 => Ok(Device::D8),
            9 => Ok(Device::D9),
            10 => Ok(Device::D10),
            11 => Ok(Device::D11),
            12 => Ok(Device::D12),
            13 => Ok(Device::D13),
            14 => Ok(Device::D14),
            15 => Ok(Device::D15),
            16 => Ok(Device::D16),
            17 => Ok(Device::D17),
            18 => Ok(Device::D18),
            19 => Ok(Device::D19),
            20 => Ok(Device::D20),
            21 => Ok(Device::D21),
            22 => Ok(Device::D22),
            23 => Ok(Device::D23),
            24 => Ok(Device::D24),
            25 => Ok(Device::D25),
            26 => Ok(Device::D26),
            27 => Ok(Device::D27),
            28 => Ok(Device::D28),
            29 => Ok(Device::D29),
            30 => Ok(Device::D30),
            31 => Ok(Device::D31),
            _ => Err(Error::NumRange),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub(crate) enum Function {
    F0 = 0,
    F1 = 1,
    F2 = 2,
    F3 = 3,
    F4 = 4,
    F5 = 5,
    F6 = 6,
    F7 = 7,
}

impl TryFrom<u8> for Function {
    type Error = Error;
    fn try_from(f: u8) -> Result<Self> {
        match f {
            0 => Ok(Function::F0),
            1 => Ok(Function::F1),
            2 => Ok(Function::F2),
            3 => Ok(Function::F3),
            4 => Ok(Function::F4),
            5 => Ok(Function::F5),
            6 => Ok(Function::F6),
            7 => Ok(Function::F7),
            _ => Err(Error::NumRange),
        }
    }
}

mod legacy {
    use super::{Bus, Device, Function};

    static IO_MUTEX: spin::Mutex<()> = spin::Mutex::new(());
    const PCI_CFG_ADDR_PORT: u16 = 0xCF8;
    const PCI_CFG_DATA_PORT: u16 = 0xCFC;

    #[derive(Clone, Copy, Debug)]
    pub(super) struct Address(pub(super) u32);
    impl Address {
        pub(super) fn addr(self) -> u32 {
            self.0
        }
    }

    pub(super) fn config_addr(
        bus: Bus,
        dev: Device,
        func: Function,
        offset: u8,
    ) -> Address {
        use bit_field::BitField;
        let addr = 0u32
            .set_bit(31, true)
            .set_bits(16..24, bus.0.into())
            .set_bits(11..16, dev as u32)
            .set_bits(8..11, func as u32)
            .set_bits(0..8, offset.into())
            .get_bits(..);
        Address(addr)
    }

    pub(super) unsafe fn write(addr: Address, val: u32) {
        let _guard = IO_MUTEX.lock();
        unsafe {
            x86::io::outl(PCI_CFG_ADDR_PORT, addr.addr());
            x86::io::outl(PCI_CFG_DATA_PORT, val);
        }
    }

    pub(super) unsafe fn read(addr: Address) -> u32 {
        let _guard = IO_MUTEX.lock();
        unsafe {
            x86::io::outl(PCI_CFG_ADDR_PORT, addr.0);
            x86::io::inl(PCI_CFG_DATA_PORT)
        }
    }
}

pub(crate) mod cfg {
    use super::{Bus, Device, Function, Result, legacy};

    pub(crate) unsafe fn write<T: Into<u32>>(
        bus: Bus,
        dev: Device,
        func: Function,
        offset: u8,
        val: T,
    ) -> Result<()> {
        let addr = legacy::config_addr(bus, dev, func, offset);
        unsafe {
            legacy::write(addr, val.into());
        }
        Ok(())
    }

    pub(crate) unsafe fn read<T: TryFrom<u32>>(
        bus: Bus,
        dev: Device,
        func: Function,
        offset: u8,
    ) -> Result<T> {
        let addr = legacy::config_addr(bus, dev, func, offset);
        unsafe {
            legacy::read(addr)
                .try_into()
                .map_err(|_| crate::result::Error::NumRange)
        }
    }
}

pub(crate) mod ecam {
    use super::{Bus, Device, Function, legacy};
    use crate::result::{Error, Result};
    use bit_field::BitField;
    use core::convert::TryFrom;

    #[derive(Clone, Copy, Debug)]
    pub(crate) struct Offset(u32);

    impl Offset {
        fn lo(self) -> u8 {
            self.0.get_bits(0..8) as u8
        }
        fn nibble(self) -> u32 {
            self.0.get_bits(8..12)
        }

        pub fn addr(self) -> u32 {
            self.0
        }
    }

    impl TryFrom<u32> for Offset {
        type Error = Error;
        fn try_from(v: u32) -> Result<Self> {
            if v.get_bits(12..32) == 0 {
                Ok(Offset(v))
            } else {
                Err(crate::result::Error::NumRange)
            }
        }
    }

    fn pio_config_addr(
        bus: Bus,
        dev: Device,
        func: Function,
        offset: Offset,
    ) -> legacy::Address {
        let addr = legacy::config_addr(bus, dev, func, offset.lo())
            .addr()
            .set_bits(24..28, offset.nibble())
            .get_bits(..);
        legacy::Address(addr)
    }

    pub(crate) unsafe fn write<T: Into<u32>>(
        bus: Bus,
        dev: Device,
        func: Function,
        offset: Offset,
        val: T,
    ) -> Result<()> {
        let addr = pio_config_addr(bus, dev, func, offset);
        unsafe {
            legacy::write(addr, val.into());
        }
        Ok(())
    }

    pub(crate) unsafe fn read<T: TryFrom<u32>>(
        bus: Bus,
        dev: Device,
        func: Function,
        offset: Offset,
    ) -> Result<T> {
        let addr = pio_config_addr(bus, dev, func, offset);
        unsafe {
            legacy::read(addr)
                .try_into()
                .map_err(|_| crate::result::Error::NumRange)
        }
    }
}
