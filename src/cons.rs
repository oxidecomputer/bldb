// Copyright 2024  The Hypatia Authors
// All rights reserved
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use crate::result::{Error, Result};
use crate::uart::Uart;
use core::time::Duration;

#[derive(Debug, Eq, PartialEq)]
pub enum Prompt {
    Tenex,
    Spinner,
    Pulser,
}

const BS: u8 = 8;
const TAB: u8 = 9;
const NL: u8 = 10;
const CR: u8 = 13;
const CTLU: u8 = 21;
const CTLW: u8 = 23;
const ESC: u8 = 27;
const DEL: u8 = 127;

pub fn readline<'a, F>(
    prompt: F,
    uart: &mut Uart,
    line: &'a mut [u8],
) -> Result<&'a str>
where
    F: FnOnce(&mut Uart) -> usize,
{
    readline_timeout(prompt, uart, Duration::ZERO, line)
}

pub fn readline_timeout<'a, F>(
    prompt: F,
    uart: &mut Uart,
    timeout: Duration,
    line: &'a mut [u8],
) -> Result<&'a str>
where
    F: FnOnce(&mut Uart) -> usize,
{
    fn find_prev_col(line: &[u8], start: usize) -> usize {
        line.iter()
            .fold(start, |v, &b| v + if b == TAB { 8 - (v & 0b111) } else { 1 })
    }

    fn backup(
        uart: &mut Uart,
        line: &[u8],
        start: usize,
        col: usize,
    ) -> (usize, usize) {
        if line.is_empty() || col == start {
            return (start, 0);
        }
        let (pcol, overstrike) = match line.last() {
            Some(&b' ') => (col - 1, false),
            Some(&b'\t') => {
                (find_prev_col(&line[..line.len() - 1], start), false)
            }
            _ => (col - 1, true),
        };
        for _ in pcol..col {
            backspace(uart, overstrike);
        }
        (pcol, line.len() - 1)
    }

    fn isword(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'_'
    }

    if line.is_empty() {
        return Ok("");
    }

    let start = prompt(uart);

    let mut k = 0;
    let mut col = start;
    while k < line.len() {
        match uart.getb_timeout(timeout) {
            None => {
                if k == 0 {
                    return Err(Error::Timeout);
                }
            }
            Some(CR | NL) => {
                uart.putb(CR);
                uart.putb(NL);
                break;
            }
            Some(BS | DEL) => {
                if k > 0 {
                    (col, k) = backup(uart, &line[..k], start, col);
                }
            }
            Some(CTLU) => {
                while k > 0 {
                    (col, k) = backup(uart, &line[..k], start, col);
                }
            }
            Some(CTLW) => {
                while k > 0 && line[k - 1].is_ascii_whitespace() {
                    (col, k) = backup(uart, &line[..k], start, col);
                }
                if k > 0 {
                    let cond = isword(line[k - 1]);
                    while k > 0
                        && !line[k - 1].is_ascii_whitespace()
                        && isword(line[k - 1]) == cond
                    {
                        (col, k) = backup(uart, &line[..k], start, col);
                    }
                }
            }
            Some(TAB) => {
                line[k] = TAB;
                k += 1;
                let ncol = (8 + col) & !0b111;
                for _ in col..ncol {
                    uart.putb(b' ');
                }
                col = ncol;
            }
            Some(b) => {
                line[k] = b;
                k += 1;
                uart.putb(b);
                col += 1;
            }
        }
    }

    core::str::from_utf8(&line[..k]).map_err(|_| Error::Utf8)
}

pub fn backspace(term: &mut Uart, overstrike: bool) {
    term.putb(BS);
    if overstrike {
        term.putb(b' ');
        term.putb(BS);
    }
}

pub fn clear(term: &mut Uart) {
    term.putb(ESC);
    term.puts("[H");
    term.putb(ESC);
    term.puts("[2J");
}

pub fn cycle(
    term: &mut Uart,
    prefix: &[u8],
    cycle: &[u8],
    suffix: &[u8],
    wait: Duration,
) {
    fn erase(term: &mut Uart, bs: &[u8]) {
        for &b in bs.iter().rev() {
            backspace(term, b != b' ');
        }
    }
    let _ = term.putbs(prefix);
    for &b in cycle.iter().cycle() {
        term.putb(b);
        let _ = term.putbs(suffix);
        match term.wait_data_ready(wait) {
            Ok(true) | Err(_) => break,
            _ => {}
        }
        erase(term, suffix);
        erase(term, &[b]);
    }
    erase(term, suffix);
    erase(term, &[0]);
    erase(term, prefix);
}

#[cfg(all(feature = "pulse_prompt", feature = "spin_prompt"))]
compile_error!(
    "The `pulse_prompt` and `spin_prompt` features are mutually exclusive"
);

#[cfg(not(any(feature = "pulse_prompt", feature = "spin_prompt")))]
pub(crate) const DEFAULT_PROMPT: Prompt = Prompt::Tenex;
#[cfg(feature = "pulse_prompt")]
pub(crate) const DEFAULT_PROMPT: Prompt = Prompt::Pulser;
#[cfg(feature = "spin_prompt")]
pub(crate) const DEFAULT_PROMPT: Prompt = Prompt::Spinner;
