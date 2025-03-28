// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use bitstruct::bitstruct;
use core::{fmt, ptr};

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum DebounceCtl {
    No = 0b00,
    PreserveLoGlitch = 0b01,
    PreserveHiGlitch = 0b10,
    _Reserved = 0b11,
}

#[derive(Clone, Copy, Debug)]
pub enum TriggerType {
    Edge,
    Level,
}

#[derive(Clone, Copy, Debug)]
pub enum ActiveLevel {
    High,
    Low,
    BothEdges,
    _Reserved,
}

#[derive(Clone, Copy, Debug)]
pub enum PinStatus {
    Low,
    High,
}

/// Represents the drive strength for a given pin.
///
/// Note that the specific values vary based on whether the PAD
/// in question is operating at 3.3V or 1.8V.  The 3V3 PAD
/// values ignore the higher bit, and only examine the lower
/// bit, giving a binary choice between 40 Ohms for 0bX0 and 80
/// Ohms at 0bX1.  The 1V8 PAD values consider both bits
/// significant, but list 0b00 as "unsupported", even though
/// this is a valid value for 40 Ohms at 3V3.  To support both
/// PAD types with a single enum, we always use 0b10 for 40 Ohms
/// at both 1V8 and 3V3, and never use 0b00.  Note that there is
/// no way to configure driving a 3V3 PAD at 60 Ohms, even
/// though 0b01 is a valid value there; at 3V3 the `Z60` variant
/// will be treated as 80 Ohms.  Caveat emptor.
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum DriveStrength {
    Unsupported = 0b00, // (40 Ohms for 3V3 pad)
    Z60 = 0b01,         // 60 Ohms (80 Ohms for 3V3 pad)
    Z40 = 0b10,         // 40 Ohms
    Z80 = 0b11,         // 80 Ohms
}

bitstruct! {
    /// Represents a GPIO register in the FCH.
    #[derive(Clone, Copy, Debug)]
    pub(crate) struct Reg(pub u32) {
        pub debounce_timer: u8 = 0..=3;
        pub debounce_timer_unit: u8 = 4;
        pub debounce_ctl: DebounceCtl = 5..=6;
        pub debounce_timer_large: u8 = 7;
        pub trigger_type: TriggerType = 8;
        pub active_level: ActiveLevel = 9..=10;
        pub interrupt_status_enable: bool = 11;
        pub interrupt_enable: bool = 12;
        pub wake_in_power_saving_mode: bool = 13;
        pub wake_in_s3: bool = 14;
        pub wake_in_s4_or_s5: bool = 15;
        pub pin_status: PinStatus = 16;
        pub drive_strength: DriveStrength = 17..=18;
        resv0: bool = 19;
        pub pull_up_enable: bool = 20;
        pub pull_down_enable: bool = 21;
        pub output_value: PinStatus = 22;
        pub output_enable: bool = 23;
        pub sw_ctl_input: PinStatus = 24;
        pub sw_ctl_input_enable: bool = 25;
        pub rx_disable: bool = 26;
        resv1: bool = 27;
        pub interrupt_status: bool = 28;
        pub wake_status: bool = 29;
        pub gpio0_pwr_btn_press_less_2sec_status: bool = 30;
        pub gpio0_pwr_btn_press_less_10s_status: bool = 31;
    }
}

impl Reg {
    /// Returns the raw u32 bits for this register.
    pub fn bits(self) -> u32 {
        self.0
    }
}

impl bitstruct::FromRaw<u8, DebounceCtl> for Reg {
    fn from_raw(raw: u8) -> DebounceCtl {
        match raw & 0b11 {
            0b00 => DebounceCtl::No,
            0b01 => DebounceCtl::PreserveLoGlitch,
            0b10 => DebounceCtl::PreserveHiGlitch,
            0b11 => DebounceCtl::_Reserved,
            _ => panic!("impossible data bits value"),
        }
    }
}

impl bitstruct::IntoRaw<u8, DebounceCtl> for Reg {
    fn into_raw(ctl: DebounceCtl) -> u8 {
        match ctl {
            DebounceCtl::No => 0b00,
            DebounceCtl::PreserveLoGlitch => 0b01,
            DebounceCtl::PreserveHiGlitch => 0b10,
            DebounceCtl::_Reserved => 0b11,
        }
    }
}

