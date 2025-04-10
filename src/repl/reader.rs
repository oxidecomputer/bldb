// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::cons;
use crate::println;
use crate::repl::Value;
use crate::result::{Error, Result};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

#[derive(Clone, Debug)]
pub enum Token {
    Push,
    Swap,
    Term,
    Value(Value),
}

#[derive(Clone)]
pub enum Command {
    Push,
    Swap,
    Cmd(String, Vec<Token>),
}

impl fmt::Debug for Command {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> core::result::Result<(), fmt::Error> {
        match self {
            Self::Push => write!(f, "Push"),
            Self::Swap => write!(f, "Swap"),
            Self::Cmd(cmd, _) => write!(f, "{cmd}"),
        }
    }
}

pub(super) fn parse_num<T: Default + TryFrom<u128>>(num: &str) -> Result<T> {
    let num = num.bytes().filter(|&b| b != b'_').collect::<Vec<_>>();
    let num = unsafe { core::str::from_utf8_unchecked(&num) };
    let (radix, numstr) = match num {
        "0" => return Ok(T::default()),
        s if s.starts_with("0x") || s.starts_with("0X") => (16, &s[2..]),
        s if s.starts_with("0t") || s.starts_with("0T") => (10, &s[2..]),
        s if s.starts_with("0b") || s.starts_with("0B") => (2, &s[2..]),
        s if s.starts_with("0") => (8, &s[0..]),
        s => (10, s),
    };
    let num =
        u128::from_str_radix(numstr, radix).map_err(|_| Error::NumParse)?;
    T::try_from(num).map_err(|_| Error::NumRange)
}

fn parse_len<T: Default + TryFrom<u128>>(mut tok: &str) -> Result<T> {
    let mut multiplier: u128 = 1;
    while !tok.is_empty() {
        if let Some(rest) = tok.strip_suffix(['k', 'K']) {
            multiplier *= 1024;
            tok = rest;
            continue;
        }
        if let Some(rest) = tok.strip_suffix(['m', 'M']) {
            multiplier *= 1024 * 1024;
            tok = rest;
            continue;
        }
        if let Some(rest) = tok.strip_suffix(['g', 'G']) {
            multiplier *= 1024 * 1024 * 1024;
            tok = rest;
            continue;
        }
        break;
    }
    let num = if tok.is_empty() { 1 } else { parse_num(tok)? };
    let num = multiplier.checked_mul(num).ok_or(Error::NumRange)?;
    T::try_from(num).map_err(|_| Error::NumRange)
}

fn split_pair(s: &str, pat: char) -> Result<(&str, Option<&str>)> {
    let mut it = s.split(pat);
    let (Some(a), b, None) = (it.next(), it.next(), it.next()) else {
        return Err(Error::BadArgs);
    };
    Ok((a, b))
}

fn eval_reader_command(
    config: &mut bldb::Config,
    cmd: &str,
    env: &mut Vec<Value>,
    lastval: &Value,
) -> bool {
    match cmd {
        "clear" => cons::clear(&mut config.cons),
        "config" => println!("{config:#x?}"),
        "result" | "res" => println!("{lastval:?}"),
        "env" | "stack" => dumpenv(env),
        "clrenv" => env.clear(),
        "help" | "man" => help(),
        _ => return false,
    }
    true
}

fn dumpenv(env: &[Value]) {
    println!("environment:");
    if !env.is_empty() {
        for (k, val) in env.iter().rev().enumerate() {
            println!("[{k}]: {val:?}");
        }
    } else {
        println!("(empty)");
    }
}

fn parse_value(s: &str) -> Result<Value> {
    let v = match s.chars().next() {
        Some(c) if c.is_ascii_digit() && !s.contains('/') => {
            let (a, b) = split_pair(s, ',')?;
            if let Some(b) = b {
                Value::Pair(parse_num(a)?, parse_len(b)?)
            } else {
                Value::Unsigned(parse_num(a)?)
            }
        }
        Some(_) => Value::Str(String::from(s)),
        _ => Value::Nil,
    };
    Ok(v)
}

