use crate::bus::{Device};
use crate::{WIDTH, HEIGHT};

const BLACK: u32 = 0x00000000u32;
const DGRAY: u32 = 0x00555555u32;
const LGRAY: u32 = 0x00AAAAAAu32;
const WHITE: u32 = 0x00FFFFFFu32;

pub const VRAM_START:     u16 = 0x8000;
pub const VRAM_END:       u16 = 0x9fff;
pub const OAM_START:      u16 = 0xfe00;
pub const OAM_END:        u16 = 0xfe9f;

#[derive(PartialEq)]
pub enum GpuMode {
    /// First scanline mode, render data from OAM memory
    ScanlineOAM,
    /// Second scanline mode, render data from VRAM (tile) memory
    ScanlineVRAM,
    /// Horizontal blank mode, CPU can access OAM and VRAM
    HBlank,
    /// Vertical blank mode, CPU can access OAM and VRAM
    VBlank,
}

#[derive(Debug,Clone,Copy)]
pub struct LCDC {
    /// LCD control operation
    /// false: stop
    /// true:  operation
    operation: bool,
    /// select tile map
    /// false: 0x9800-0x9bff
    /// true:  0x9c00-0x9fff
    windows_tile_map: bool,
    /// window display
    /// false: off
    /// true:  on
    window_display: bool,
    /// BG & window tile data select
    /// false: 0x8800-0x9cff
    /// true:  0x8000-0x8fff
    bg_tile_data_select: bool,
    /// BG tile map display select
    /// false: 0x9800-0x9bff
    /// true:  0x9c00-0x9fff
    bg_tile_map_select: bool,
    /// obj sprite size (width x height)
    /// false: 8x8
    /// true:  8x16
    obj_size: bool,
    /// obj (sprite) display
    /// false: off
    /// true:  on
    obj_display: bool,
    /// bg & window display
    /// false: off
    /// true:  on
    bg_display: bool,
}

impl LCDC {
    pub fn from_u8(byte: u8) -> Self {
        Self {
            operation:           byte & 0b10000000 != 0,
            windows_tile_map:    byte & 0b01000000 != 0,
            window_display:      byte & 0b00100000 != 0,
            bg_tile_data_select: byte & 0b00010000 != 0,
            bg_tile_map_select:  byte & 0b00001000 != 0,
            obj_size:            byte & 0b00000100 != 0,
            obj_display:         byte & 0b00000010 != 0,
            bg_display:          byte & 0b00000001 != 0,
        }
    }

    pub fn to_u8(&self) -> u8 {
        (self.operation as u8) << 7 |
            (self.windows_tile_map as u8) << 6 |
            (self.window_display as u8) << 5 |
            (self.bg_tile_data_select as u8) << 4 |
            (self.bg_tile_map_select as u8) << 3 |
            (self.obj_size as u8) << 2 |
            (self.obj_display as u8) << 1 |
            (self.bg_display as u8)
    }
}

#[derive(Default,Clone,Copy,Debug)]
pub struct Sprite {
    /// tile_idx: sprite shows tile number
    tile_idx: u8,
    /// x: sprite left position
    /// y: sprite top position
    x: isize,
    y: isize,
    /// priority:
    /// 0: on top of background and window
    /// 1: behind color 1, 2, 3 of background and window
    priority: bool,
    /// flip_y: flip if 1
    flip_y: bool,
    /// flip: flip if 1
    flip_x: bool,
    /// palette_number:
    /// 0: from OBJ0PAL
    /// 1: from OBJ1PAL
    palette_number: bool
}

pub struct Gpu {
    /// Clock to switch mode
    clock: u64,
    /// current display line number
    pub line: u8,
    /// lcdc, LCD control line
    pub lcdc: LCDC,
    /// background & window palette data
    pub bg_palette: u8,
    /// object palette 0
    pub ob0_palette: u8,
    /// object palette 1
    pub ob1_palette: u8,
    /// current display mode
    pub mode: GpuMode,
    /// SCY: background Y position
    pub scy: u8,
    /// SCX: background X position
    pub scx: u8,
    /// vram: 0x8000-0x9BFF 6144 bytes
    vram: Vec<u8>,
    /// oam: 0xFE00-0xFE9F 160 bytes
    oam: Vec<u8>,

    /// sprite
    sprite: [Sprite;40],
    // whether vblank interrupt is occured
    pub is_interrupt: bool
}

impl Gpu {
    pub fn new() -> Self {
        let ram = vec![0; (VRAM_END - VRAM_START + 1) as usize];
        let oam = vec![0; (OAM_END - OAM_START + 1) as usize];
        Self {
            clock: 0,
            line: 0,
            lcdc: LCDC::from_u8(0x91),
            bg_palette: 0,
            ob0_palette: 0,
            ob1_palette: 0,
            mode: GpuMode::ScanlineOAM,
            scy: 0,
            scx: 0,
            vram: ram,
            oam: oam,
            sprite: [Default::default();40],
            is_interrupt: false
        }
    }

    pub fn get_tile_line(&self, tile_idx: usize, line_idx: usize) -> Vec<u32> {
        assert!(line_idx < 8);
        let addr = (tile_idx * 8 + line_idx) * 2;

        let byte1 = self.vram[addr];
        let byte2 = self.vram[addr+1];

        let mut pxs = Vec::with_capacity(8);

        for j in (0..8).rev() {
            let bit1 = ((byte1 >> j) & 0x1) == 0;
            let bit2 = ((byte2 >> j) & 0x1) == 0;
            let color = match (bit1, bit2) {
                (false, false) => BLACK,
                (false, true) => DGRAY,
                (true, false) => LGRAY,
                (true, true) => WHITE,
            };
            pxs.push(color);
        }
        pxs
    }