impl bitstruct::FromRaw<bool, TriggerType> for Reg {
    fn from_raw(raw: bool) -> TriggerType {
        match raw {
            false => TriggerType::Edge,
            true => TriggerType::Level,
        }
    }
}

impl bitstruct::IntoRaw<bool, TriggerType> for Reg {
    fn into_raw(trigger_type: TriggerType) -> bool {
        match trigger_type {
            TriggerType::Edge => false,
            TriggerType::Level => true,
        }
    }
}

impl bitstruct::FromRaw<u8, ActiveLevel> for Reg {
    fn from_raw(raw: u8) -> ActiveLevel {
        match raw & 0b11 {
            0b00 => ActiveLevel::High,
            0b01 => ActiveLevel::Low,
            0b10 => ActiveLevel::BothEdges,
            0b11 => ActiveLevel::_Reserved,
            _ => panic!("impossible data bits value"),
        }
    }
}

impl bitstruct::IntoRaw<u8, ActiveLevel> for Reg {
    fn into_raw(level: ActiveLevel) -> u8 {
        match level {
            ActiveLevel::High => 0b00,
            ActiveLevel::Low => 0b01,
            ActiveLevel::BothEdges => 0b10,
            ActiveLevel::_Reserved => 0b11,
        }
    }
}

impl bitstruct::FromRaw<bool, PinStatus> for Reg {
    fn from_raw(raw: bool) -> PinStatus {
        match raw {
            false => PinStatus::Low,
            true => PinStatus::High,
        }
    }
}

impl bitstruct::IntoRaw<bool, PinStatus> for Reg {
    fn into_raw(status: PinStatus) -> bool {
        match status {
            PinStatus::Low => false,
            PinStatus::High => true,
        }
    }
}

impl bitstruct::FromRaw<u8, DriveStrength> for Reg {
    fn from_raw(raw: u8) -> DriveStrength {
        match raw & 0b11 {
            0b00 => DriveStrength::Unsupported,
            0b01 => DriveStrength::Z60,
            0b10 => DriveStrength::Z40,
            0b11 => DriveStrength::Z80,
            _ => panic!("impossible data bits value"),
        }
    }
}

impl bitstruct::IntoRaw<u8, DriveStrength> for Reg {
    fn into_raw(strength: DriveStrength) -> u8 {
        match strength {
            DriveStrength::Unsupported => 0b00,
            DriveStrength::Z60 => 0b01,
            DriveStrength::Z40 => 0b10,
            DriveStrength::Z80 => 0b11,
        }
    }
}

/// Represents the bank(s) of GPIOs on the SoC.
pub(crate) struct Gpios {
    gpios: [u32; 256],
}

impl Gpios {
    /// Returns the function currently configured for the given pin
    /// in the IO mux.
    pub fn get_pin(&self, pin: u8) -> Reg {
        let k = pin as usize;
        Reg(unsafe { ptr::read_volatile(&self.gpios[k]) })
    }

    /// Sets the IO mux configuration for the given pin to the
    /// given function.
    ///
    /// # Safety
    /// The caller must ensure that the given mux settings are
    /// correct.
    pub unsafe fn set_pin(&mut self, pin: u8, reg: Reg) {
        unsafe {
            let k = pin as usize;
            ptr::write_volatile(&mut self.gpios[k], reg.0);
        }
    }
}

impl fmt::Debug for Gpios {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let start = self.gpios.as_ptr().addr();
        let end = start + self.gpios.len();
        let gpios = start..end;
        write!(f, "{gpios:#x?}")
    }
}

pub unsafe fn init() -> &'static mut Gpios {
    const GPIO_BASE_ADDR_OFFSET: usize = 0x0500;
    let base_addr = bldb::gpio_page_addr().addr() + GPIO_BASE_ADDR_OFFSET;
    let ptr = ptr::with_exposed_provenance_mut::<Gpios>(base_addr);
    unsafe { &mut *ptr }
}
