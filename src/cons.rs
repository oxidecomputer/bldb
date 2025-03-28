// Copyright 2024  The Hypatia Authors
// All rights reserved
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use crate::result::{Error, Result};
use crate::uart::Uart;

pub fn readline<'a>(
    prompt: &str,
    uart: &mut Uart,
    line: &'a mut [u8],
) -> Result<&'a str> {
    const BS: u8 = 8;
    const TAB: u8 = 9;
    const NL: u8 = 10;
    const CR: u8 = 13;
    const CTLU: u8 = 21;
    const CTLW: u8 = 23;
    const DEL: u8 = 127;

    fn find_prev_col(line: &[u8], start: usize) -> usize {
        line.iter()
            .fold(start, |v, &b| v + if b == TAB { 8 - (v & 0b111) } else { 1 })
    }

    fn backspace(
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
            uart.putb(BS);
            if overstrike {
                uart.putb(b' ');
                uart.putb(BS);
            }
        }
        (pcol, line.len() - 1)
    }

    fn isword(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'_'
    }

    if line.is_empty() {
        return Ok("");
    }

    let start = prompt.len();
    uart.puts(prompt);

    let mut k = 0;
    let mut col = start;
    while k < line.len() {
        match uart.getb() {
            CR | NL => {
                uart.putb(CR);
                uart.putb(NL);
                break;
            }
            BS | DEL => {
                if k > 0 {
                    (col, k) = backspace(uart, &line[..k], start, col);
                }
            }
            CTLU => {
                while k > 0 {
                    (col, k) = backspace(uart, &line[..k], start, col);
                }
            }
            CTLW => {
                while k > 0 && line[k - 1].is_ascii_whitespace() {
                    (col, k) = backspace(uart, &line[..k], start, col);
                }
                if k > 0 {
                    let cond = isword(line[k - 1]);
                    while k > 0
                        && !line[k - 1].is_ascii_whitespace()
                        && isword(line[k - 1]) == cond
                    {
                        (col, k) = backspace(uart, &line[..k], start, col);
                    }
                }
            }
            TAB => {
                line[k] = TAB;
                k += 1;
                let ncol = (8 + col) & !0b111;
                for _ in col..ncol {
                    uart.putb(b' ');
                }
                col = ncol;
            }
            b => {
                line[k] = b;
                k += 1;
                uart.putb(b);
                col += 1;
            }
        }
    }

    core::str::from_utf8(&line[..k]).map_err(|_| Error::Utf8)
}

pub fn clear(term: &mut Uart) {
    const ESC: u8 = 27;
    term.putb(ESC);
    term.puts("[H");
    term.putb(ESC);
    term.puts("[2J");
}
