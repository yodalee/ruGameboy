pub mod vm;
mod bus;
mod cpu;
mod gpu;
mod instruction;
mod joypad;
mod memory;
mod register;
mod timer;

pub enum JoypadKey {
    RIGHT,
    LEFT,
    UP,
    DOWN,
    A,
    B,
    SELECT,
    START,
}

#[repr(u8)]
#[derive(Clone)]
pub enum Pixel {
    BLACK = 0,
    LGRAY = 1,
    DGRAY = 2,
    WHITE = 3,
}
