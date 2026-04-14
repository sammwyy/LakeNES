use crate::mappers::{Mapper, Mirroring};
use alloc::vec::Vec;
use bitflags::bitflags;

// =========================================================================
//  CONSTANTS & LOOKUP TABLES
// =========================================================================

const NES_PALETTE: [u32; 64] = [
    0x666666, 0x002A88, 0x1412A7, 0x3B00A4, 0x5C007E, 0x6E0040, 0x6C0600, 0x561D00, 0x333500,
    0x0B4800, 0x005200, 0x004F08, 0x00404D, 0x000000, 0x000000, 0x000000, 0xADADAD, 0x155FD9,
    0x4240FF, 0x7527FE, 0xA01ACC, 0xB71E7B, 0xB53120, 0x994E00, 0x6B6D00, 0x388700, 0x0C9300,
    0x008F32, 0x007C8D, 0x000000, 0x000000, 0x000000, 0xFFFEFF, 0x64B0FF, 0x9290FF, 0xC676FF,
    0xF36AFF, 0xFE6ECC, 0xFE8170, 0xEA9E22, 0xBCBE00, 0x88D800, 0x5CE430, 0x45E082, 0x48CDDE,
    0x4F4F4F, 0x000000, 0x000000, 0xFFFEFF, 0xC0DFFF, 0xD3D2FF, 0xE8C8FF, 0xFBC2FF, 0xFEC4EA,
    0xFECCC5, 0xF7D8A5, 0xE4E594, 0xCFEF96, 0xBDF4AB, 0xB3F3CC, 0xB5EBF2, 0xB8B8B8, 0x000000,
    0x000000,
];

// =========================================================================
//  REGISTERS
// =========================================================================

bitflags! {
    #[derive(Clone, Copy)]
    pub struct Control: u8 {
        const NAMETABLE_X        = 0x01;
        const NAMETABLE_Y        = 0x02;
        const VRAM_INCREMENT     = 0x04; // 0 = add 1, 1 = add 32
        const SPRITE_PATTERN     = 0x08; // 0: $0000; 1: $1000
        const BACKGROUND_PATTERN = 0x10; // 0: $0000; 1: $1000
        const SPRITE_SIZE        = 0x20;
        const MASTER_SLAVE       = 0x40;
        const ENABLE_NMI         = 0x80;
    }
}

bitflags! {
    #[derive(Clone, Copy)]
    pub struct Mask: u8 {
        const GRAYSCALE          = 0x01;
        const RENDER_BACKGROUND_LEFT = 0x02;
        const RENDER_SPRITES_LEFT    = 0x04;
        const RENDER_BACKGROUND      = 0x08;
        const RENDER_SPRITES         = 0x10;
        const ENHANCE_RED            = 0x20;
        const ENHANCE_GREEN          = 0x40;
        const ENHANCE_BLUE           = 0x80;
    }
}

bitflags! {
    #[derive(Clone, Copy)]
    pub struct Status: u8 {
        const UNUSED          = 0x1F;
        const SPRITE_OVERFLOW = 0x20;
        const SPRITE_ZHIT     = 0x40;
        const VBLANK          = 0x80;
    }
}

#[derive(Clone, Copy, Default)]
struct LoopyRegister {
    pub reg: u16,
}

impl LoopyRegister {
    pub fn new() -> Self {
        Self { reg: 0 }
    }

    // Coarse X (5 bits)
    pub fn coarse_x(&self) -> u8 {
        (self.reg & 0x001F) as u8
    }

    pub fn set_coarse_x(&mut self, val: u8) {
        self.reg = (self.reg & !0x001F) | (val as u16 & 0x001F);
    }

    // Coarse Y (5 bits)
    pub fn coarse_y(&self) -> u8 {
        ((self.reg >> 5) & 0x001F) as u8
    }

    pub fn set_coarse_y(&mut self, val: u8) {
        self.reg = (self.reg & !0x03E0) | ((val as u16 & 0x001F) << 5);
    }

    // Nametable Select (2 bits)
    pub fn nametable_x(&self) -> u8 {
        ((self.reg >> 10) & 0x01) as u8
    }

