// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::println;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use alloc::vec::Vec;

/// Expands the compressed ramdisk into a dedicated RAM region and returns
/// a slice around the its contents.
fn inflate<'a>(src: &[u8], dst: &'a mut [u8]) -> Result<&'a [u8]> {
    use miniz_oxide::inflate::TINFLStatus;
    use miniz_oxide::inflate::core::DecompressorOxide;
    use miniz_oxide::inflate::core::decompress;
    use miniz_oxide::inflate::core::inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER;

    let mut r = DecompressorOxide::new();
    let flags = TINFL_FLAG_PARSE_ZLIB_HEADER;
    let (s, _, o) = decompress(&mut r, src, dst, 0, flags);
    if s != TINFLStatus::Done {
        println!("inflate failed: state is {s:?}");
        return Err(Error::SadBalloon);
    }
    Ok(&dst[..o])
}

pub fn run(config: &mut bldb::Config, env: &mut Vec<Value>) -> Result<Value> {
    let usage = |error| {
        println!("usage: inflate <src addr>,<src len> [<dst addr>,<dst len>]");
        error
    };
    let src = repl::popenv(env)
        .as_slice(&config.page_table, 0)
        .and_then(|o| o.ok_or(Error::BadArgs))
        .map_err(usage)?;
    let dst = repl::popenv(env)
        .as_slice_mut(&config.page_table, 0)
        .map_err(usage)?
        .unwrap_or_else(|| bldb::ramdisk_region_init_mut());
    let inflated = inflate(src, dst)?;
    Ok(Value::Slice(inflated))
}
