// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Code for dealing with the UFS ramdisk.

use crate::cpio;
use crate::io;
use crate::println;
use crate::result::{Error, Result};
use crate::uart::Uart;
use crate::ufs;
use alloc::boxed::Box;
use core::convert::TryInto;

/// The type of file, taken from the inode.
///
/// Unix files can be one of a limited set of types; for
/// instance, directories are a type of file.  The type
/// is encoded in the mode field of the inode; these are
/// the various types that are recognized.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FileType {
    Unused,
    Fifo,
    Char,
    Dir,
    Block,
    Regular,
    SymLink,
    ShadowInode,
    Sock,
    AttrDir,
}

pub trait File: io::Read {
    fn file_type(&self) -> FileType;
}

pub trait FileSystem {
    fn open(&self, path: &str) -> Result<Box<dyn File>>;
    fn list(&self, path: &str) -> Result<()>;
    fn as_str(&self) -> &str;
}

pub fn mount(ramdisk: &'static [u8]) -> Result<Box<dyn FileSystem>> {
    mount_cpio(ramdisk).or_else(|_| {
        let fs = ufs::FileSystem::new(ramdisk)?;
        if let Ok(ufs::State::Clean) = fs.state() {
            let flags = fs.flags();
            println!("ramdisk mounted successfully (Clean, {flags:?})");
            Ok(Box::new(fs))
        } else {
            println!("ramdisk mount failed: invalid state {:?}", fs.state());
            Err(Error::FsInvState)
        }
    })
}

pub fn mount_cpio(ramdisk: &'static [u8]) -> Result<Box<dyn FileSystem>> {
    let fs = Box::new(cpio::FileSystem::try_new(ramdisk)?);
    println!("cpio miniroot mounted successfully");
    Ok(fs)
}

pub fn list(fs: &dyn FileSystem, path: &str) -> Result<()> {
    fs.list(path)
}

pub fn cat(uart: &mut Uart, fs: &dyn FileSystem, path: &str) -> Result<()> {
    let file = fs.open(path)?;
    if file.file_type() != FileType::Regular {
        println!("cat: not a regular file");
        return Err(Error::BadArgs);
    }
    let mut offset = 0;
    let size = file.size();
    while offset != size {
        let mut buf = [0u8; 1024];
        let nb = file.read(offset.try_into().unwrap(), &mut buf)?;
        uart.putbs_crnl(&buf[..nb]);
        offset += nb;
    }
    Ok(())
}

pub fn copy(fs: &dyn FileSystem, path: &str, dst: &mut [u8]) -> Result<usize> {
    let file = fs.open(path)?;
    if file.file_type() != FileType::Regular {
        println!("copy: not a regular file");
        return Err(Error::BadArgs);
    }
    let len = core::cmp::min(file.size(), dst.len());
    let nb = file.read(0, &mut dst[..len])?;
    Ok(nb)
}

pub fn sha256(fs: &dyn FileSystem, path: &str) -> Result<[u8; 32]> {
    use sha2::{Digest, Sha256};

    let file = fs.open(path)?;
    if file.file_type() != FileType::Regular {
        println!("sha256: can only sum regular files");
        return Err(Error::BadArgs);
    }
    let mut sum = Sha256::new();
    let mut offset = 0;
    let size = file.size();
    while offset != size {
        let mut buf = [0u8; 1024];
        let nb = file.read(offset.try_into().unwrap(), &mut buf)?;
        sum.update(&buf[..nb]);
        offset += nb;
    }
    let hash = sum.finalize();
    Ok(hash.into())
}
