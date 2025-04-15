# Boot Loader and Debugger

`bldb` is a standalone program designed to fill the gap between
`phbl` and `kmdb`.  It is intended as a development tool, and
subsumes the functionality of the older `nanobl-rs` tool.

`bldb` runs from the x86 reset vector, but provides an
interactive interface and can also load and run a host
operating system.

It is loaded from SPI flash by the PSP, and execution starts in
16-bit real mode.  It is responsible for:

* bringing the bootstrap core up into 64-bit long mode with
  paging enabled
* providing an interactive, command-line driven interface to
  inspect and manipulate the machine's state
* Supports uploading a filesystem image over the UART
* Supports loading an ELF file (e.g., a kernel) directly from
  a RAM disk

## Basic Usage

`bldb` presents a REPL to the user.   Users type commands to the
REPL, and those commands can be chained, in a manner similar to
chaining function calls.  If one wanted the effect `f(g(h(x)))`,
then as in Haskell one may write `f . g . h x`.

Commands use an "environment stack" for arguments and to save
values (when appropriate).  The REPL will always print the value
returned by the last command.

The `@` command duplicates the value at the top of the stack and
pushes the duplicate.  The `$` command will push a `nil`.  The
`#` command swaps the two elements at the top of the stack.

To push an element onto the stack, use the `push` command.  To
pop the top element, one may use the `pop` command.  Note also
that one can use the `.` command separator and `$` to push and
pop items onto and from the environment stack.  For example,

```
$a b c
```

Pushes the strings "a", "b", and "c" onto the stack, while,

```
.$
```

will pop the top element.

## Booting a machine

Assuming you have booted into `bldb` on some machine of
interest, you will be presented with a prompt (`@`), at which
you may type commands.  To send a compressed ramdisk, inflate
it, mount it, load a kernel from it, and call into that kernel,
passing the ramdisk base address and length as arguments, run:

```
call . load /platform/oxide/kernel/amd64/unix . mount . @inflate . rz
```

And then send your compressed ramdisk image using ZMODEM.  For
example, via `sz -w 1024 -b ramdisk.ufs.z`.

This command will receive a file via ZMODEM, and push the slice
containing the received contents onto the environment stack.
It will then invoke the `inflate` command; `inflate` will pop
the slice pushed by `rz` off of the stack, and use that as the
source data to expand.  It will push the slice that it expanded
into onto the stack.  The `@` command will duplicate that, so
that now two instances of the slice containing the expanded
ramdisk are at the top of the stack.  Next, `mount` will be
invoked, which will pop a copy of the ramdisk slice and use that
to initialize the state of the filesystem; `mount` does not push
a value back onto the stack when it returns.  Next, the path to
the kernel will be pushed onto the stack, and the `load` command
will be invoked.  `load` will pop the pathname off of the stack
and load the named ELF file (in this case, `unix`).  `load` will
push the entry point from the ELF header onto the stack.
Finally, `call` will be invoked: it will pop the entry point off
the stack, and pop the base address and length of the ramdisk
that was pushed by `inflate`.  It will execute an x86 `CALL`
instruction with the ELF entry point as the target `%rip`, and
passing the address and length as the first two arguments to
that function.

If you prefer the more traditional "pipe" syntax using `|`
characters, you may use that instead.  The above example is
equivalent to:

```
rz | @inflate | mount | load /platform/oxide/kernel/amd64/unix | call
```

## Transferring with XMODEM

Note that ZMODEM hasn't been completely reliable in testing.  If
things appear to hang, try sending a BREAK.  If that fails,
there is an XMODEM fallback in `bldb`, invoked via the `rx`
command at the REPL.  XMODEM is a receiver-initiated protocol,
and to avoid a race condition between the receiver issuing the
transfer handshake and the sender invoking the XMODEM send
program (e.g., `sx`), `bldb` will wait for a single character to
arrive on the UART before starting the protocol.  This character
must be the ASCII lower-case letter 'g'; any other letter will
abort the transfer and return to the REPL.

