use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use log::{error, debug};

#[macro_use]
extern crate num_derive;
use minifb::{Key, Window, WindowOptions};

mod cpu;
mod gpu;
mod register;
mod instruction;
mod bus;
mod memory;
mod vm;
mod timer;

use vm::{Vm, WIDTH, HEIGHT};

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
        if vm.run().is_err() {
            break;
        }
        window.update_with_buffer(&vm.buffer, WIDTH, HEIGHT).unwrap();
    }
    vm.dump();
    Ok(())
}
