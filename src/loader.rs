// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Parses and loads a binary image, represented as a slice of
//! bytes, into its executable form in RAM.

extern crate alloc;

use crate::io::Read;
use crate::mem;
use crate::mmu::LoaderPageTable;
use crate::println;
use crate::ramdisk::File;
use crate::result::{Error, Result};
use alloc::vec::Vec;
use goblin::container::{Container, Ctx, Endian};
use goblin::elf::ProgramHeader;
use goblin::elf::program_header::PT_LOAD;
use goblin::elf::{self, Elf};

const PAGE_SIZE: usize = 4096;

/// Loads an executable image contained in the given file
/// creating virtual mappings as required.  Returns the image's
/// ELF entry point on success.
pub(crate) fn load(
    page_table: &mut LoaderPageTable,
    file: &dyn File,
) -> Result<u64> {
    let mut buf = [0u8; PAGE_SIZE];
    file.read(0, &mut buf).map_err(|_| Error::FsRead)?;
    let elf = parse_elf(&buf)?;
    for segment in elf.program_headers.iter().filter(|&h| h.p_type == PT_LOAD) {
        let file_range = segment.file_range();
        if file.size() < file_range.end {
            return Err(Error::ElfTruncatedObj);
        }
        load_segment(page_table, segment, file)?;
    }
    crate::println!("Loaded ELF file: entry point {:#x?}", elf.entry);
    Ok(elf.entry)
}

/// Loads an executable image contained in the given byte slice,
/// creating virtual mappings as required.  Returns the image's
/// ELF entry point on success.
pub(crate) fn load_bytes(
    page_table: &mut LoaderPageTable,
    bytes: &[u8],
) -> Result<u64> {
    let elf = parse_elf(bytes)?;
    for section in elf.program_headers.iter().filter(|&h| h.p_type == PT_LOAD) {
        let file_range = section.file_range();
        if bytes.len() < file_range.end {
            return Err(Error::ElfTruncatedObj);
        }
        load_segment(page_table, section, &bytes)?;
    }
    crate::println!(
        "Loaded ELF object from memory: entry point {:#x?}",
        elf.entry
    );
    Ok(elf.entry)
}

pub(crate) fn elfinfo(file: &dyn File) -> Result<()> {
    use goblin::elf;

    let mut buf = [0u8; PAGE_SIZE];
    file.read(0, &mut buf).map_err(|_| Error::FsRead)?;
    let elf = parse_elf(&buf)?;
    println!("ELF header (version {}):", elf.header.e_version);
    println!(
        "Class: {:?}\tObject type: {}\tMachine: {}\tEndian: {:?}",
        elf.header.container().map_err(|_| Error::ElfClass)?,
        elf::header::et_to_str(elf.header.e_type),
        elf::header::machine_to_str(elf.header.e_machine),
        elf.header.endianness().map_err(|_| Error::ElfEndian)?,
    );
    println!("entry point {:#x?}", elf.header.e_entry);
    for (k, segment) in elf.program_headers.iter().enumerate() {
        println!(
            concat!(
                "HDR[{}]: ",
                "TYPE: {}, ",
                "FILE: {:?}, ",
                "VIRT: {:#x?}, ",
                "PHYS: {:#x?}, ",
                "ALIGN: {}, ",
                "PERMS: {}{}{}",
            ),
            k,
            elf::program_header::pt_to_str(segment.p_type),
            segment.file_range(),
            segment.vm_range(),
            segment.p_paddr,
            segment.p_align,
            if segment.is_read() { 'R' } else { '-' },
            if segment.is_write() { 'W' } else { '-' },
            if segment.is_executable() { 'X' } else { '-' },
        );
    }
    Ok(())
}

/// Parses the ELF executable contained in the given byte slice.
fn parse_elf(bytes: &[u8]) -> Result<Elf> {
    let header = parse_header(bytes)?;
    let mut elf = Elf::lazy_parse(header).map_err(|_| Error::ElfParseObject)?;
    elf.program_headers = parse_program_headers(bytes, header)?;
    Ok(elf)
}