One may use a script to automate this, such as
[`sxmodem`](https://github.com/oxidecomputer/bldb/blob/main/sxmodem)
in this repository:

```
#!/bin/ksh93
printf g
exec sx -vv -Xk "$@"
```

With this, one can transfer, inflate, mount, load, and call into Unix as:

```
call . load /platform/oxide/kernel/amd64/unix . mount . @inflate . rx
```

Or, if one prefers,

```
rx | @inflate | mount | load /platform/oxide/kernel/amd64/unix | call
```

See also the
[`rconsx` script](https://github.com/oxidecomputer/bldb/blob/main/rconsx).

## Commands

The reader supports a handful of "reader commands":

* `clear` clears the terminal window
* `config` displays the current system configuration
* `env` or `stack` displays the current environment stack
* `clrenv` clears the environment stack
* `res` or `result` displays the last returned value
* `help` or `man` displays online help text

Supported commands include:

* `push item(s)` to push one or more items onto the environment
  stack.
* `pop` to pop and return the item currently at the top of the
  environment stack.  Returns nil if the stack is empty.
* `rz <addr,len>` to receive a file via ZMODEM.
* `rx <addr,len>` to receive a file via XMODEM.
* `inflate <src addr>,<src len> [<dst addr>,<dst len>]`
  decompresses the a ZLIB compressed slice from the given
  source to the given destination.
* `mount <addr,len>` to mount a UFS ramdisk or cpio miniroot.
* `umount` to unmount the ramdisk.
* `ls <file>` to list a file or directory on the ramdisk.
* `cat <file>` to display the contents of a file.
* `copy <file> <dst addr>,<dst len>` to copy the contents of a
  file to a region of memory.
* `elfinfo <file>` to read the contents of the ELF header and
  segment headers of an ELF file.
* `load <file>` to load the given ELF file and retrieve its
  entry point.
* `loadmem <addr>,<len>` to load an ELF object from the given
  region of memory.
* `call <location> [<up to 6 args>]` calls the System V ABI
  compliant function at `<location>`, passing up to six
  arguments taken from the environment stack argument list
  terminated by nil.
* `rdmsr <u32>` to read the numbered MSR (note some MSRs can be
  specified by name, such as `IA32_APIC_BASE`).
* `wrmsr <u32> <u64>` to write the given value to the given MSR.
* `jfmt <num>` to format a number using the "jazzy" format from
  the illumos `mdb` debugger.
* `sha256 <file>` to compute the SHA256 checksum of a file in
  the ramdisk.
* `sha256mem <addr,len>` to compute the SHA256 checksum over a
  region of memory.
* `inb <port>`, `inw <port>`, `inl <port>` to read data from an
  x86 IO port.
* `outb <port> <u8>`, `outw <port> <u16>`, `outl <port> <u32>`
  to write data to an x86 IO port.
* `iomuxget <pin>` to get the function currently active in the
  IO mux for the given pin.
* `iomuxset <pin> <function>` to configure the IO mux for the
  given pin to the given function, where `<function>` is one of,
  `F0`, `F1`, `F2`, or `F3`.
* `gpioget pin` to get the state of the given GPIO pin
* `gpioset pin <state>` to set the given GPIO pin to the given
  state, which is a comma-separated list of:
  * `pu` to enable the internal pullup (`-pu` to disable)
  * `pd` to enable the internal pulldown (`-pd` to disable)
  * `ah` to configure active high
  * `al` to configure active low
  * `oh` to configure output high
  * `ol` to configure output low
  * `out` to configure as output (output enable is true)
  * `in` to configure as input (output enable is false)
* `hexdump <addr>,<len>` to produce a hexdump of `len` bytes of
  memory starting at `base`.
* `peek <addr>,<len>` to read `len` bytes starting at `addr`.
  `len` must be 1, 2, 4, 8, or 16.
* `poke <addr>,<len> <value>` to poke a value into the `len`
  bytes starting at `addr`.  `len` must be 1, 2, 4, 8, or 16.
  The value is written in native byte order.
* `mapping address` to display the page table mapping for the
  given address, if any.
* `mappings` to display all virtual memory mappings.
* `map <phys addr>,<len> <virt addr> <attrs>` maps `len` bytes
  at physical address `phys addr` to virtual address `virt addr`
  with the given attributesk, which is a comma-separated list
  of:
  * `r` to enable page read permission (the default)
  * `-r` to remove page read permission
  * `w` to enable page write permission
  * `-w` to remove page write permission (the default)
  * `x` to enable page executable permission (the default)
  * `-x` to remove page execute permission
  * `c` to enable page cachability (the default)
  * `-c` to disable page caching
  * `g` to set this page as a "global" page
  * `-g` to remove the global page attribute (the default)
  `<phys addr>`, `<len>`, `<phys addr>` must all be multiples
  of 4KiB.  If these values are also multiples of 2MiB or 1GiB,
  those size mappings will be used.  To map such a region using
  smaller page sizes, issue multiple `map` commands covering
  smaller regions to make up a contiguous whole.
* `unmap <virt addr>,<len>` to remove a virtual memory mapping
  for the range of given virtual address space covering `<len>`
  bytes starting at `<virt addr>`.  As with mapping, `<len>` and
  `<virt addr>` must both be multiples of 4KiB.  If these values
  are also multiples of 2MiB or 1GiB, those size mappings will
  be used.  To unmap such a region mapped with smaller page
  sizes, issue mulitple `unmap` calls.
* `rdsmn <addr>` to read a 32-bit word from the given SMN
  address.
* `wrsmn <addr> <value>` to write a 32-bit word to the given SMN
  address.
* `cpuid <leaf> <subleaf>` to return the results of the `CPUID`
  instruction for the given leaf and subleaf.
* `ecamrd <b/d/f> <offset>` read a 32-bit word from PCIe
  extended configuration space for the given bus/device/function
* `ecamwr <b/d/f> <offset> <value>` writes a 32-bit word to PCIe
  extended configuration space for the given bus/device/function
* `getbits <start>,<end> <value>` returns the given bit range
  from `<value>`
* `setbits <start>,<end> <new bits> <value>` sets the given bit
  range in `<value>` to `<new bits>`
* `spinner` displays a moving "spinner" on the terminal until a
  byte is received on the UART.  The `pulser` and `throbber`
  commands do essentially the same thing, with a different
  character pattern.  The `megapulser` command exists just for
  fun.
* `prompt <tenex | spinner | pulser>` to change the default
  prompt type.  `tenex` is the "@" prompt.  The other two are
  animated; see the `spinner` and `pulser` commands above.

## Building bldb

We use `cargo` and the [`xtask`][1] pattern for builds.

```
cargo xtask build
```

This generates a "Debug" binary in the file
`target/x86_64-oxide-none-elf/debug/bldb`.

**Note**: Linking `bldb` requires using the LLVM linker,
ld.lld, or the GNU linker.

The LLVM linker is available with the Rust toolchain and is the
default, as configured in `x86_64-oxide-none-elf.json`.   If
building on illumos, however, it is unlikely to be in the your
default `$PATH`.  Running, `find $HOME/.rustup -name ld.lld`
can show you where it is.

If you don't want to add the resulting directory to your
`$PATH`, or if you prefer to use the GNU linker, set the
[environment variable](
https://doc.rust-lang.org/cargo/reference/environment-variables.html)
`CARGO_TARGET_X86_64_OXIDE_NONE_ELF_LINKER` to the path to
whatever linker you would like to use.  On most Linux systems
with the GNU tools, that will be
`CARGO_TARGET_X86_64_OXIDE_NONE_ELF_LINKER=ld`.

By default, Oxide's build systems install GNU ld as `gld`

## Bldb development

Modifying `bldb` follows the typical development patterns of
most Rust programs, and we have several `cargo xtask` targets to
help with common tasks.  Typically, one might use:

* `cargo xtask test` to run unit tests
* `cargo miri test` to run tests under Miri
* `cargo xtask clippy` to run the linter
* `cargo xtask clean` to remove build artifacts and intermediate
   files
* `cargo xtask expand` to expand macros
* `cargo xtask disasm` to build the bldb image and dump a
  disassembly listing of it

`cargo check` is fully supported for e.g. editor integration,
and formatting should be kept consistent via `cargo fmt`.

Most `xtask` targets will also accept either a `--release` or
`--debug` argument to get either an optimized or debugging
build; debug is the default.  To build a release version for
production, run:

```
cargo xtask build --release
```

This will produce an optimized standalone binary named
`target/x86_64-oxide-none-elf/release/bldb`.  If one builds
a debug binary, as in the previous section, it will be named,
`target/x86_64-oxide-none-elf/debug/bldb`.

These binaries are suitable for use with the
[amd-host-image-builder][2] tool.  For example, to create an
image suitable for writing to flash on a gimlet from a debug
`bldb` binary, one may change to the `amd-host-image-builder`
repository and run:

```
cargo run -- \
    -B amd-firmware/GN/1.0.0.1 \
    -B amd-firmware/GN/1.0.0.6 \
    -c etc/milan-gimlet-b.efs.json5 \
    -r ${BLDB_REPO_ROOT}/target/x86_64-oxide-none-elf/debug/bldb \
    -o milan-gimlet-b-bldb.img
```

The resulting `milan-gimlet-b-bldb.img` is suitable for writing
into a gimlet's SPI ROM.

Changes are submitted and reviewed using the GitHub pull request
model.  CI triggered by github actions ensures that tests pass.

## TODO

More tests and more testing.

It would be nice to have better representations for interesting
device registers and similar objects.

I'm sure there are bugs.  In particular, it's possible to
violate memory safety in a number of ways (and there likely
always will be: it's up to the users to be careful with
mappings and so on).

[1]: https://github.com/matklad/cargo-xtask
[2]: https://github.com/oxidecomputer/amd-host-image-builder/
