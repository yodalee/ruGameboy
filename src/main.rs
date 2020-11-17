use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use log::{error, debug};

use minifb::{Key, Window, WindowOptions};

mod cpu;
mod register;
mod instruction;
mod bus;
mod memory;

use cpu::Cpu;
use instruction::Instruction;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

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

    let mut cpu = Cpu::new(binary);

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new(
        "rust Gameboy",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    ).unwrap_or_else(|e| { panic!("{}", e); });
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let byte = match cpu.fetch() {
            Ok(byte) => byte,
            Err(()) => break,
        };

        if let Some(inst) = Instruction::from_byte(byte as u8) {
            match cpu.execute(inst) {
                Ok(offset) => cpu.pc += offset,
                Err(()) => break,
            }
        } else {
            debug!("Unsupport instruction {:#x}", byte);
            break;
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }

    debug!("{}", cpu.dump());

    Ok(())
}
