// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use bitstruct::bitstruct;
use core::ops::Range;

pub(crate) const KIB: usize = 1024;
pub(crate) const MIB: usize = 1024 * KIB;
pub(crate) const GIB: usize = 1024 * MIB;

/// A V4KA represents a 4KiB aligned, canonical virtual memory
/// address.  The address may or may not be mapped.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub(crate) struct V4KA(usize);

/// Lower canonical address space supremum.
pub const LOW_CANON_SUP: usize = 0x0000_7FFF_FFFF_FFFF + 1;
// Higher canonical address space infimum.
pub const HI_CANON_INF: usize = 0xFFFF_8000_0000_0000 - 1;

/// Returns true IFF the given address is canonical.
pub const fn is_canonical(va: usize) -> bool {
    va <= 0x0000_7FFF_FFFF_FFFF || 0xFFFF_8000_0000_0000 <= va
}

/// Returns true IFF the address is a valid physical address.
pub const fn is_physical(pa: u64) -> bool {
    pa < (1 << 46)
}

/// Returns true IFF the range of virtual addresses
/// in [start, end) is canonical.
pub const fn is_canonical_range(start: usize, end: usize) -> bool {
    // If the range ends before it starts and end is not exactly
    // zero, the range is not canonical.
    if end < start && end != 0 {
        return false;
    }
    // If in the lower portion of the canonical address space,
    // end is permitted to be exactly one beyond the supremum.
    if start < LOW_CANON_SUP && start <= end && end <= LOW_CANON_SUP {
        return true;
    }
    // Otherwise, the range is valid IFF it is in the upper
    // portion of the canonical address space, or end is 0.
    HI_CANON_INF < start && (HI_CANON_INF < end || end == 0)
}

impl V4KA {
    /// The alignment factor.
    pub(crate) const ALIGN: usize = 4096;
    pub(crate) const MASK: usize = Self::ALIGN - 1;
    pub(crate) const SIZE: usize = Self::ALIGN;

    /// Returns a new V4KA constructed from the given virtual
    /// address, which must be both canonical and properly
    /// aligned.
    pub(crate) const fn new(va: usize) -> V4KA {
        assert!(is_canonical(va));
        assert!(va & Self::MASK == 0);
        V4KA(va)
    }

    /// Returns the integer value of the raw virtual address.
    pub(crate) const fn addr(self) -> usize {
        self.0
    }
}

/// Returns a Range of 4KiB virtual addresses spanning the given
/// region [ptr, ptr + len).
pub fn page_range_raw(ptr: *const (), len: usize) -> Range<V4KA> {
    let addr = ptr.addr();
    let start = V4KA::new(round_down_4k(addr));
    let end = V4KA::new(round_up_4k(addr + len));
    start..end
}

/// A P4KA represents a 4KiB aligned, valid address in the
/// physical address space.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct P4KA(u64);

impl P4KA {
    /// The alignment factor.
    pub(crate) const ALIGN: u64 = 4096;
    pub(crate) const MASK: u64 = Self::ALIGN - 1;

    /// Constructs a new P4KA from the given physical address,
    /// must be properly aligned and lie within the range of the
    /// physical address space.
    pub(crate) const fn new(pa: u64) -> P4KA {
        assert!(is_physical(pa));
        assert!(pa & Self::MASK == 0);
        P4KA(pa)
    }

    /// Returns the integer value of the raw physical address.
    pub(crate) const fn phys_addr(self) -> u64 {
        self.0
    }
}

bitstruct! {
    /// Records the permissions of a mapped into the virtual address
    /// space.
    #[derive(Clone, Copy, Debug)]
    pub(crate) struct Attrs(u64) {
        /// True if readable.
        pub(crate) r: bool = 0;
        /// True if writable.
        pub(crate) w: bool = 1;
        /// False if cacheable.
        pub(crate) nc: bool = 4;
        /// True if global.
        pub(crate) g: bool = 8;
        /// True if part of the kernel nucleus
        pub(crate) k: bool = 11;
        /// False if executable.
        pub(crate) nx: bool = 63;
    }
}

impl Attrs {
    /// Returns empty Attrs.
    pub(crate) fn empty() -> Self {
        Self(0)
    }

