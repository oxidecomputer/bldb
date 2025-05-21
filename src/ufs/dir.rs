// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::ufs::{FileType, Inode};
use core::fmt;
use core::mem;

/// The maximum length of a name.
pub const MAX_NAME_LEN: usize = 255;

// Legnth of a diretory prefix (before the name).
pub const PREFIX_LEN: usize = 8;

/// Newtype around an inode representing a directory file.
pub struct Directory {
    pub(super) inode: Inode,
}

impl Directory {
    /// Creates a new directory from the given inode. Asserts
    /// that the inode refers to a directory.
    pub fn new(inode: Inode) -> Directory {
        let mode = inode.mode();
        assert_eq!(mode.typ(), FileType::Dir);
        Directory { inode }
    }

    /// Tries to create a new `Dirctory`` from the given inode.
    /// Returns `None`` if the inode's type is not a directory.
    pub fn try_new(inode: Inode) -> Option<Directory> {
        let isdir = inode.mode().typ() == FileType::Dir;
        isdir.then(|| Self::new(inode))
    }

    /// Returns an interator over the directory entries in this
    /// directory.
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self)
    }
}

/// A directory entry iterator.  Iterates over the directory
/// entries in the given directory.
pub struct Iter<'a> {
    inode: &'a Inode,
    pos: u64,
}

impl Iter<'_> {
    /// Creates a new directory entry iterator for the given
    /// directory.
    pub fn new(dir: &Directory) -> Iter<'_> {
        let pos = 0;
        let inode = &dir.inode;
        Iter { inode, pos }
    }
}

impl Iterator for Iter<'_> {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = [0u8; PREFIX_LEN];
        let nread = self.inode.read(self.pos, &mut buf).ok()?;
        if nread < PREFIX_LEN {
            return None;
        }
        let ino = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let reclen = u16::from_ne_bytes([buf[4], buf[5]]) as usize;
        if reclen == 0 {
            return None;
        }
        let namelen = u16::from_ne_bytes([buf[6], buf[7]]) as usize;
        if reclen - PREFIX_LEN < namelen || namelen > MAX_NAME_LEN {
            return None;
        }
        let mut name = [0u8; MAX_NAME_LEN + 1];
        let dst = &mut name[..namelen];
        let namepos = self.pos + PREFIX_LEN as u64;
        let nread = self.inode.read(namepos, dst).ok()?;
        if nread != namelen {
            return None;
        }
        let entry =
            Entry { ino, reclen: reclen as u16, namelen: namelen as u16, name };
        self.pos += reclen as u64;
        Some(entry)
    }
}

/// The in-memory representation of a directory entry.
#[repr(C)]
pub struct Entry {
    ino: u32,
    reclen: u16,
    namelen: u16,
    name: [u8; MAX_NAME_LEN + 1],
}

impl Entry {
    /// Returns the size of this entry.
    pub fn dirsiz(&self) -> u16 {
        const BASE_SIZE: usize = mem::size_of::<Entry>() - MAX_NAME_LEN - 1; // c'mon dude; it's 264
        let name_size = (self.namelen + 1 + 3) & !3;
        BASE_SIZE as u16 + name_size
    }

    /// Returns the file name contained in this directory entry.
    pub fn name(&self) -> &[u8] {
        let name = &self.name[..self.namelen as usize];
        if let Some(nul) = name.iter().position(|&b| b == 0u8) {
            &name[..nul]
        } else {
            name
        }
    }

    /// Returns the inode number for this directory entry.
    pub fn ino(&self) -> u32 {
        self.ino
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "Entry {{")?;
        writeln!(f, "    size: {}", self.dirsiz())?;
        writeln!(f, "    ino: {}", self.ino)?;
        writeln!(f, "    reclen: {}", self.reclen)?;
        writeln!(f, "    namelen: {}", self.namelen)?;
        let name = unsafe { core::str::from_utf8_unchecked(self.name()) };
        writeln!(f, "    name = {name}")?;
        write!(f, "}}")
    }
}
