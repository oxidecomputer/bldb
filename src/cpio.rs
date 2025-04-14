// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! cpio miniroot support.

use crate::io;
use crate::ramdisk;
use crate::result::{Error, Result};
use crate::{print, println};
use alloc::boxed::Box;

pub(crate) struct FileSystem {
    sd: io::Sd,
}

impl FileSystem {
    pub(crate) fn try_new(bs: &[u8]) -> Result<FileSystem> {
        if bs.starts_with(b"070707") {
            let sd = unsafe { io::Sd::from_slice(bs) };
            Ok(FileSystem { sd })
        } else {
            Err(Error::FsInvMagic)
        }
    }
}

pub(crate) struct File {
    data: io::Sd,
}

impl ramdisk::File for File {
    fn file_type(&self) -> ramdisk::FileType {
        ramdisk::FileType::Regular
    }
}

impl io::Read for File {
    fn read(&self, offset: u64, dst: &mut [u8]) -> Result<usize> {
        let s = unsafe { self.data.as_slice() };
        s.read(offset, dst)
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}

impl ramdisk::FileSystem for FileSystem {
    fn open(&self, path: &str) -> Result<Box<dyn ramdisk::File>> {
        let cpio = unsafe { self.sd.as_slice() };
        let key = path.strip_prefix("/").unwrap_or(path);
        for file in cpio_reader::iter_files(cpio) {
            if file.name() == key {
                let data = unsafe { io::Sd::from_slice(file.file()) };
                return Ok(Box::new(File { data }));
            }
        }
        Err(Error::FsNoFile)
    }

    fn list(&self, path: &str) -> Result<()> {
        let cpio = unsafe { self.sd.as_slice() };
        let key = path.strip_prefix('/').unwrap_or(path);
        for file in cpio_reader::iter_files(cpio) {
            if file.name() == key {
                lsfile(path, &file);
                return Ok(());
            }
        }
        let mut found = false;
        for file in cpio_reader::iter_files(cpio) {
            if file.name().starts_with(key) {
                lsfile(file.name(), &file);
                found = true;
            }
        }
        if found { Ok(()) } else { Err(Error::FsNoFile) }
    }

    fn as_str(&self) -> &str {
        "cpio"
    }
}

fn lsfile(path: &str, file: &cpio_reader::Entry) {
    print!("#{ino:<4} ", ino = file.ino());
    print_mode(file.mode());
    println!(
        " {nlink:<2} {uid:<3} {gid:<3} {size:>8} {path}",
        nlink = file.nlink(),
        uid = file.uid(),
        gid = file.gid(),
        size = file.file().len(),
    );
}

fn first_char(mode: cpio_reader::Mode) -> char {
    use cpio_reader::Mode;
    match mode {
        _ if mode.contains(Mode::DIRECTORY) => 'd',
        _ if mode.contains(Mode::CHARACTER_SPECIAL_DEVICE) => 'c',
        _ if mode.contains(Mode::BLOCK_SPECIAL_DEVICE) => 'b',
        _ if mode.contains(Mode::SYMBOLIK_LINK) => 'l',
        _ if mode.contains(Mode::SOCKET) => 's',
        _ if mode.contains(Mode::NAMED_PIPE_FIFO) => 'f',
        _ => '-',
    }
}

fn print_mode(mode: cpio_reader::Mode) {
    use cpio_reader::Mode;
    print!("{}", first_char(mode));
    let alt = |bit, t, f| {
        if mode.contains(bit) { t } else { f }
    };
    // For some reason, the cpio reader library appears to have
    // the meaning of these bits mirrored with respect to the owner
    // bits.
    print!("{}", alt(Mode::WORLD_READABLE, 'r', '-'));
    print!("{}", alt(Mode::WORLD_WRITABLE, 'w', '-'));
    if !mode.contains(Mode::SUID) {
        print!("{}", alt(Mode::WORLD_EXECUTABLE, 'x', '-'));
    } else {
        print!("{}", alt(Mode::WORLD_EXECUTABLE, 's', 'S'));
    }

    print!("{}", alt(Mode::GROUP_READABLE, 'r', '-'));
    print!("{}", alt(Mode::GROUP_WRITABLE, 'w', '-'));
    if !mode.contains(Mode::SGID) {
        print!("{}", alt(Mode::GROUP_EXECUTABLE, 'x', '-'));
    } else {
        print!("{}", alt(Mode::GROUP_EXECUTABLE, 's', 'S'));
    }

    print!("{}", alt(Mode::USER_READABLE, 'r', '-'));
    print!("{}", alt(Mode::USER_WRITABLE, 'w', '-'));
    if !mode.contains(Mode::STICKY) {
        print!("{}", alt(Mode::USER_EXECUTABLE, 'x', '-'));
    } else {
        print!("{}", alt(Mode::USER_EXECUTABLE, 't', 'T'));
    }
}
