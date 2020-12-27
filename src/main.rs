use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use log::{error, debug};

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

fn main() -> io::Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        error!("Usage: ruGameboy < binary.gb>");
        std::process::exit(1);
    }

    let mut file = File::open(&args[1])?;
    let mut binary = Vec::new();
    file.read_to_end(&mut binary)?;

    let mut vm = Vm::new(binary);
    let mut window = Window::new(
        "rust Gameboy",
        WIDTH,
        HEIGHT,
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
