
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

mod cpu;
mod register;

use cpu::Cpu;

fn main() -> io::Result<()> {

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Usage: ruGameboy <binary.gb>");
    }

    let mut file = File::open(&args[1])?;
    let mut binary = Vec::new();
    file.read_to_end(&mut binary)?;
    let mut cpu = Cpu::new(binary);

    while cpu.pc < 0x0160 {
        let inst = cpu.fetch();
        cpu.execute(inst);
    }

    println!("{}", cpu.dump());

    Ok(())
}
