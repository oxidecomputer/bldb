// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::result::Result;

/// A "Storage Device" that represents the memory allocated to
/// a ramdisk.
///
/// This is essentially a destructured slice, which we introduce
/// to work around some lifetime issues.
#[derive(Debug)]
pub(crate) struct Sd {
    pub(crate) ptr: *const u8,
    pub(crate) len: usize,
}

impl Sd {
    pub(crate) unsafe fn new(ptr: *const u8, len: usize) -> Sd {
        Sd { ptr, len }
    }

    /// Creates a new `Sd` from a slice.
    ///
    /// # Safety
    /// It is up to the caller to ensure that the data in `bs`
    /// is not moved or dropped while this `Sd`, or any other
    /// derived from it, is alive.
    pub(crate) unsafe fn from_slice(bs: &[u8]) -> Sd {
        unsafe { Sd::new(bs.as_ptr(), bs.len()) }
    }

    /// Reconstitutes this `Sd`` into a slice
    pub(crate) unsafe fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
    }

    pub(crate) fn data(&self) -> *const u8 {
        self.ptr
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn subset(&self, offset: usize, len: usize) -> Sd {
        assert!(offset + len <= self.len);
        let ptr = self.ptr.wrapping_add(offset);
        Sd { ptr, len }
    }
}

pub(crate) trait Read {
    fn read(&self, off: u64, dst: &mut [u8]) -> Result<usize>;
    fn size(&self) -> usize;
}

impl Read for &[u8] {
    fn read(&self, off: u64, dst: &mut [u8]) -> Result<usize> {
        let off = off as usize;
        if off >= self.len() {
            return Ok(0);
        }
        let bytes = &self[off..];
        let len = usize::min(bytes.len(), dst.len());
        if len > 0 {
            dst[..len].copy_from_slice(&bytes[..len]);
        }
        Ok(len)
    }

    fn size(&self) -> usize {
        self.len()
    }
}