    pub fn set_nametable_x(&mut self, val: u8) {
        self.reg = (self.reg & !0x0400) | ((val as u16 & 0x01) << 10);
    }

    pub fn nametable_y(&self) -> u8 {
        ((self.reg >> 11) & 0x01) as u8
    }

    pub fn set_nametable_y(&mut self, val: u8) {
        self.reg = (self.reg & !0x0800) | ((val as u16 & 0x01) << 11);
    }

    // Fine Y (3 bits)
    pub fn fine_y(&self) -> u8 {
        ((self.reg >> 12) & 0x0007) as u8
    }

    pub fn set_fine_y(&mut self, val: u8) {
        self.reg = (self.reg & !0x7000) | ((val as u16 & 0x0007) << 12);
    }
}

// =========================================================================
//  PPU STRUCT
// =========================================================================

pub struct PPU {
    // Memory
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    pub oam_addr: u8,

    // Output
    pub frame_buffer: Vec<u32>,
    pub nmi_interrupt: bool,

    // Internal State
    pub cycle: i16,    // 0-340
    pub scanline: i16, // 0-261
    pub frame_count: u64,

    // Registers
    ctrl: Control,
    mask: Mask,
    status: Status,

    // Loopy
    v_ram: LoopyRegister, // Current VRAM addr
    t_ram: LoopyRegister, // Temp VRAM addr
    fine_x: u8,
    w_toggle: bool, // write toggle

    // Data Buffer
    data_buffer: u8,

    // Background Rendering
    bg_next_tile_id: u8,
    bg_next_tile_attr: u8,
    bg_next_tile_lsb: u8,
    bg_next_tile_msb: u8,

    bg_shifter_pattern_lo: u16,
    bg_shifter_pattern_hi: u16,
    bg_shifter_attrib_lo: u16,
    bg_shifter_attrib_hi: u16,

    // Sprites
    scanline_sprites: Vec<u8>, // Indices of sprites on this scanline
    sprite_shifter_pattern_lo: [u8; 8],
    sprite_shifter_pattern_hi: [u8; 8],
    sprite_zero_hit_possible: bool,
    sprite_zero_being_rendered: bool,

    // Legacy / Debug compatibility
    pub scroll_x: u8,
    pub scroll_y: u8,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            palette_table: [0; 32],
            vram: [0; 2048],
            oam_data: [0; 256],
            oam_addr: 0,
            frame_buffer: alloc::vec![0u32; 256 * 240],
            nmi_interrupt: false,
            cycle: 0,
            scanline: 0,
            frame_count: 0,
            ctrl: Control::empty(),
            mask: Mask::empty(),
            status: Status::empty(),
            v_ram: LoopyRegister::new(),
            t_ram: LoopyRegister::new(),
            fine_x: 0,
            w_toggle: false,
            data_buffer: 0,
            bg_next_tile_id: 0,
            bg_next_tile_attr: 0,
            bg_next_tile_lsb: 0,
            bg_next_tile_msb: 0,
            bg_shifter_pattern_lo: 0,
            bg_shifter_pattern_hi: 0,
            bg_shifter_attrib_lo: 0,
            bg_shifter_attrib_hi: 0,

