use crate::bus::BusDevice;

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

pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub vram: [u8; 2048],
    pub palette_table: [u8; 32],

    ctrl: u8,
    mask: u8,
    status: u8,
    pub oam_data: [u8; 256],
    pub oam_addr: u8,
    pub scroll_x: u8,
    pub scroll_y: u8,
    addr: u16,
    pub frame_buffer: [u8; 256 * 240 * 3],
    pub cycle: u16,
    pub scanline: u16,
    pub bg_opaque_pixels: [bool; 256],

    pub nmi_interrupt: bool,
    address_latch: bool,
    data_buffer: u8,
}

// memory map
impl PPU {
    pub fn new(chr_rom: Vec<u8>) -> Self {
        Self {
            chr_rom,
            vram: [0; 2048],
            palette_table: [0; 32],
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            oam_data: [0; 256],
            scroll_x: 0,
            scroll_y: 0,
            addr: 0,
            frame_buffer: [0; 256 * 240 * 3],
            address_latch: false,
            data_buffer: 0,
            cycle: 0,
            scanline: 0,
            nmi_interrupt: false,
            bg_opaque_pixels: [false; 256],
        }
    }

    fn write_data(&mut self, value: u8) {
        let addr = self.addr & 0x3FFF;

        match addr {
            0x0000..=0x1FFF => {}
            0x2000..=0x3EFF => {
                let vram_index = (addr & 0x07FF) as usize;
                self.vram[vram_index] = value;
            }
            0x3F00..=0x3FFF => {
                let mut p_addr = (addr & 0x1F) as usize;
                if p_addr >= 0x10 && p_addr % 4 == 0 {
                    p_addr -= 0x10;
                }
                self.palette_table[p_addr] = value;
            }
            _ => {}
        }
        self.increment_addr();
    }

    fn read_data(&mut self) -> u8 {
        let addr = self.addr & 0x3FFF;
        self.increment_addr();

        match addr {
            0x0000..=0x3EFF => {
                let res = self.data_buffer;
                let mirrored_addr = if addr >= 0x2000 {
                    (addr & 0x07FF) as usize
                } else {
                    addr as usize
                };

                if addr < 0x2000 {
                    self.data_buffer = self.chr_rom[mirrored_addr];
                } else {
                    self.data_buffer = self.vram[mirrored_addr];
                }
                res
            }
            0x3F00..=0x3FFF => {
                self.data_buffer = self.vram[(addr & 0x07FF) as usize];
                self.palette_table[(addr & 0x1F) as usize]
            }
            _ => 0,
        }
    }

    fn increment_addr(&mut self) {
        let inc = if (self.ctrl & 0x04) == 0 { 1 } else { 32 };
        self.addr = self.addr.wrapping_add(inc) & 0x3FFF;
    }

    pub fn write_address(&mut self, value: u8) {
        if !self.address_latch {
            self.addr = (self.addr & 0x00FF) | ((value as u16 & 0x3F) << 8);
        } else {
            self.addr = (self.addr & 0xFF00) | (value as u16);
        }
        self.address_latch = !self.address_latch;
    }
}

// rendering
impl PPU {
    pub fn render_scanline(&mut self) {
        let scanline = self.scanline as usize;
        if scanline >= 240 {
            return;
        }

        let rendering_enabled = (self.mask & 0x18) != 0;
        if !rendering_enabled {
            return;
        }

        let bg_enabled = (self.mask & 0x08) != 0;
        if bg_enabled {
            self.render_bg_line(scanline);
        }

        let sprites_enabled = (self.mask & 0x10) != 0;
        if sprites_enabled {
            self.render_sprite_line(scanline);
        }
    }
    fn render_bg_line(&mut self, y: usize) {
        let bank = if (self.ctrl & 0x10) != 0 {
            0x1000
        } else {
            0x0000
        };
        let base_nt = (self.ctrl & 0x03) as usize;

        for x in 0..256 {
            let total_x = x + self.scroll_x as usize;
            let total_y = y + self.scroll_y as usize;

            let mut nt_idx = base_nt;
            if total_x >= 256 {
                nt_idx ^= 0x01;
            }
            if total_y >= 240 {
                nt_idx ^= 0x02;
            }

            let physical_nt = match nt_idx {
                0 => 0,
                1 => 0,
                2 => 1,
                3 => 1,
                _ => 0,
            };

            let tile_x = (total_x % 256) / 8;
            let tile_y = (total_y % 240) / 8;

            let vram_idx = (physical_nt * 1024) + (tile_y * 32) + tile_x;

            if vram_idx >= 2048 {
                continue;
            }

            let tile_id = self.vram[vram_idx] as usize;
            let palette_idx = self.get_background_palette_idx(physical_nt, tile_x, tile_y);

            let pixel_x = 7 - (total_x % 8);
            let pixel_y = total_y % 8;

            let low = self.chr_rom[bank + tile_id * 16 + pixel_y];
            let high = self.chr_rom[bank + tile_id * 16 + pixel_y + 8];
            let pixel_val = ((low >> pixel_x) & 1) | (((high >> pixel_x) & 1) << 1);

            self.bg_opaque_pixels[x] = pixel_val != 0;

            let color_hex = self.get_color_from_palette(palette_idx, pixel_val);
            self.set_pixel(x as u32, y as u32, color_hex);
        }
    }

    fn get_background_palette_idx(&self, nt: usize, tile_x: usize, tile_y: usize) -> u8 {
        let attr_addr = (nt * 1024) + 960 + (tile_y / 4) * 8 + (tile_x / 4);

        if attr_addr >= 2048 {
            return 0;
        }

        let attribute_byte = self.vram[attr_addr];
        let quadrant_x = (tile_x % 4) / 2;
        let quadrant_y = (tile_y % 4) / 2;

        match (quadrant_y, quadrant_x) {
            (0, 0) => attribute_byte & 0b11,
            (0, 1) => (attribute_byte >> 2) & 0b11,
            (1, 0) => (attribute_byte >> 4) & 0b11,
            (1, 1) => (attribute_byte >> 6) & 0b11,
            _ => 0,
        }
    }