    /// Returns a new Attrs structure with the given permissions.
    pub(crate) fn new(r: bool, w: bool, x: bool, c: bool, k: bool) -> Self {
        Self(0).with_r(r).with_w(w).with_x(x).with_nc(!c).with_k(k)
    }

    /// Returns a new Attrs specialized for loader text.
    pub(crate) fn new_text() -> Self {
        Self(0).with_r(true).with_x(true)
    }

    /// Returns a new Attrs specialized for loader read-only
    /// data.
    pub(crate) fn new_rodata() -> Self {
        Self(0).with_r(true).with_nx(true)
    }

    /// Returns a new Attrs specialized for loader read/write
    /// data.
    pub(crate) fn new_data() -> Self {
        Self(0).with_r(true).with_w(true).with_nx(true)
    }

    /// Returns new Attrs specialized for loader BSS.  These are
    /// functionally identical to data attributes.
    pub(crate) fn new_bss() -> Self {
        Self::new_data()
    }

    /// Returns new Attrs specialized for MMIO regions. Notably,
    /// these are uncached.
    pub(crate) fn new_mmio() -> Self {
        Attrs(0).with_r(true).with_w(true).with_x(false).with_c(false)
    }

    /// Returns new Attrs suitable for the host kernel nucleus.
    pub(crate) fn new_kernel(r: bool, w: bool, x: bool) -> Self {
        Attrs(0).with_r(r).with_w(w).with_x(x).with_k(true)
    }

    /// Returns new Attrs suitable for matching for read.
    pub(crate) fn new_ro() -> Self {
        Self(0).with_r(true)
    }

    /// Returns new Attrs suitable for matching for write.
    pub(crate) fn new_rw() -> Self {
        Self(0).with_r(true).with_w(true)
    }

    /// Returns new Attrs suitable for matching for execute.
    pub(crate) fn new_x() -> Self {
        Self(0).with_x(true)
    }

    /// Returns true IFF executable.
    pub(crate) fn x(&self) -> bool {
        !self.nx()
    }

    /// Returns a new instance of Attrs with `nx` set to the
    /// logical negation of `x`.
    pub(crate) fn with_x(self, x: bool) -> Self {
        self.with_nx(!x)
    }

    /// Sets the value of `nx` to the logical negation of `x`.
    pub(crate) fn set_x(&mut self, x: bool) {
        self.set_nx(!x);
    }

    /// Returns true IFF cacheable.
    pub(crate) fn c(&self) -> bool {
        !self.nc()
    }

    /// Returns a new instance of Attrs with `nc` set to the
    /// logical negation of `c`.
    pub(crate) fn with_c(self, c: bool) -> Self {
        self.with_nc(!c)
    }

    /// Sets the value of `nc` to the logical negation of `c`.
    pub(crate) fn set_c(&mut self, c: bool) {
        self.set_nc(!c);
    }

    pub(crate) fn permits(self, wants: Attrs) -> bool {
        (!wants.r() || self.r())
            && (!wants.w() || self.w())
            && (!wants.nx() || self.nx())
    }
}

/// A region of virtual memory.
#[derive(Clone, Debug)]
pub(crate) struct Region {
    range: Range<V4KA>,
    attrs: Attrs,
}

impl Region {
    /// Returns a new region spanning the given [start, end)
    /// address pair and attributes.
    pub fn new(range: Range<V4KA>, attrs: Attrs) -> Region {
        Region { range, attrs }
    }

    /// Returns the range start address.
    pub fn start(&self) -> V4KA {
        self.range.start
    }

    /// Returns the range end address.
    pub fn end(&self) -> V4KA {
        self.range.end
    }

    /// Returns the range attributes.
    pub fn attrs(&self) -> Attrs {
        self.attrs
    }
}

/// Aligns the given address up to the next higher 4KiB
/// boundary, possibly wrapping around to 0.
pub fn round_up_4k(va: usize) -> usize {
    round_down_4k(va.wrapping_add(V4KA::MASK))
}

/// Aligns the given address to the next lowest 4KIB
/// boundary.
pub fn round_down_4k(va: usize) -> usize {
    va & !V4KA::MASK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attrs_permits() {
        let has = Attrs::new_data();
        assert!(has.nx());
        assert!(has.r());
        assert!(has.w());
        let wants = Attrs::new_ro();
        assert!(!wants.nx());
        assert!(!wants.w());
        assert!(wants.r());
        assert!(has.permits(wants));
    }
}