            scanline_sprites: Vec::with_capacity(8),
            sprite_shifter_pattern_lo: [0; 8],
            sprite_shifter_pattern_hi: [0; 8],
            sprite_zero_hit_possible: false,
            sprite_zero_being_rendered: false,
            scroll_x: 0,
            scroll_y: 0,
        }
    }

    // =========================================================================
    //  CPU READ/WRITE
    // =========================================================================

    // $2000 PPUCTRL
    fn write_ctrl(&mut self, value: u8) {
        self.ctrl = Control::from_bits_truncate(value);
        self.t_ram.set_nametable_x((value >> 0) & 1);
        self.t_ram.set_nametable_y((value >> 1) & 1);
    }

    // $2001 PPUMASK
    fn write_mask(&mut self, value: u8) {
        self.mask = Mask::from_bits_truncate(value);
    }

    // $2002 PPUSTATUS
    fn read_status(&mut self) -> u8 {
        let res = (self.status.bits() & 0xE0) | (self.data_buffer & 0x1F);
        self.status.remove(Status::VBLANK);
        self.w_toggle = false;
        res
    }

    // $2005 PPUSCROLL
    fn write_scroll(&mut self, value: u8) {
        if !self.w_toggle {
            // First write: fine X and coarse X
            self.fine_x = value & 0x07;
            self.t_ram.set_coarse_x(value >> 3);
            self.w_toggle = true;
            // Legacy update
            self.scroll_x = value;
        } else {
            // Second write: fine Y and coarse Y
            self.t_ram.set_fine_y(value & 0x07);
            self.t_ram.set_coarse_y(value >> 3);
            self.w_toggle = false;
            // Legacy update
            self.scroll_y = value;
        }
    }

    // $2006 PPUADDR
    fn write_ppu_addr(&mut self, value: u8) {
        if !self.w_toggle {
            // First write: high byte
            let old = self.t_ram.reg;
            self.t_ram.reg = (old & 0x00FF) | ((value as u16 & 0x3F) << 8);
            self.t_ram.reg &= 0x3FFF; // Clear bit 14
            self.w_toggle = true;
        } else {
            // Second write: low byte
            let old = self.t_ram.reg;
            self.t_ram.reg = (old & 0xFF00) | (value as u16);
            self.v_ram.reg = self.t_ram.reg;
            self.w_toggle = false;
        }
    }

    // $2007 PPUDATA Write
    fn write_ppu_data(&mut self, value: u8, mapper: &mut dyn Mapper) {
        self.ppu_write(self.v_ram.reg, value, mapper);

        // Increment VRAM address
        let inc = if self.ctrl.contains(Control::VRAM_INCREMENT) {
            32
        } else {
            1
        };
        self.v_ram.reg = (self.v_ram.reg + inc) & 0x7FFF; // 15-bit
    }

    // $2007 PPUDATA Read
    fn read_ppu_data(&mut self, mapper: &mut dyn Mapper) -> u8 {
        let mut data = self.data_buffer;
        self.data_buffer = self.ppu_read(self.v_ram.reg, mapper);

        // If reading palette, return immediately (don't buffer)
        if self.v_ram.reg >= 0x3F00 {
            data = self.data_buffer;
        }

        let inc = if self.ctrl.contains(Control::VRAM_INCREMENT) {
            32
        } else {
            1
        };
        self.v_ram.reg = (self.v_ram.reg + inc) & 0x7FFF;

        data
    }

    // =========================================================================
    //  INTERNAL BUS
    // =========================================================================

    fn ppu_write(&mut self, addr: u16, value: u8, mapper: &mut dyn Mapper) {
        let addr = addr & 0x3FFF;
        match addr {
            0x0000..=0x1FFF => {
                mapper.write_chr(addr, value);
            }
            0x2000..=0x3EFF => {
                // Nametables
                let addr = addr & 0x0FFF;
                let mode = mapper.mirroring();
                let mirrored = match mode {
                    Mirroring::Horizontal => {
                        // 000-3FF -> 000, 400-7FF -> 000, 800-BFF -> 400, C00-FFF -> 400
                        if addr < 0x0800 {
                            addr & 0x03FF
                        } else {
                            (addr & 0x03FF) + 0x400
                        }
                    }
                    Mirroring::Vertical => {
                        // 000-3FF -> 000, 400-7FF -> 400, 800-BFF -> 000, C00-FFF -> 400
                        addr & 0x07FF
                    }
                    Mirroring::OneScreenLow => addr & 0x03FF,
                    Mirroring::OneScreenHigh => (addr & 0x03FF) + 0x400,
                };
                self.vram[mirrored as usize] = value;
            }
            0x3F00..=0x3FFF => {
                // Palettes
                let mut addr = addr & 0x001F;
                if addr == 0x0010 {
                    addr = 0x0000;
                }
                if addr == 0x0014 {
                    addr = 0x0004;
                }
                if addr == 0x0018 {
                    addr = 0x0008;
                }
                if addr == 0x001C {
                    addr = 0x000C;
                }
                self.palette_table[addr as usize] = value;
            }
            _ => {}
        }
    }

    fn ppu_read(&self, addr: u16, mapper: &mut dyn Mapper) -> u8 {
        let addr = addr & 0x3FFF;
        match addr {
            0x0000..=0x1FFF => mapper.read_chr(addr),
            0x2000..=0x3EFF => {
                let addr = addr & 0x0FFF;
                let mode = mapper.mirroring();
                let mirrored = match mode {
                    Mirroring::Horizontal => {
                        if addr < 0x0800 {
                            addr & 0x03FF
                        } else {
                            (addr & 0x03FF) + 0x400
                        }
                    }
                    Mirroring::Vertical => addr & 0x07FF,
                    Mirroring::OneScreenLow => addr & 0x03FF,
                    Mirroring::OneScreenHigh => (addr & 0x03FF) + 0x400,
                };
                self.vram[mirrored as usize]
            }
            0x3F00..=0x3FFF => {
                let mut addr = addr & 0x001F;
                if addr == 0x0010 {
                    addr = 0x0000;
                }
                if addr == 0x0014 {
                    addr = 0x0004;
                }
                if addr == 0x0018 {
                    addr = 0x0008;
                }
                if addr == 0x001C {
                    addr = 0x000C;
                }
                self.palette_table[addr as usize]
            }
            _ => 0,
        }
    }

    // =========================================================================
    //  CLOCK & RENDERING
    // =========================================================================

    pub fn step(&mut self, mapper: &mut dyn Mapper) {
        if self.scanline >= -1 && self.scanline < 240 {
            // Visible Frame & Pre-render line
            if self.scanline == 0 && self.cycle == 0 {
                self.cycle = 1; // Skip cycle 0 on first line (odd frames mechanism skipped for simplicity)
            }

            if self.scanline == -1 && self.cycle == 1 {
                self.status
                    .remove(Status::VBLANK | Status::SPRITE_OVERFLOW | Status::SPRITE_ZHIT);
                self.sprite_zero_hit_possible = false;
                for i in 0..8 {
                    self.sprite_shifter_pattern_lo[i] = 0;
                    self.sprite_shifter_pattern_hi[i] = 0;
                }
            }

            // Rendering cycles
            if (self.cycle >= 2 && self.cycle < 258) || (self.cycle >= 321 && self.cycle < 338) {
                self.update_shifters();

                match (self.cycle - 1) % 8 {
                    0 => {
                        self.load_bg_shifters();
                        self.bg_next_tile_id =
                            self.ppu_read(0x2000 | (self.v_ram.reg & 0x0FFF), mapper);
                    }
                    2 => {
                        self.bg_next_tile_attr = self.ppu_read(
                            0x23C0
                                | (self.v_ram.nametable_y() as u16) << 11
                                | (self.v_ram.nametable_x() as u16) << 10
                                | ((self.v_ram.coarse_y() >> 2) as u16) << 3
                                | (self.v_ram.coarse_x() >> 2) as u16,
                            mapper,
                        );

                        if (self.v_ram.coarse_y() & 0x02) != 0 {
                            self.bg_next_tile_attr >>= 4;
                        }
                        if (self.v_ram.coarse_x() & 0x02) != 0 {
                            self.bg_next_tile_attr >>= 2;
                        }
                        self.bg_next_tile_attr &= 0x03;
                    }
                    4 => {
                        let addr = (self.ctrl.contains(Control::BACKGROUND_PATTERN) as u16) << 12
                            | (self.bg_next_tile_id as u16) << 4
                            | (self.v_ram.fine_y() as u16);
                        self.bg_next_tile_lsb = self.ppu_read(addr, mapper);
                    }
                    6 => {
                        let addr = (self.ctrl.contains(Control::BACKGROUND_PATTERN) as u16) << 12
                            | (self.bg_next_tile_id as u16) << 4
                            | (self.v_ram.fine_y() as u16)
                            | 8;
                        self.bg_next_tile_msb = self.ppu_read(addr, mapper);
                    }
                    7 => {
                        self.increment_scroll_x();
                    }
                    _ => {}
                }
            }

            if self.cycle == 256 {
                self.increment_scroll_y();
            }

            if self.cycle == 257 {
                self.load_bg_shifters();
                self.transfer_address_x();
            }

            if self.cycle == 338 || self.cycle == 340 {
                self.bg_next_tile_id = self.ppu_read(0x2000 | (self.v_ram.reg & 0x0FFF), mapper);
            }

            if self.scanline == -1 && self.cycle >= 280 && self.cycle < 305 {
                self.transfer_address_y();
            }

            // Foreground Rendering (Sprites) - Simplified for now
            // Just clearing at 257 for next line
            if self.cycle == 257 && self.scanline >= 0 {
                self.evaluate_sprites(mapper);
            }
        }

        // Actually render pixel
        if self.scanline >= 0 && self.scanline < 240 && self.cycle >= 1 && self.cycle <= 256 {
            self.render_pixel(mapper);
        }

        if self.scanline == 241 && self.cycle == 1 {
            self.status.insert(Status::VBLANK);
            if self.ctrl.contains(Control::ENABLE_NMI) {
                self.nmi_interrupt = true;
            }
        }

        // Advance
        self.cycle += 1;
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline >= 261 {
                self.scanline = -1;
                self.frame_count += 1;
            }
        }
    }

    fn increment_scroll_x(&mut self) {
        if !self.mask.contains(Mask::RENDER_BACKGROUND) && !self.mask.contains(Mask::RENDER_SPRITES)
        {
            return;
        }
        if self.v_ram.coarse_x() == 31 {
            self.v_ram.set_coarse_x(0);
            self.v_ram.set_nametable_x(!self.v_ram.nametable_x() & 1);
        } else {
            self.v_ram.set_coarse_x(self.v_ram.coarse_x() + 1);
        }
    }

    fn increment_scroll_y(&mut self) {
        if !self.mask.contains(Mask::RENDER_BACKGROUND) && !self.mask.contains(Mask::RENDER_SPRITES)
        {
            return;
        }
        if self.v_ram.fine_y() < 7 {
            self.v_ram.set_fine_y(self.v_ram.fine_y() + 1);
        } else {
            self.v_ram.set_fine_y(0);
            let mut y = self.v_ram.coarse_y();
            if y == 29 {
                y = 0;
                self.v_ram.set_nametable_y(!self.v_ram.nametable_y() & 1);
            } else if y == 31 {
                y = 0;
            } else {
                y += 1;
            }
            self.v_ram.set_coarse_y(y);
        }
    }

    fn transfer_address_x(&mut self) {
        if !self.mask.contains(Mask::RENDER_BACKGROUND) && !self.mask.contains(Mask::RENDER_SPRITES)
        {
            return;
        }
        self.v_ram.set_nametable_x(self.t_ram.nametable_x());
        self.v_ram.set_coarse_x(self.t_ram.coarse_x());
    }

    fn transfer_address_y(&mut self) {
        if !self.mask.contains(Mask::RENDER_BACKGROUND) && !self.mask.contains(Mask::RENDER_SPRITES)
        {
            return;
        }
        self.v_ram.set_fine_y(self.t_ram.fine_y());
        self.v_ram.set_nametable_y(self.t_ram.nametable_y());
        self.v_ram.set_coarse_y(self.t_ram.coarse_y());
    }

    fn load_bg_shifters(&mut self) {
        self.bg_shifter_pattern_lo =
            (self.bg_shifter_pattern_lo & 0xFF00) | self.bg_next_tile_lsb as u16;
        self.bg_shifter_pattern_hi =
            (self.bg_shifter_pattern_hi & 0xFF00) | self.bg_next_tile_msb as u16;

        self.bg_shifter_attrib_lo = (self.bg_shifter_attrib_lo & 0xFF00)
            | if (self.bg_next_tile_attr & 0x01) != 0 {
                0xFF
            } else {
                0x00
            };
        self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xFF00)
            | if (self.bg_next_tile_attr & 0x02) != 0 {
                0xFF
            } else {
                0x00
            };
    }

    fn update_shifters(&mut self) {
        if self.mask.contains(Mask::RENDER_BACKGROUND) {
            self.bg_shifter_pattern_lo <<= 1;
            self.bg_shifter_pattern_hi <<= 1;
            self.bg_shifter_attrib_lo <<= 1;
            self.bg_shifter_attrib_hi <<= 1;
        }

        if self.mask.contains(Mask::RENDER_SPRITES) && self.cycle >= 1 && self.cycle < 258 {
            for i in 0..self.scanline_sprites.len() {
                let sprite_x = self.oam_data[self.scanline_sprites[i] as usize * 4 + 3];
                if self.cycle as u16 - 1 > sprite_x as u16 {
                    // Simple decrement shift
                    self.sprite_shifter_pattern_lo[i] <<= 1;
                    self.sprite_shifter_pattern_hi[i] <<= 1;
                }
            }
        }
    }

    fn render_pixel(&mut self, mapper: &mut dyn Mapper) {
        let mut bg_pixel = 0u8;
        let mut bg_palette = 0u8;

        if self.mask.contains(Mask::RENDER_BACKGROUND) {
            let bit_mux = 0x8000 >> self.fine_x;
            let p0 = (self.bg_shifter_pattern_lo & bit_mux) != 0;
            let p1 = (self.bg_shifter_pattern_hi & bit_mux) != 0;
            bg_pixel = (p1 as u8) << 1 | (p0 as u8);

            let pal0 = (self.bg_shifter_attrib_lo & bit_mux) != 0;
            let pal1 = (self.bg_shifter_attrib_hi & bit_mux) != 0;
            bg_palette = (pal1 as u8) << 1 | (pal0 as u8);
        }

        let mut fg_pixel = 0u8;
        let mut fg_palette = 0u8;
        let mut fg_priority = false;
        let mut fg_sprite_zero = false;

        if self.mask.contains(Mask::RENDER_SPRITES) {
            self.sprite_zero_being_rendered = false;

            for i in 0..self.scanline_sprites.len() {
                let oam_idx = self.scanline_sprites[i] as usize;
                let sprite_x = self.oam_data[oam_idx * 4 + 3];

                // Only process sprites that have started (fine-grained shift counter
                // already handles the actual pixel output via shifters).
                let x_pos = self.cycle as u16 - 1;
                if x_pos >= sprite_x as u16 && x_pos < (sprite_x as u16 + 8) {
                    if fg_pixel == 0 {
                        let p0 = (self.sprite_shifter_pattern_lo[i] & 0x80) != 0;
                        let p1 = (self.sprite_shifter_pattern_hi[i] & 0x80) != 0;
                        fg_pixel = (p1 as u8) << 1 | (p0 as u8);

                        if fg_pixel != 0 {
                            let attr = self.oam_data[oam_idx * 4 + 2];
                            fg_palette = (attr & 0x03) + 4;
                            fg_priority = (attr & 0x20) == 0; // 0 = in front of BG

                            // Sprite zero is the one at OAM index 0
                            if oam_idx == 0 {
                                fg_sprite_zero = true;
                            }
                            break;
                        }
                    }
                }
            }
        }

        let pixel = match (bg_pixel, fg_pixel) {
            (0, 0) => 0,
            (0, fg) => fg_palette << 2 | fg,
            (bg, 0) => bg_palette << 2 | bg,
            (bg, fg) => {
                // Sprite-zero hit detection:
                // Conditions: both BG and sprite-0 pixels are opaque, rendering is on,
                // and the pixel is not in the left 8 columns if masking is active.
                if fg_sprite_zero
                    && self.sprite_zero_hit_possible
                    && self.mask.contains(Mask::RENDER_BACKGROUND)
                    && self.mask.contains(Mask::RENDER_SPRITES)
                {
                    let x = self.cycle - 1;
                    let left_clipping = !self.mask.contains(Mask::RENDER_BACKGROUND_LEFT)
                        || !self.mask.contains(Mask::RENDER_SPRITES_LEFT);
                    // x==255 is excluded per NES hardware spec
                    if x != 255 && (!left_clipping || x >= 8) {
                        self.status.insert(Status::SPRITE_ZHIT);
                    }
                }

                if fg_priority {
                    fg_palette << 2 | fg
                } else {
                    bg_palette << 2 | bg
                }
            }
        };

        // Palette lookup
        let color = self.get_color_from_palette(pixel, mapper);

        let x = self.cycle - 1;
        let y = self.scanline;
        if x >= 0 && x < 256 && y >= 0 && y < 240 {
            self.frame_buffer[y as usize * 256 + x as usize] = color;
        }
    }

    fn get_color_from_palette(&self, pixel: u8, mapper: &mut dyn Mapper) -> u32 {
        let addr = if (pixel & 0x03) == 0 { 0 } else { pixel };
        let pal_idx = self.ppu_read(0x3F00 + addr as u16, mapper) as usize;
        NES_PALETTE[pal_idx & 0x3F]
    }

    fn evaluate_sprites(&mut self, mapper: &mut dyn Mapper) {
        // Clear for next line
        self.scanline_sprites.clear();
        let sprite_height = if self.ctrl.contains(Control::SPRITE_SIZE) {
            16
        } else {
            8
        };
        // Evaluate for NEXT scanline
        let scanline = self.scanline;
        let target_line = scanline + 1;
        if target_line >= 240 {
            return;
        }

        let mut count = 0;
        self.sprite_zero_hit_possible = false; // Will be set if sprite 0 is in next line

        for i in 0..64 {
            let y = self.oam_data[i * 4] as i16;
            let diff = target_line - y - 1;
            if diff >= 0 && diff < sprite_height {
                if count < 8 {
                    if i == 0 {
                        self.sprite_zero_hit_possible = true;
                    }
                    self.scanline_sprites.push(i as u8);

                    // Fetch Data immediately (cheat)
                    let attr = self.oam_data[i * 4 + 2];
                    let tile_idx = self.oam_data[i * 4 + 1];

                    let (tile_addr_lo, tile_addr_hi) = if sprite_height == 8 {
                        // 8x8
                        let table = if self.ctrl.contains(Control::SPRITE_PATTERN) {
                            0x1000
                        } else {
                            0x0000
                        };
                        let row = if (attr & 0x80) != 0 { 7 - diff } else { diff };
                        let lo = table | (tile_idx as u16) << 4 | row as u16;
                        (lo, lo + 8)
                    } else {
                        // 8x16
                        // Logic: Even = $0000, Odd = $1000. Index = tile_idx & 0xFE
                        let table = ((tile_idx & 1) as u16) * 0x1000;
                        let idx = tile_idx & 0xFE;
                        let mut row = diff;
                        if (attr & 0x80) != 0 {
                            row = 15 - row;
                        }

                        let lo = if row < 8 {
                            table | (idx as u16) << 4 | row as u16
                        } else {
                            table | ((idx + 1) as u16) << 4 | (row - 8) as u16
                        };
                        (lo, lo + 8)
                    };

                    let mut pat_lo = self.ppu_read(tile_addr_lo, mapper);
                    let mut pat_hi = self.ppu_read(tile_addr_hi, mapper);

                    // Flip Horizontally?
                    if (attr & 0x40) != 0 {
                        // Reverse bits
                        pat_lo = pat_lo.reverse_bits();
                        pat_hi = pat_hi.reverse_bits();
                    }

                    self.sprite_shifter_pattern_lo[count] = pat_lo;
                    self.sprite_shifter_pattern_hi[count] = pat_hi;
                    count += 1;
                }
            }
        }
    }

    pub fn read(&mut self, addr: u16, mapper: &mut dyn Mapper) -> u8 {
        match addr & 0x0007 {
            0x0002 => self.read_status(),
            0x0004 => self.oam_data[self.oam_addr as usize],
            0x0007 => self.read_ppu_data(mapper),
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8, mapper: &mut dyn Mapper) {
        match addr & 0x0007 {
            0x0000 => self.write_ctrl(value),
            0x0001 => self.write_mask(value),
            0x0003 => self.oam_addr = value,
            0x0004 => {
                self.oam_data[self.oam_addr as usize] = value;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            0x0005 => self.write_scroll(value),
            0x0006 => self.write_ppu_addr(value),
            0x0007 => self.write_ppu_data(value, mapper),
            _ => {}
        }
    }
}
