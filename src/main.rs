use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use log::{error, debug};
use clap::{App, Arg};

#[macro_use]
extern crate num_derive;
use minifb::{Key, Window, WindowOptions, KeyRepeat};

mod cpu;
mod gpu;
mod register;
mod instruction;
mod bus;
mod memory;
mod vm;
mod timer;
mod joypad;

use vm::{Vm, WIDTH, HEIGHT};
use joypad::{JoypadKey};

const MAX_ENLARGE_SCALE: usize = 5;

fn arg_check_range<T>(arg: &str, range: (T, T)) -> Result<T, String>
    where T: Ord + std::str::FromStr + std::fmt::Display
{
    let (min, max) = range;
    match arg.parse::<T>() {
        Ok(n) if min <= n && n <= max => Ok(n),
        Err(_) => Err(String::from("Please select an integer as argument")),
        _ => Err(format!("Please select integer in range {} to {}", min , max)),
    }
}

fn main() -> io::Result<()> {
    env_logger::init();

    let prog = App::new("ruGameboy")
                    .arg(Arg::with_name("scale")
                            .help("Set the scale of enlarge in range [1-5]")
                            .short("s")
                            .long("scale")
                            .default_value("1"))
                    .arg(Arg::with_name("binary")
                            .help("Set the binary file to run")
                            .required(true))
                    .get_matches();

    let bin_name = prog.value_of("binary").unwrap();

    let scale = prog.value_of("scale").unwrap();
    let scale = arg_check_range(scale, (1, MAX_ENLARGE_SCALE)).unwrap_or_else(|e| {
                    error!("scale: {}", e);
                    std::process::exit(1);
                });

    let mut file = File::open(bin_name)?;
    let mut binary = Vec::new();
    file.read_to_end(&mut binary)?;

    let mut vm = Vm::new(binary);
    let mut window = Window::new(
        "rust Gameboy",
        WIDTH * scale,
        HEIGHT * scale,
        WindowOptions::default(),
    ).unwrap_or_else(|e| { panic!("{}", e); });
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {

        // check key press
        window.get_keys_pressed(KeyRepeat::No).map(|keys| {
            for key in keys {
                match key {
                    Key::Up    => vm.cpu.bus.joypad.presskey(JoypadKey::UP),
                    Key::Down  => vm.cpu.bus.joypad.presskey(JoypadKey::DOWN),
                    Key::Left  => vm.cpu.bus.joypad.presskey(JoypadKey::LEFT),
                    Key::Right => vm.cpu.bus.joypad.presskey(JoypadKey::RIGHT),
                    Key::A     => vm.cpu.bus.joypad.presskey(JoypadKey::START),
                    Key::S     => vm.cpu.bus.joypad.presskey(JoypadKey::SELECT),
                    Key::Z     => vm.cpu.bus.joypad.presskey(JoypadKey::A),
                    Key::X     => vm.cpu.bus.joypad.presskey(JoypadKey::B),
                    _ => (),
                }
            }
        });

        // check key release
        window.get_keys_released().map(|keys| {
            for key in keys {
                match key {
                    Key::Up    => vm.cpu.bus.joypad.releasekey(JoypadKey::UP),
                    Key::Down  => vm.cpu.bus.joypad.releasekey(JoypadKey::DOWN),
                    Key::Left  => vm.cpu.bus.joypad.releasekey(JoypadKey::LEFT),
                    Key::Right => vm.cpu.bus.joypad.releasekey(JoypadKey::RIGHT),
                    Key::A     => vm.cpu.bus.joypad.releasekey(JoypadKey::START),
                    Key::S     => vm.cpu.bus.joypad.releasekey(JoypadKey::SELECT),
                    Key::Z     => vm.cpu.bus.joypad.releasekey(JoypadKey::A),
                    Key::X     => vm.cpu.bus.joypad.releasekey(JoypadKey::B),
                    _ => (),
                }
            }
        });

        if vm.run().is_err() {
            break;
        }
        window.update_with_buffer(&vm.buffer, WIDTH, HEIGHT).unwrap();
    }
    vm.dump();
    Ok(())
}
