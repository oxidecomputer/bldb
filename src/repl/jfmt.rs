// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use crate::{print, println};
use alloc::{vec, vec::Vec};

const PREFIX: &str = "                ";

fn puts(s: &[char], suffix: &str) {
    print!("{PREFIX}");
    for &c in s {
        print!("{c}");
    }
    print!("{suffix}");
}

fn putsln(s: &[char]) {
    puts(s, "\n");
}

pub fn jfmt(num: u128) {
    let n = 128 - num.leading_zeros() as usize;

    let mut v = Vec::new();
    let mut ones = Vec::new();
    for k in 0..n {
        let bit = (num >> k) & 0b1 == 0b1;
        v.push(bit);
        if bit {
            ones.push(k);
        }
    }
    v.reverse();

    let mut cs = vec![' '; n];

    println!("{PREFIX}{num:b}");
    for k in 0..v.len() {
        cs[k] = if v[k] { '▴' } else { ' ' };
    }
    putsln(&cs);

    let max1 = ones.iter().last().map_or(0, |&l| l);
    let bit_width = max1.checked_ilog10().unwrap_or(0) as usize + 1;
    let mask_width = (max1 + 4) / 4;
    for this1 in ones.into_iter() {
        let off = n - 1 - this1;
        for (k, &b) in v.iter().enumerate() {
            cs[k] = match (k, b) {
                (k, true) if k < off => '│',
                (k, _) if k < off => ' ',
                (k, _) if k == off => '╰',
                _ => '─',
            };
        }
        puts(&cs, "── ");
        println!(
            "bit {this1:bit_width$} mask 0x{mask:0>mask_width$x}",
            mask = 1u128 << this1
        );
    }

    println!();
    println!("{PREFIX}hex: {num:#x}");
    println!("{PREFIX}dec: {num}");
    println!("{PREFIX}oct: {num:#o}");
}

pub fn run(_config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let value = match repl::popenv(env) {
        Value::Unsigned(value) => value,
        _ => {
            println!("usage: jfmt <number>");
            return Err(Error::BadArgs);
        }
    };
    jfmt(value);
    Ok(Value::Nil)
}