    fn get_color_from_palette(&self, palette_idx: u8, pixel_val: u8) -> u32 {
        if pixel_val == 0 {
            NES_PALETTE[self.palette_table[0] as usize]
        } else {
            let palette_start = palette_idx as usize * 4;
            let color_idx = self.palette_table[palette_start + pixel_val as usize] as usize;
            NES_PALETTE[color_idx]
        }
    }

    fn render_sprite_line(&mut self, y: usize) {
        let scanline = y as i32;
        let rendering_enabled = (self.mask & 0x18) == 0x18;

        for i in (0..64).rev() {
            let base = i * 4;
            let sprite_y = self.oam_data[base] as i32 + 1;
            let tile_id = self.oam_data[base + 1] as usize;
            let attributes = self.oam_data[base + 2];
            let sprite_x = self.oam_data[base + 3] as u32;

            if scanline >= sprite_y && scanline < sprite_y + 8 {
                let bank = if (self.ctrl & 0x08) != 0 {
                    0x1000
                } else {
                    0x0000
                };
                let flip_v = (attributes & 0x80) != 0;
                let flip_h = (attributes & 0x40) != 0;
                let palette_idx = attributes & 0x03;
                let priority = (attributes & 0x20) != 0;

                let mut row = scanline - sprite_y;
                if flip_v {
                    row = 7 - row;
                }

                let low = self.chr_rom[bank + tile_id * 16 + row as usize];
                let high = self.chr_rom[bank + tile_id * 16 + row as usize + 8];

                for col in 0..8 {
                    let x_offset = if flip_h { 7 - col } else { col };
                    let shift = 7 - x_offset;
                    let pixel_val = ((low >> shift) & 0x01) | (((high >> shift) & 0x01) << 1);

                    let final_x = sprite_x + col as u32;

                    if final_x < 256 && pixel_val != 0 {
                        if i == 0 && rendering_enabled && (self.status & 0x40) == 0 {
                            if self.bg_opaque_pixels[final_x as usize] {
                                let hide_left_8 =
                                    (self.mask & 0x02) == 0 || (self.mask & 0x04) == 0;
                                if !(hide_left_8 && final_x < 8) && final_x < 255 {
                                    self.status |= 0x40;
                                }
                            }
                        }

                        let bg_transparent = self.is_pixel_transparent(final_x, y as u32);

                        if !priority || bg_transparent {
                            let color_idx = self.palette_table
                                [0x10 + (palette_idx as usize * 4) + pixel_val as usize];
                            self.set_pixel(
                                final_x,
                                scanline as u32,
                                NES_PALETTE[color_idx as usize],
                            );
                        }
                    }
                }
            }
        }
    }

    fn is_pixel_transparent(&self, x: u32, y: u32) -> bool {
        let index = ((y * 256) + x) as usize * 3;
        let r = self.frame_buffer[index];
        let g = self.frame_buffer[index + 1];
        let b = self.frame_buffer[index + 2];

        let bg_color = NES_PALETTE[self.palette_table[0] as usize];
        r == ((bg_color >> 16) & 0xFF) as u8
            && g == ((bg_color >> 8) & 0xFF) as u8
            && b == (bg_color & 0xFF) as u8
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        let index = ((y * 256) + x) as usize * 3;
        if index + 2 < self.frame_buffer.len() {
            self.frame_buffer[index] = ((color >> 16) & 0xFF) as u8; // R
            self.frame_buffer[index + 1] = ((color >> 8) & 0xFF) as u8; // G
            self.frame_buffer[index + 2] = (color & 0xFF) as u8; // B
        }
    }
}

// cycles
impl PPU {
    pub fn step(&mut self) {
        self.cycle += 1;

        // HBlank
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;

            // VBlank
            if self.scanline > 261 {
                self.scanline = 0;
                self.status &= 0x7F; // Clear VBlank
                self.status &= 0xBF; // Clear Sprite 0 Hit
                self.nmi_interrupt = false;
            }
        }

        match self.scanline {
            0..=239 => {
                if self.cycle == 256 {
                    self.render_scanline();
                }
            }
            241 => {
                if self.cycle == 1 {
                    self.status |= 0x80; // Set VBlank Flag
                    if (self.ctrl & 0x80) != 0 {
                        self.nmi_interrupt = true;
                    }
                }
            }
            _ => {}
        }
    }
}

// BusDevice implementation
impl BusDevice for PPU {
    fn read(&mut self, addr: u16) -> u8 {
        match addr % 8 {
            2 => {
                // PPUSTATUS
                let res = self.status;
                self.status &= 0x7F; // VBlank
                self.address_latch = false;
                res
            }
            4 => self.oam_data[self.oam_addr as usize], // OAMDATA
            7 => self.read_data(),                      // PPUDATA
            _ => 0,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr % 8 {
            0 => self.ctrl = value,
            1 => self.mask = value,
            3 => self.oam_addr = value, // OAMADDR
            4 => {
                // OAMDATA
                self.oam_data[self.oam_addr as usize] = value;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            5 => {
                // PPUSCROLL
                if !self.address_latch {
                    self.scroll_x = value;
                } else {
                    self.scroll_y = value;
                }
                self.address_latch = !self.address_latch;
            }
            6 => self.write_address(value), // PPUADDR
            7 => self.write_data(value),    // PPUDATA
            _ => {}
        }
    }
}