    fn build_background(&self, buffer: &mut Vec<u32>) {
        // TODO implement (row, col) offset from (scx, scy)
        for row in 0..HEIGHT {
            let tile_row = row / 8;
            let line_idx = row % 8;
            for col in 0..(WIDTH/8) {
                let tile_addr = tile_row * 32 + col + (0x9800 - 0x8000);
                let tile_idx = self.vram[tile_addr] as usize;
                let pixels = self.get_tile_line(tile_idx, line_idx);

                let pixel_start = row * WIDTH + col * 8;
                buffer.splice(pixel_start..(pixel_start + 8), pixels.iter().cloned());
            }
        }
    }

    fn build_sprite(&self, buffer: &mut Vec<u32>) {
        for sprite in self.sprite.iter() {
            // check sprite intersect with screen
            if sprite.y + 8 <= 0 || sprite.x + 9 <= 0 ||
               (sprite.x as usize) > WIDTH || (sprite.y as usize) > HEIGHT {
                continue;
            }
            for row_idx in 0..8 {
                let y = sprite.y + row_idx as isize;
                if y < 0 || (y as usize) > HEIGHT {
                    continue;
                }
                let y_idx = if sprite.flip_y { 7-row_idx } else { row_idx };
                let pixels = self.get_tile_line(sprite.tile_idx as usize, y_idx);
                for col_idx in 0..8 {
                    let x = sprite.x + col_idx as isize;
                    if x < 0 || (x as usize) > WIDTH {
                        continue;
                    }
                    let x_idx = if sprite.flip_x { 7-col_idx } else { col_idx };
                    // fill the buffer
                    buffer[y as usize * WIDTH + x as usize] = pixels[x_idx];
                }
            }
        }
    }

    pub fn build_screen(&self, buffer: &mut Vec<u32>) {
        if self.lcdc.bg_display {
            self.build_background(buffer);
        }

        if self.lcdc.obj_display {
            self.build_sprite(buffer);
        }
    }

    pub fn update(&mut self, clock: u64) {
        // switch state
        self.clock = self.clock.wrapping_add(clock);
        match self.mode {
            GpuMode::ScanlineOAM if self.clock >= 80 => {
                self.clock -= 80;
                self.mode = GpuMode::ScanlineVRAM;
            },
            GpuMode::ScanlineVRAM if self.clock >= 172 => {
                self.clock -= 172;
                self.mode = GpuMode::HBlank;
            },
            GpuMode::HBlank if self.clock >= 204 => {
                self.clock -= 204;
                if self.line >= 143 {
                    self.mode = GpuMode::VBlank;
                    // enable vblank interrupt
                    self.is_interrupt = true;
                } else {
                    self.mode = GpuMode::ScanlineOAM;
                }
                self.line += 1;
            },
            GpuMode::VBlank if self.clock >= 456 => {
                self.clock -= 456;
                self.line += 1;
                if self.line >= 153 {
                    self.line = 0;
                    self.mode = GpuMode::ScanlineOAM;
                }
            },
            _ => {},
        }
    }

    fn update_sprite(&mut self, addr: usize) {
        let sprite_idx = addr / 4;
        let value = self.oam[addr];
        match addr & 0x03 {
            0 => self.sprite[sprite_idx].y = value as isize - 16,
            1 => self.sprite[sprite_idx].x = value as isize - 8,
            2 => self.sprite[sprite_idx].tile_idx = value,
            3 => {
                self.sprite[sprite_idx].priority       = ((value >> 0x7) & 0x1) != 0;
                self.sprite[sprite_idx].flip_y         = ((value >> 0x6) & 0x1) != 0;
                self.sprite[sprite_idx].flip_x         = ((value >> 0x5) & 0x1) != 0;
                self.sprite[sprite_idx].palette_number = ((value >> 0x4) & 0x1) != 0;
            }
            _ => {},
        }
    }
}

impl Device for Gpu {
    fn load(&self, addr: u16) -> Result<u8, ()> {
        match addr {
            VRAM_START ..= VRAM_END => {
                let addr = (addr - VRAM_START) as usize;
                match self.vram.get(addr) {
                    Some(elem) => Ok(*elem),
                    None => Err(()),
                }
            }
            OAM_START ..= OAM_END => {
                let addr = (addr - OAM_START) as usize;
                match self.oam.get(addr) {
                    Some(elem) => Ok(*elem),
                    None => Err(()),
                }
            }
            _ => Err(()),
        }
    }

    fn store(&mut self, addr: u16, value: u8) -> Result<(), ()> {
        match addr {
            VRAM_START ..= VRAM_END => {
                let addr = (addr - VRAM_START) as usize;
                match self.vram.get_mut(addr as usize) {
                    Some(elem) => {
                        *elem = value;
                        Ok(())
                    },
                    None => Err(()),
                }
            }
            OAM_START ..= OAM_END => {
                let addr = (addr - OAM_START) as usize;
                match self.oam.get_mut(addr as usize) {
                    Some(elem) => {
                        *elem = value;
                        self.update_sprite(addr);
                        Ok(())
                    },
                    None => Err(()),
                }
            }
            _ => Err(()),
        }
    }
}