pub fn read(
    config: &mut bldb::Config,
    env: &mut Vec<Value>,
    lastval: &Value,
) -> Result<Vec<Command>> {
    let mut buf = [0u8; 1024];
    let line = loop {
        let Ok(line) = cons::readline("@", &mut config.cons, &mut buf) else {
            return Err(Error::Reader);
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if eval_reader_command(config, line, env, lastval) {
            continue;
        }
        break line;
    };
    let mut cmds = Vec::<Command>::new();
    let cs: Box<dyn Iterator<Item = &str>> = if line.contains('|') {
        Box::new(line.split('|').rev())
    } else {
        Box::new(line.split('.'))
    };
    for cmd in cs {
        let mut cmd = cmd.trim();
        let cmdline = String::from(cmd);
        while !cmd.is_empty() {
            if let Some(rest) = cmd.strip_prefix("@") {
                cmds.push(Command::Push);
                cmd = rest.trim();
                continue;
            }
            if let Some(rest) = cmd.strip_prefix("#") {
                cmds.push(Command::Swap);
                cmd = rest.trim();
                continue;
            }
            break;
        }
        let mut tokens = Vec::<Token>::new();
        for mut tok in cmd.split_ascii_whitespace() {
            while !tok.is_empty() {
                if let Some(rest) = tok.strip_prefix("@") {
                    tokens.push(Token::Push);
                    tok = rest.trim();
                    continue;
                }
                if let Some(rest) = tok.strip_prefix("#") {
                    tokens.push(Token::Swap);
                    tok = rest.trim();
                    continue;
                }
                if let Some(rest) = tok.strip_prefix("$") {
                    tokens.push(Token::Term);
                    tok = rest.trim();
                    continue;
                }
                tokens.push(Token::Value(parse_value(tok)?));
                break;
            }
        }
        if !tokens.is_empty() {
            if let Token::Value(Value::Str(cmd)) = tokens[0].clone() {
                tokens[0] = Token::Value(Value::Cmd(cmd));
            }
        }
        cmds.push(Command::Cmd(cmdline, tokens));
    }
    Ok(cmds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_len_suffix() {
        assert_eq!(1024_usize, parse_len("k").unwrap());
        assert_eq!(4096_usize, parse_len("4K").unwrap());
    }

    #[test]
    fn parse_value_tests() {
        assert!(matches!(parse_value("").unwrap(), Value::Nil));
        assert!(matches!(
            parse_value("0x1000,4k").unwrap(),
            Value::Pair(0x1000, 4096)
        ));
    }
}

fn help() {
    println!(
        r#"
## Basic Usage

You are at the `bldb` REPL, where you type commands to the
loader/debugger.  Those commands can be chained, in a manner
similar to chaining function calls.  If one wanted the effect
`f(g(h(x)))`, then as in Haskell one may write `f . g . h x`.

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

To send a compressed ramdisk, inflate it, mount it, load a
kernel from it, and call into that kernel, passing the ramdisk
base address and length as arguments, run:

```
call . load /platform/oxide/kernel/amd64/unix . mount . @inflate . rz
```

And then send your compressed ramdisk image using ZMODEM.  For
example, via `sz -w 1024 -b ramdisk.ufs.z`.

If you prefer the more traditional "pipe" syntax using `|`
characters, you may use that instead.  The above example is
equivalent to:

```
rz | @inflate | mount | load /platform/oxide/kernel/amd64/unix | call
```

## Commands

The reader supports a handful of "reader commands":

* `clear` clears the terminal window
* `config` displays the current system configuration
* `env` or `stack` displays the current environment stack
* `clrenv` clears the environment stack
* `res` or `result` displays the last returned value
* `help` or `man` displays this text

Supported commands include:

* `push item(s)` to push one or more items onto the environment
  stack.
* `pop` to pop and return the item currently at the top of the
  environment stack.  Returns nil if the stack is empty.
* `rz <addr,len>` to receive a file via ZMODEM
* `rx <addr,len>` to receive a file via XMODEM
* `inflate <src addr>,<src len> [<dst addr>,<dst len>]`
  decompresses the a ZLIB compressed slice from the given
  source to the given destination.
* `mount <addr,len>` to mount the UFS ramdisk
* `ls <file>` to list a file or directory on the ramdisk
* `cat <file>` to display the contents of a file
* `copy <file> <dst addr>,<dst len>` to copy the contents of a
  file to a region of memory.
* `elfinfo <file>` to read the contents of the ELF header and
  segment headers of an ELF file
* `load <file>` to load the given ELF file and retrieve its
  entry point
* `loadmem <addr>,<len>` to load an ELF object from the given
  region of memory.
* `call <location> [<up to 6 args>]` calls the System V ABI
  compliant function at `<location>`, passing up to six
  arguments taken from the environment stack argument list
  terminated by nil.
* `rdmsr <u32>` to read the numbered MSR (note some MSRs can be
  specified by name, such as `IA32_APIC_BASE`)
* `wrmsr <u32> <u64>` to write the given value to the given MSR
* `jfmt <num>` to format a number using the "jazzy" format from
  the illumos `mdb` debugger
* `sha256 <file>` to compute the SHA256 checksum of a file in
  the ramdisk
* `sha256mem <addr,len>` to compute the SHA256 checksum over a
  region of memory
* `inb <port>`, `inw <port>`, `inl <port>` to read data from an
  x86 IO port
* `outb <port> <u8>`, `outw <port> <u16>`, `outl <port> <u32>`
  to write data to an x86 IO port
* `iomuxget <pin>` to get the function currently active in the
  IO mux for the given pin
* `iomuxset <pin> <function>` to configure the IO mux for the
  given pin to the given function, where `<function>` is one of,
  `F0`, `F1`, `F2`, or `F3`
* `gpioget pin` to get the state of the given GPIO pin
* `gpioset pin <state>` to set the given GPIO pin to the given
  state, which includes:
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
  given address, if any
* `mappings` to display all virtual memory mappings
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
* `getbits <start>,<end> <value>` returns  the given bit range
  from `<value>`
* `setbits <start>,<end> <new bits> <value>` sets the given bit
  range in `<value>` to `<new bits>`
* `spinner` displays a moving "spinner" on the terminal until a
  byte is received on the UART.
"#
    );
}
