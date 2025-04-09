// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Code for dealing with the UFS ramdisk.

use crate::println;
use crate::result::{Error, Result};
use crate::uart::Uart;
use crate::ufs;

pub fn mount(
    ramdisk: &'static [u8],
) -> Result<Option<ufs::FileSystem<'static>>> {
    let fs = ufs::FileSystem::new(ramdisk);
    if let Ok(ufs::State::Clean) = fs.state() {
        let flags = fs.flags();
        println!("ramdisk mounted successfully (Clean, {flags:?})");
        Ok(Some(fs))
    } else {
        println!("ramdisk mount failed: invalid state {:?}", fs.state());
        Err(Error::FsInvState)
    }
}

pub fn list(fs: &ufs::FileSystem<'_>, path: &str) -> Result<()> {
    let path = path.as_bytes();
    let inode = fs.namei(path)?;
    if inode.file_type() == ufs::FileType::Dir {
        lsdir(fs, &ufs::Directory::new(&inode));
    } else {
        lsfile(&inode, path);
    }
    Ok(())
}

fn lsdir(fs: &ufs::FileSystem<'_>, dir: &ufs::Directory<'_>) {
    for dentry in dir.iter() {
        let ino = dentry.ino();
        match fs.inode(ino) {
            Ok(file) => lsfile(&file, dentry.name()),
            Err(e) => println!("ls: failed dir ent for ino #{ino}: {e:?}"),
        }
    }
}

fn lsfile(file: &ufs::Inode<'_>, name: &[u8]) {
    println!(
        "#{ino:<4} {mode:?} {nlink:<2} {uid:<3} {gid:<3} {size:>8} {name}",
        mode = file.mode(),
        ino = file.ino(),
        nlink = file.nlink(),
        uid = file.uid(),
        gid = file.gid(),
        size = file.size(),
        name = unsafe { core::str::from_utf8_unchecked(name) }
    );
}

pub fn cat(
    uart: &mut Uart,
    fs: &ufs::FileSystem<'_>,
    path: &str,
) -> Result<()> {
    let path = path.as_bytes();
    let file = fs.namei(path)?;
    if file.file_type() != ufs::FileType::Regular {
        println!("cat: not a regular file");
        return Err(Error::BadArgs);
    }
    let mut offset = 0;
    let size = file.size();
    while offset != size {
        let mut buf = [0u8; 1024];
        let nb = file.read(offset as u64, &mut buf)?;
        uart.putbs_crnl(&buf[..nb]);
        offset += nb;
    }
    Ok(())
}

pub fn copy(
    fs: &ufs::FileSystem<'_>,
    path: &str,
    dst: &mut [u8],
) -> Result<usize> {
    let path = path.as_bytes();
    let file = fs.namei(path)?;
    if file.file_type() != ufs::FileType::Regular {
        println!("copy: not a regular file");
        return Err(Error::BadArgs);
    }
    let len = core::cmp::min(file.size(), dst.len());
    let nb = file.read(0, &mut dst[..len])?;
    Ok(nb)
}

pub fn sha256(fs: &ufs::FileSystem<'_>, path: &str) -> Result<[u8; 32]> {
    use sha2::{Digest, Sha256};

    let path = path.as_bytes();
    let file = fs.namei(path)?;
    if file.file_type() != ufs::FileType::Regular {
        println!("sha256: can only sum regular files");
        return Err(Error::BadArgs);
    }
    let mut sum = Sha256::new();
    let mut offset = 0;
    let size = file.size();
    while offset != size {
        let mut buf = [0u8; 1024];
        let nb = file.read(offset as u64, &mut buf)?;
        sum.update(&buf[..nb]);
        offset += nb;
    }
    let hash = sum.finalize();
    Ok(hash.into())
}

pub fn open<'a>(
    fs: &'a ufs::FileSystem<'a>,
    path: &str,
) -> Result<ufs::Inode<'a>> {
    let path = path.as_bytes();
    fs.namei(path)
}
