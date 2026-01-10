pub fn read_register(&mut self, addr: u16) -> u8 {
    match addr % 8 {
        2 => {
            let res = self.status;
            self.status &= 0x7F;
            self.address_latch = false;
            res
        }
        7 => self.read_data(),
        _ => 0,
    }
}

pub fn write_register(&mut self, addr: u16, value: u8) {
    match addr % 8 {
        0 => self.ctrl = value,
        1 => self.mask = value,
        6 => {
            if !self.address_latch {
                self.addr = (self.addr & 0x00FF) | ((value as u16) << 8);
            } else {
                self.addr = (self.addr & 0xFF00) | (value as u16);
            }
            self.address_latch = !self.address_latch;
        }
        7 => self.write_data(value),
        _ => {}
    }
}

pub fn render_pattern_table(&mut self) {
    for tile_n in 0..256 {
        for row in 0..8 {
            let mut low_byte = self.chr_rom[tile_n * 16 + row];
            let mut high_byte = self.chr_rom[tile_n * 16 + row + 8];

            for col in (0..8).rev() {
                let pixel_color_index = (low_byte & 0x01) | ((high_byte & 0x01) << 1);
                low_byte >>= 1;
                high_byte >>= 1;

                let color_hex = match pixel_color_index {
                    0 => NES_PALETTE[0x0F],
                    1 => NES_PALETTE[0x21],
                    2 => NES_PALETTE[0x16],
                    _ => NES_PALETTE[0x30],
                };

                let x = (tile_n % 16) * 8 + col;
                let y = (tile_n / 16) * 8 + row;
                self.set_pixel(x as u32, y as u32, color_hex);
            }
        }
    }
}

pub fn render_background(&mut self) {
    let bank = if (self.ctrl & 0x10) != 0 {
        0x1000
    } else {
        0x0000
    };
    for nt in 0..2 {
        let nt_offset = nt * 1024;

        for i in 0..960 {
            let tile_id = self.vram[nt_offset + i] as usize;
            let tile_x = (i % 32) + (nt * 32);
            let tile_y = i / 32;

            let palette_idx = self.get_background_palette_idx(tile_x, tile_y);

            for row in 0..8 {
                let mut low_byte = self.chr_rom[bank + tile_id * 16 + row];
                let mut high_byte = self.chr_rom[bank + tile_id * 16 + row + 8];

                for col in (0..8).rev() {
                    let pixel_val = (low_byte & 0x01) | ((high_byte & 0x01) << 1);
                    low_byte >>= 1;
                    high_byte >>= 1;

                    let color_hex = if pixel_val == 0 {
                        let color_idx = self.palette_table[0] as usize;
                        NES_PALETTE[color_idx]
                    } else {
                        let palette_start = palette_idx as usize * 4;
                        let color_idx =
                            self.palette_table[palette_start + pixel_val as usize] as usize;
                        NES_PALETTE[color_idx]
                    };

                    let screen_x = (tile_x as i32 * 8 + col as i32) - self.scroll_x as i32;
                    let screen_y = (tile_y as i32 * 8 + row as i32) - self.scroll_y as i32;

                    if screen_x >= 0 && screen_x < 256 && screen_y >= 0 && screen_y < 240 {
                        self.set_pixel(screen_x as u32, screen_y as u32, color_hex);
                    }
                }
            }
        }
    }
}

pub fn render_sprites(&mut self) {
    for i in (0..64).rev() {
        let base = i * 4;
        let tile_y = self.oam_data[base] as u32;
        let tile_id = self.oam_data[base + 1] as usize;
        let attributes = self.oam_data[base + 2];
        let tile_x = self.oam_data[base + 3] as u32;

        if tile_y >= 240 {
            continue;
        }

        let bank = if (self.ctrl & 0x08) != 0 {
            0x1000
        } else {
            0x0000
        };
        let flip_v = (attributes & 0x80) != 0;
        let flip_h = (attributes & 0x40) != 0;
        let palette_idx = attributes & 0x03;

        for row in 0..8 {
            let mut y_offset = row;
            if flip_v {
                y_offset = 7 - row;
            }

            let mut low_byte = self.chr_rom[bank + tile_id * 16 + y_offset];
            let mut high_byte = self.chr_rom[bank + tile_id * 16 + y_offset + 8];

            for col in (0..8).rev() {
                let mut x_offset = col;
                if flip_h {
                    x_offset = 7 - col;
                }

                let pixel_val = (low_byte & 0x01) | ((high_byte & 0x01) << 1);
                low_byte >>= 1;
                high_byte >>= 1;

                if pixel_val != 0 {
                    let color_idx =
                        self.palette_table[0x10 + (palette_idx as usize * 4) + pixel_val as usize];
                    let color_hex = NES_PALETTE[color_idx as usize];

                    self.set_pixel(tile_x + x_offset as u32, tile_y + 1 + row as u32, color_hex);
                }
            }
        }
    }
}

fn check_sprite_0_hit(&mut self) {
    if (self.status & 0x40) != 0 || (self.mask & 0x18) != 0x18 {
        return;
    }

    let sprite_0_y = self.oam_data[0] as u16 + 1;
    let sprite_0_x = self.oam_data[3] as u16;

    if self.scanline == sprite_0_y && self.cycle >= sprite_0_x && self.cycle < sprite_0_x + 8 {
        self.status |= 0x40;
    }
}
