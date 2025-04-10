// Copyright 2021  The Hypatia Authors
// All rights reserved
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use crate::cpuid;

pub const NANOS_PER_SEC: u128 = 1_000_000_000;

/// Returns the clock frequency of the current CPU in Hertz.
pub fn frequency() -> u128 {
    const DEFAULT_HZ: u128 = 2_000_000_000;
    if let Some(tsc_info) = cpuid::tscinfo() {
        if tsc_info.nominal_frequency() != 0 {
            return tsc_info
                .tsc_frequency()
                .map(|freq| freq.into())
                .unwrap_or(DEFAULT_HZ);
        }
    }
    DEFAULT_HZ
}

pub fn rdtsc() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}
