
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

mod cpu;
mod register;
mod instruction;

use cpu::Cpu;
use instruction::Instruction;

fn main() -> io::Result<()> {

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Usage: ruGameboy <binary.gb>");
    }

    let mut file = File::open(&args[1])?;
    let mut binary = Vec::new();
    file.read_to_end(&mut binary)?;
    let mut cpu = Cpu::new(binary);

    loop {
        let byte = cpu.load8();
        if let Some(inst) = Instruction::from_byte(byte) {
            match cpu.execute(inst) {
                Ok(offset) => cpu.pc += offset,
                Err(()) => break,
            }
        } else {
            dbg!(&format!("Unsupport instruction {:#x}", byte));
            break;
        }
    }

    println!("{}", cpu.dump());

    Ok(())
}
