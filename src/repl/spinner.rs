// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::repl;
use crate::uart;
use core::time::Duration;

fn run(term: &mut uart::Uart, bs: &[u8], timeout: Duration) {
    const BS: u8 = 8;
    for &b in bs.iter().cycle() {
        term.putb(b);
        if term.try_getb_timeout(timeout).is_ok() {
            break;
        }
        term.putb(BS);
    }
    term.putb(BS);
}

pub(super) fn spinner(
    config: &mut bldb::Config,
    _env: &mut [repl::Value],
) -> repl::Result<repl::Value> {
    run(&mut config.cons, b"|/-\\", Duration::from_millis(100));
    Ok(repl::Value::Nil)
}

pub(super) fn pulser(
    config: &mut bldb::Config,
    _env: &mut [repl::Value],
) -> repl::Result<repl::Value> {
    run(&mut config.cons, b"oOo.", Duration::from_millis(500));
    Ok(repl::Value::Nil)
}
