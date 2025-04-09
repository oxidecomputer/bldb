// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Simple hex dump routine.

use crate::bldb;
use crate::println;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use alloc::vec::Vec;
use bit_field::BitField;
use core::ops::Range;

pub fn get(_config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: getbits <start>,<end> <value>");
        error
    };
    let (start, end) =
        repl::popenv(env).as_pair().and_then(check_bits_pair).map_err(usage)?;
    let value = repl::popenv(env).as_num::<u128>().map_err(usage)?;
    let bits = value.get_bits(start..end);
    Ok(Value::Unsigned(bits))
}
pub fn set(_config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: setbits <start>,<end> <replace> <value>");
        error
    };
    let (start, end) =
        repl::popenv(env).as_pair().and_then(check_bits_pair).map_err(usage)?;
    let new_bits = repl::popenv(env).as_num::<u128>().map_err(usage)?;
    let mut value = repl::popenv(env).as_num::<u128>().map_err(usage)?;
    if !value_fits(start..end, new_bits) {
        return Err(Error::NumRange);
    }
    value.set_bits(start..end, new_bits);
    Ok(Value::Unsigned(value))
}

fn check_bits_pair(pair: (u64, usize)) -> Result<(usize, usize)> {
    let start = pair.0 as usize;
    let end = pair.1;
    if start == end || start > 128 || end > 128 {
        return Err(Error::NumRange);
    }
    Ok((start.min(end), end.max(start)))
}

fn value_fits(bits: Range<usize>, value: u128) -> bool {
    let nbits = bits.end - bits.start;
    assert_ne!(nbits, 0);
    value <= (!0u128 >> (128 - nbits))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn value_does_fit() {
        assert!(value_fits(1..2, 1));
        assert!(value_fits(0..128, !0));
        assert!(value_fits(0..4, 0xf));
    }

    #[test]
    fn value_doesnt_fit() {
        assert!(!value_fits(1..2, 2));
        assert!(!value_fits(0..4, 0xFF));
    }
}