/// Parses and validates the ELF header from the given byte
/// slice.  Note that much of the heavy lifting of validating
/// the ELF header is done by the parsing library.
fn parse_header(bytes: &[u8]) -> Result<elf::Header> {
    let binary = Elf::parse_header(bytes).map_err(|_| Error::ElfParseHeader)?;
    if binary.e_machine != elf::header::EM_X86_64 {
        return Err(Error::ElfArch);
    }
    let container = binary.container().map_err(|_| Error::ElfClass)?;
    if container != Container::Big {
        return Err(Error::ElfContainer64);
    }
    let endian = binary.endianness().map_err(|_| Error::ElfEndian)?;
    if endian != Endian::Little {
        return Err(Error::ElfLEndian);
    }
    if binary.e_type != elf::header::ET_EXEC {
        return Err(Error::ElfExec);
    }
    if binary.e_entry == 0 {
        return Err(Error::ElfZero);
    }
    if binary.e_ident[elf::header::EI_VERSION] != elf::header::EV_CURRENT
        || binary.e_version != elf::header::EV_CURRENT.into()
    {
        return Err(Error::ElfVersion);
    }
    // Apparently, illumos uses the 'ELFOSABI_SOLARIS' ABI type
    // for the kernel.  Ignore this for now.
    // if binary.e_ident[elf::header::EI_OSABI] != elf::header::ELFOSABI_NONE {
    //     return Err("ELF: bad image ABI (is not NONE)");
    // }
    Ok(binary)
}

/// Parses the ELF program headers in the contained given image
/// and header.  Separated from parsing the rest of the image
/// as we want to avoid excessive allocations for things that we
/// do not use, such as the symbol and strings tables.
fn parse_program_headers(
    bytes: &[u8],
    header: elf::Header,
) -> Result<Vec<ProgramHeader>> {
    let container = header.container().map_err(|_| Error::ElfContainer)?;
    let endian = header.endianness().map_err(|_| Error::ElfEndian)?;
    let ctx = Ctx::new(container, endian);
    ProgramHeader::parse(
        bytes,
        header.e_phoff as usize,
        header.e_phnum as usize,
        ctx,
    )
    .map_err(|_| Error::ElfParsePHeader)
}

/// Loads the given ELF segment, creating virtual mappings for
/// it as required.
fn load_segment<T: Read + ?Sized>(
    page_table: &mut LoaderPageTable,
    segment: &ProgramHeader,
    file: &T,
) -> Result<()> {
    let pa = segment.p_paddr;
    if pa % mem::P4KA::ALIGN != 0 {
        return Err(Error::ElfSegPAlign);
    }
    let vm = segment.vm_range();
    if vm.contains(&mem::LOW_CANON_SUP) || vm.contains(&mem::HI_CANON_INF) {
        return Err(Error::ElfSegNonCanon);
    }
    if vm.start % mem::V4KA::ALIGN != 0 {
        return Err(Error::ElfSegVAlign);
    }
    if vm.end <= vm.start {
        return Err(Error::ElfSegEmpty);
    }
    let start = mem::V4KA::new(vm.start);
    let end = mem::V4KA::new(mem::round_up_4k(vm.end));
    let region = start..end;
    let pa = mem::P4KA::new(pa);
    {
        let dst = unsafe {
            page_table.map_ram(region.clone(), mem::Attrs::new_data(), pa)?;
            let p = page_table.try_with_addr(start.addr())?;
            let len = end.addr() - start.addr();
            core::ptr::write_bytes(p, 0, len);
            core::slice::from_raw_parts_mut(p, len)
        };
        let filesz = segment.p_filesz as usize;
        let len = usize::min(filesz, dst.len());
        if len > 0 && file.read(segment.p_offset, &mut dst[..len])? != len {
            return Err(Error::ElfTruncatedObj);
        }
    }
    let attrs = mem::Attrs::new_kernel(
        segment.is_read(),
        segment.is_write(),
        segment.is_executable(),
    );
    unsafe {
        page_table.map_ram(region, attrs, pa)?;
    }
    Ok(())
}
