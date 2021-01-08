use crate::bus::Device;
use std::default::Default;

pub const TIMER_START: u16 = 0xff04;
pub const TIMER_END: u16 = 0xff07;

enum TimerScale {
    X1  = 0b00, // freq 4096
    X4  = 0b11, // freq 16384
    X16 = 0b10, // freq 65536
    X64 = 0b01, // freq 262144
}

impl Default for TimerScale {
    fn default() -> Self { TimerScale::X1 }
}

#[derive(Default)]
pub struct TimerControl {
    scale: TimerScale,
    running: bool,
}

#[derive(Default)]
pub struct Timer {
    /// ff04 div, incremented 16384 times a second
    div: u8,
    /// ff05 tima, incremented by frequency set by TAC
    tima: u8,
    /// ff06 tma, when tima overflow, it load value from tma
    tma: u8,
    /// ff07 tac: timer control
    /// Bit2:
    ///   0: Stop timer
    ///   1: Start Timer
    /// Bit10:
    ///   00 => 4.096 KHz
    ///   01 => 262.144 KHz Hz
    ///   10 => 65.536 KHz
    ///   11 => 16.384 KHz
    tac: TimerControl,

    // implementation
    div_counter: u64,
    timer_counter: u64,
    roundvalue: u64,
    pub is_interrupt: bool,
}

impl Timer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn is_interrupt(&self) -> bool{
        self.is_interrupt
    }

    pub fn update(&mut self, clock: u64) {
        // handle div
        // div has a constant update rate: 16384 Hz
        // which means its round value is 4MHz / 16384 = 256
        self.div_counter += clock;
        if self.div_counter >= 256 {
            self.div_counter -= 256;
            self.div = self.div.wrapping_add(1);
        }

        // handle tac
        if self.tac.running {
            self.timer_counter += clock;
            if self.timer_counter >= self.roundvalue {
                self.timer_counter -= self.roundvalue;

                if self.tma == 0xff {
                    self.tma = self.tima;
                } else {
                    self.tma = self.tma.wrapping_add(1);
                }
            }
        }
    }
}

impl Device for Timer {
    fn load(&self, addr: u16) -> Result<u8, ()> {
        match addr {
            0xFF04 => Ok(self.div),
            0xFF05 => Ok(self.tima),
            0xFF06 => Ok(self.tma),
            0xFF07 => Ok({
                ( if self.tac.running { 1 << 2 } else { 0 } ) |
                ( match self.tac.scale {
                    TimerScale::X1  => 0b00,
                    TimerScale::X4  => 0b11,
                    TimerScale::X16 => 0b10,
                    TimerScale::X64 => 0b01,
                })
            }),
            _ => Err(()),
        }
    }

    fn store(&mut self, addr: u16, value: u8) -> Result<(), ()> {
        match addr {
            0xFF04 => self.div = 0,
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => {
                self.tac.running = (value & 0x4) != 0;
                self.tac.scale = match value & 0x3 {
                    0 => TimerScale::X1,
                    1 => TimerScale::X64,
                    2 => TimerScale::X16,
                    3 => TimerScale::X4,
                    _ => return Err(()),
                };
                self.roundvalue = match self.tac.scale {
                    TimerScale::X1  => 1024, // 4MHz / 1024 = 4.096 KHz
                    TimerScale::X4  => 256,  // 4MHz / 256  = 16.384 KHz
                    TimerScale::X16 => 64,   // 4MHz / 64   = 65.536 KHz
                    TimerScale::X64 => 16,   // 4MHz / 16   = 262.144 KHz
                };
                // reset timer_counter so it will surpass limit too much
                self.timer_counter = 0;
            },
            _ => return Err(()),
        }
        Ok(())
    }

    fn range(&self) -> (u16, u16) {
        (TIMER_START, TIMER_END)
    }
}
