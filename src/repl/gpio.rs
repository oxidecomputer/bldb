// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bldb;
use crate::gpio;
use crate::println;
use crate::repl::{self, Value};
use crate::result::{Error, Result};
use alloc::vec::Vec;

pub(super) fn get(
    config: &mut bldb::Config,
    env: &mut Vec<Value>,
) -> Result<Value> {
    let usage = |error| {
        println!("usage: gpioget <pin>");
        error
    };
    let pin = repl::popenv(env).as_num::<u8>().map_err(usage)?;
    let reg = config.gpios.get_pin(pin);
    print(pin, reg);
    Ok(Value::Unsigned(reg.bits().into()))
}

fn print(pin: u8, reg: gpio::Reg) {
    println!("GPIO pin {pin} state: {reg:x?} {{");
    println!("    debounce_timer: {}", reg.debounce_timer());
    println!("    debounce_timer_unit: {}", reg.debounce_timer_unit());
    println!("    debounce_ctl: {:?}", reg.debounce_ctl());
    println!("    debounce_timer_large: {:?}", reg.debounce_timer_large());
    println!("    trigger_type: {:?}", reg.trigger_type());
    println!("    active_level: {:?}", reg.active_level());
    println!(
        "    interrupt_status_enable: {:?}",
        reg.interrupt_status_enable()
    );
    println!("    interrupt_enable: {:?}", reg.interrupt_enable());
    println!(
        "    wake_in_power_saving_mode: {:?}",
        reg.wake_in_power_saving_mode()
    );
    println!("    wake_in_s3: {:?}", reg.wake_in_s3());
    println!("    wake_in_s4_or_s5: {:?}", reg.wake_in_s4_or_s5());
    println!("    pin_status: {:?}", reg.pin_status());
    println!("    drive_strength: {:?}", reg.drive_strength());
    println!("    resv0: (ignored)");
    println!("    pull_up_enable: {:?}", reg.pull_up_enable());
    println!("    pull_down_enable: {:?}", reg.pull_down_enable());
    println!("    output_value: {:?}", reg.output_value());
    println!("    output_enable: {:?}", reg.output_enable());
    println!("    sw_ctl_input: {:?}", reg.sw_ctl_input());
    println!("    sw_ctl_input_enable: {:?}", reg.sw_ctl_input_enable());
    println!("    rx_disable: {:?}", reg.rx_disable());
    println!("    resv1: (ignored)");
    println!("    interrupt_status: {:?}", reg.interrupt_status());
    println!("    wake_status: {:?}", reg.wake_status());
    println!(
        "    gpio0_pwr_btn_press_less_2sec_status: {:?}",
        reg.gpio0_pwr_btn_press_less_2sec_status()
    );
    println!(
        "    gpio0_pwr_btn_press_less_10s_status: {:?}",
        reg.gpio0_pwr_btn_press_less_10s_status()
    );

    println!("}}");
}

struct ParsedState {
    output_enable: bool,
    pullup: bool,
    pulldown: bool,
    active: gpio::ActiveLevel,
    output: gpio::PinStatus,
}

impl ParsedState {
    fn try_from_string(s: &str) -> Result<ParsedState> {
        let mut pullup = false;
        let mut pulldown = false;
        let mut active = gpio::ActiveLevel::High;
        let mut output = gpio::PinStatus::Low;
        let mut output_enable = false;
        for tok in s.split(',') {
            match tok {
                "-pu" => pullup = false,
                "+pu" | "pu" => pullup = true,
                "-pd" => pulldown = false,
                "+pd" | "pd" => pulldown = true,
                "ah" => active = gpio::ActiveLevel::High,
                "al" => active = gpio::ActiveLevel::Low,
                "oh" | "-ol" => output = gpio::PinStatus::High,
                "ol" | "-oh" => output = gpio::PinStatus::Low,
                "out" => output_enable = true,
                "in" => output_enable = false,
                _ => return Err(Error::BadArgs),
            }
        }
        Ok(ParsedState { output_enable, pullup, pulldown, active, output })
    }
}

pub(super) fn set(
    config: &mut bldb::Config,
    env: &mut Vec<Value>,
) -> Result<Value> {
    let usage = |err| {
        println!("usage: gpioset <pin> <function>");
        err
    };
    let pin = repl::popenv(env).as_num::<u8>().map_err(usage)?;
    let statestr = repl::popenv(env).as_string().map_err(usage)?;
    let state = ParsedState::try_from_string(&statestr).map_err(usage)?;
    let mut reg = config.gpios.get_pin(pin);
    reg.set_pull_up_enable(state.pullup);
    reg.set_pull_down_enable(state.pulldown);
    reg.set_output_enable(state.output_enable);
    reg.set_active_level(state.active);
    reg.set_output_value(state.output);
    unsafe {
        config.gpios.set_pin(pin, reg);
    }
    Ok(Value::Nil)
}
