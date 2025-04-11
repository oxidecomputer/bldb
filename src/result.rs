// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::fmt;

/// Various errors
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Error {
    UartFifoOverrun,
    UartParity,
    UartFraming,
    UartBreak,
    Timeout,
    FsInvMagic,
    FsNoRoot,
    FsInvPath,
    FsNoFile,
    FsOffset,
    FsInvState,
    FsRead,
    ElfTruncatedObj,
    ElfParseObject,
    ElfParseHeader,
    ElfParsePHeader,
    ElfSegPAlign,
    ElfSegVAlign,
    ElfSegNonCanon,
    ElfSegEmpty,
    ElfVersion,
    ElfEndian,
    ElfLEndian,
    ElfContainer,
    ElfContainer64,
    ElfArch,
    ElfClass,
    ElfExec,
    ElfZero,
    Reader,
    Utf8,
    NumParse,
    NumRange,
    NoCommand,
    BadArgs,
    Recv,
    SadBalloon,
    PtrNonCanon,
    Unmapped,
    PtrAlign,
    PageAlign,
    PtrProvenance,
    Mmu(&'static str),
}

impl Error {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UartFifoOverrun => "UART RX FIFO Overrun",
            Self::UartParity => "UART parity error",
            Self::UartFraming => "UART framing error",
            Self::UartBreak => "UART BREAK",
            Self::Timeout => "Timeout",
            Self::FsNoRoot => "No file system currently mounted",
            Self::FsInvMagic => "FFS: Bad magic number in superblock",
            Self::FsInvPath => "Invalid path",
            Self::FsNoFile => "No such file or directory",
            Self::FsOffset => "Invalid file offset (exceeds maximum)",
            Self::FsRead => "Read error",
            Self::FsInvState => "Invalid UFS filesystem state",
            Self::ElfTruncatedObj => "ELF: Object truncated",
            Self::ElfParseObject => "ELF: Failed to parse object",
            Self::ElfParseHeader => "ELF: Failed to parse ELF header",
            Self::ElfParsePHeader => "ELF: Failed to parse program header",
            Self::ElfSegPAlign => {
                "ELF: program segment is not physically 4KiB aligned"
            }
            Self::ElfSegVAlign => {
                "ELF: Program segment not virtually 4KiB aligned"
            }
            Self::ElfSegNonCanon => "ELF: Program segment is not canonical",
            Self::ElfSegEmpty => {
                "ElF: Program segment ends before start or is empty"
            }
            Self::ElfVersion => "ELF: Invalid version number",
            Self::ElfEndian => "ELF: Invalid endianness",
            Self::ElfLEndian => "ELF: Object is not little-endian",
            Self::ElfContainer => "ELF: Bad container",
            Self::ElfContainer64 => "ELF: Object is not 64-bit",
            Self::ElfArch => "ELF: Incorrect machine architecture",
            Self::ElfClass => "ELF: Invalid container class",
            Self::ElfExec => "ELF: Object not executable",
            Self::ElfZero => "ELF: Object has nil entry point",
            Self::Reader => "Reader error",
            Self::Utf8 => "UTF-8 conversion error",
            Self::NumParse => "Error parsing number from string",
            Self::NumRange => "Parsed number out of range",
            Self::NoCommand => "Unknown command",
            Self::BadArgs => "Bad command arguments",
            Self::Recv => "Receive failed",
            Self::SadBalloon => "Inflate failed",
            Self::PtrNonCanon => "Pointer is non-canonical",
            Self::Unmapped => "Memory region not mapped",
            Self::PageAlign => "Address not page aligned",
            Self::PtrAlign => "Pointer misaligned",
            Self::PtrProvenance => "Pointer has unknown provenance",
            Self::Mmu(s) => s,
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> core::result::Result<(), fmt::Error> {
        write!(f, "{}", self.as_str())
    }
}

pub type Result<T> = core::result::Result<T, Error>;
