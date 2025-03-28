// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{bldb, cpuid};
use core::{fmt, ptr};

/// Supported pin functions.
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum PinFunction {
    F0 = 0b00,
    F1 = 0b01,
    F2 = 0b10,
    F3 = 0b11,
}

#[repr(transparent)]
pub struct IoMux {
    mux: [u8; 256],
}

impl IoMux {
    /// Returns the function currently configured for the given pin
    /// in the IO mux.
    pub fn get_pin(&self, pin: u8) -> PinFunction {
        let k = pin as usize;
        let raw = unsafe { ptr::read_volatile(&self.mux[k]) };
        match raw & 0b0000_0011 {
            0b00 => PinFunction::F0,
            0b01 => PinFunction::F1,
            0b10 => PinFunction::F2,
            0b11 => PinFunction::F3,
            _ => unreachable!(),
        }
    }

    /// Sets the IO mux configuration for the given pin to the
    /// given function.
    ///
    /// # Safety
    /// The caller must ensure that the given mux settings are
    /// correct.
    pub unsafe fn set_pin(&mut self, pin: u8, function: PinFunction) {
        unsafe {
            let k = pin as usize;
            ptr::write_volatile(&mut self.mux[k], function as u8);
        }
    }
}

impl fmt::Debug for IoMux {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let start = self.mux.as_ptr().addr();
        let end = start + self.mux.len();
        let iomux = start..end;
        write!(f, "{iomux:#x?}")
    }
}

/// Initializes the IO mux so that pins for UART 0 are mapped to
/// UART functions.  In some cases, the values observed are
/// different from the documented reset values, so we force them
/// to our desired settings.
///
/// # Safety
/// The caller must ensure that the IO mux MMIO region is in the
/// current address space.
pub unsafe fn init() -> &'static mut IoMux {
    const IOMUX_BASE_ADDR_OFFSET: usize = 0x0D00;
    let base_addr = bldb::iomux_page_addr().addr() + IOMUX_BASE_ADDR_OFFSET;
    let ptr = ptr::with_exposed_provenance_mut::<IoMux>(base_addr);
    let iomux = unsafe { &mut *ptr };
    if let Some(settings) = mux_settings() {
        for &(pin, function) in settings.iter() {
            unsafe {
                iomux.set_pin(pin, function);
            }
        }
    }
    iomux
}

/// Returns the correct IO mux settings for the current system,
/// if any.
fn mux_settings() -> Option<&'static [(u8, PinFunction)]> {
    const IOMUX135_GPIO: u8 = 135;
    const IOMUX136_GPIO: u8 = 136;
    const IOMUX137_GPIO: u8 = 137;
    const IOMUX138_GPIO: u8 = 138;
    const SP5: u32 = 4;

    match cpuid::cpuinfo()? {
        // We really ought to explicitly check socket type
        // for the earlier processor models here.
        (0x17, 0x00..=0x0f, 0x0..=0xf, _) | // Naples
        (0x17, 0x30..=0x3f, 0x0..=0xf, _) | // Rome
        (0x19, 0x00..=0x0f, 0x0..=0xf, _) | // Milan
        (0x19, 0x10..=0x1f, 0x0..=0xf, Some(SP5)) | // Genoa
        (0x19, 0xa0..=0xaf, 0x0..=0xf, Some(SP5)) | // Bergamo and Sienna
        (0x1a, 0x00..=0x1f, 0x0..=0xf, Some(SP5)) => { // Turin
            Some(&[
                (IOMUX135_GPIO, PinFunction::F0),
                (IOMUX136_GPIO, PinFunction::F0),
                (IOMUX137_GPIO, PinFunction::F0),
                (IOMUX138_GPIO, PinFunction::F0),
            ])
        },
        _ => None,
    }
}
