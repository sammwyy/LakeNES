use super::Mapper;
use alloc::{vec, vec::Vec};

pub struct MMC5 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    ex_ram: [u8; 1024],

    // PRG/RAM registers
    prg_mode: u8,
    chr_mode: u8,
    ram_protect_a: u8,
    ram_protect_b: u8,
    ex_ram_mode: u8,
    mirroring_reg: u8,
    fill_tile: u8,
    fill_attr: u8,

    prg_regs: [u8; 5], // $5113 - $5117

    // CHR registers
    chr_regs_a: [u16; 8], // $5120 - $5127
    chr_regs_b: [u16; 4], // $5128 - $512B
    chr_high_bits: u16,  // $5130

    // Multiplier
    multiplicand: u8,
    multiplier: u8,

    // IRQ
    irq_target: u8,
    irq_enabled: bool,
    irq_pending: bool,
    irq_in_frame: bool,
    irq_counter: u8,
    last_addr: u16,
    addr_match_count: u8,

    // Split Scroll
    split_control: u8,
    split_y_scroll: u8,
    split_chr_page: u8,

    // Internal state
    last_chr_set: u8, // 0 for A, 1 for B
    scanline_fetching_sprites: bool,
    fetch_in_scanline: u16,
    
    // For Extended Attribute Mode (Ex1)
    extended_attrib_data: u8,
}

impl MMC5 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Self {
            prg_rom,
            chr_rom,
            prg_ram: vec![0; 65536],
            ex_ram: [0; 1024],

            prg_mode: 3,
            chr_mode: 0,
            ram_protect_a: 0,
            ram_protect_b: 0,
            ex_ram_mode: 0,
            mirroring_reg: 0,
            fill_tile: 0,
            fill_attr: 0,

            prg_regs: [0, 0, 0, 0, 0xFF],

            chr_regs_a: [0; 8],
            chr_regs_b: [0; 4],
            chr_high_bits: 0,

            multiplicand: 0,
            multiplier: 0,

            irq_target: 0,
            irq_enabled: false,
            irq_pending: false,
            irq_in_frame: false,
            irq_counter: 0,
            last_addr: 0,
            addr_match_count: 0,

            split_control: 0,
            split_y_scroll: 0,
            split_chr_page: 0,

            last_chr_set: 0,
            scanline_fetching_sprites: false,
            fetch_in_scanline: 0,

            extended_attrib_data: 0,
        }
    }

    fn is_ram_writable(&self) -> bool {
        self.ram_protect_a == 0b10 && self.ram_protect_b == 0b01
    }

    fn get_prg_offset(&self, addr: u16) -> (bool, usize) {
        let mode = self.prg_mode & 0x03;
        match mode {
            0 => { // 32k
                if addr >= 0x8000 {
                    let page = (self.prg_regs[4] & 0x7C) as usize;
                    (false, (page * 8192) | (addr as usize & 0x7FFF))
                } else { // 6000-7FFF
                    (true, ((self.prg_regs[0] & 0x07) as usize * 8192) | (addr as usize & 0x1FFF))
                }
            }
            1 => { // 16k
                if addr >= 0xC000 {
                    let page = (self.prg_regs[4] & 0x7E) as usize;
                    (false, (page * 8192) | (addr as usize & 0x3FFF))
                } else if addr >= 0x8000 {
                    let is_ram = (self.prg_regs[2] & 0x80) == 0;
                    let page = (self.prg_regs[2] & 0x7E) as usize;
                    (is_ram, (page * 8192) | (addr as usize & 0x3FFF))
                } else {
                    (true, ((self.prg_regs[0] & 0x07) as usize * 8192) | (addr as usize & 0x1FFF))
                }
            }
            2 => { // 16k + 8k
                if addr >= 0xE000 {
                    let page = (self.prg_regs[4] & 0x7F) as usize;
                    (false, (page * 8192) | (addr as usize & 0x1FFF))
                } else if addr >= 0xC000 {
                    let is_ram = (self.prg_regs[3] & 0x80) == 0;
                    let page = (self.prg_regs[3] & 0x7F) as usize;
                    (is_ram, (page * 8192) | (addr as usize & 0x1FFF))
                } else if addr >= 0x8000 {
                    let is_ram = (self.prg_regs[2] & 0x80) == 0;
                    let page = (self.prg_regs[2] & 0x7E) as usize;
                    (is_ram, (page * 8192) | (addr as usize & 0x3FFF))
                } else {
                    (true, ((self.prg_regs[0] & 0x07) as usize * 8192) | (addr as usize & 0x1FFF))
                }
            }
            3 => { // 8k
                let (bank, is_ram) = match addr {
                    0x6000..=0x7FFF => (self.prg_regs[0] & 0x07, true),
                    0x8000..=0x9FFF => (self.prg_regs[1] & 0x7F, (self.prg_regs[1] & 0x80) == 0),
                    0xA000..=0xBFFF => (self.prg_regs[2] & 0x7F, (self.prg_regs[2] & 0x80) == 0),
                    0xC000..=0xDFFF => (self.prg_regs[3] & 0x7F, (self.prg_regs[3] & 0x80) == 0),
                    0xE000..=0xFFFF => (self.prg_regs[4] & 0x7F, false),
                    _ => (0, false),
                };
                (is_ram, (bank as usize * 8192) | (addr as usize & 0x1FFF))
            }
            _ => (false, 0),
        }
    }

    fn read_chr_bank(&self, addr: u16) -> usize {
        let mode = self.chr_mode & 0x03;
        
        // Extended Attribute Mode (Ex1) overrides for BG fetches
        if self.ex_ram_mode == 1 && !self.scanline_fetching_sprites {
            let bank = (self.chr_high_bits | (self.extended_attrib_data as u16 & 0x3F)) as usize;
            return (bank * 4096) | (addr as usize & 0x0FFF);
        }

        let use_set_b = !self.scanline_fetching_sprites && self.last_chr_set == 1;
        let bank = if use_set_b {
            match mode {
                0 => self.chr_regs_b[3], // 8k: $512B
                1 => self.chr_regs_b[3], // 4k: $512B
                2 => self.chr_regs_b[if (addr & 0x0800) == 0 { 1 } else { 3 }], // 2k: $5129, $512B
                3 => self.chr_regs_b[(addr >> 10) as usize & 0x03], // 1k: $5128-$512B
                _ => 0,
            }
        } else {
            match mode {
                0 => self.chr_regs_a[7], // 8k: $5127
                1 => self.chr_regs_a[if addr < 0x1000 { 3 } else { 7 }], // 4k: $5123, $5127
                2 => self.chr_regs_a[(addr >> 11) as usize * 2 + 1], // 2k: $5121, $5123, $5125, $5127
                3 => self.chr_regs_a[(addr >> 10) as usize & 0x07], // 1k: $5120-$5127
                _ => 0,
            }
        };

        let bank_size = match mode {
            0 => 8192,
            1 => 4096,
            2 => 2048,
            3 => 1024,
            _ => 1024,
        };

        (bank as usize * bank_size) | (addr as usize % bank_size)
    }
}

impl Mapper for MMC5 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        match addr {
            0x5015 => 0, // Sound stub
            0x5204 => {
                let mut status = 0;
                if self.irq_pending { status |= 0x80; }
                if self.irq_in_frame { status |= 0x40; }
                self.irq_pending = false;
                status
            }
            0x5205 => ((self.multiplicand as u16 * self.multiplier as u16) & 0xFF) as u8,
            0x5206 => ((self.multiplicand as u16 * self.multiplier as u16) >> 8) as u8,
            0x5C00..=0x5FFF => {
                if self.ex_ram_mode >= 2 {
                    self.ex_ram[(addr - 0x5C00) as usize]
                } else {
                    0
                }
            }
            0x6000..=0xFFFF => {
                let (is_ram, offset) = self.get_prg_offset(addr);
                if is_ram {
                    let len = self.prg_ram.len();
                    self.prg_ram[offset % len]
                } else {
                    let len = self.prg_rom.len();
                    self.prg_rom[offset % len]
                }
            }
            _ => 0,
        }
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        match addr {
            0x5100 => self.prg_mode = data & 0x03,
            0x5101 => self.chr_mode = data & 0x03,
            0x5102 => self.ram_protect_a = data & 0x03,
            0x5103 => self.ram_protect_b = data & 0x03,
            0x5104 => self.ex_ram_mode = data & 0x03,
            0x5105 => self.mirroring_reg = data,
            0x5106 => self.fill_tile = data,
            0x5107 => self.fill_attr = data & 0x03,
            0x5113..=0x5117 => {
                self.prg_regs[(addr - 0x5113) as usize] = data;
            }
            0x5120..=0x5127 => {
                self.chr_regs_a[(addr - 0x5120) as usize] = data as u16 | self.chr_high_bits;
                self.last_chr_set = 0;
            }
            0x5128..=0x512B => {
                self.chr_regs_b[(addr - 0x5128) as usize] = data as u16 | self.chr_high_bits;
                self.last_chr_set = 1;
            }
            0x5130 => self.chr_high_bits = (data as u16 & 0x03) << 8,
            0x5200 => self.split_control = data,
            0x5201 => self.split_y_scroll = data,
            0x5202 => self.split_chr_page = data,
            0x5203 => self.irq_target = data,
            0x5204 => self.irq_enabled = (data & 0x80) != 0,
            0x5205 => self.multiplicand = data,
            0x5206 => self.multiplier = data,
            0x5C00..=0x5FFF => {
                match self.ex_ram_mode {
                    0 | 1 => {
                        if self.irq_in_frame {
                            self.ex_ram[(addr - 0x5C00) as usize] = data;
                        }
                    }
                    2 => self.ex_ram[(addr - 0x5C00) as usize] = data,
                    _ => {}
                }
            }
            0x6000..=0xFFFF => {
                if self.is_ram_writable() {
                    let (is_ram, offset) = self.get_prg_offset(addr);
                    if is_ram {
                        let len = self.prg_ram.len();
                        self.prg_ram[offset % len] = data;
                    }
                }
            }
            _ => {}
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        if self.chr_rom.is_empty() {
            return 0;
        }
        let offset = self.read_chr_bank(addr);
        self.chr_rom[offset % self.chr_rom.len()]
    }

    fn write_chr(&mut self, _addr: u16, _data: u8) {}

    fn irq_flag(&self) -> bool {
        self.irq_pending && self.irq_enabled
    }

    fn ppu_bus_address(&mut self, addr: u16) {
        let addr = addr & 0x3FFF;
        
        // IRQ Detection: 3 identical nametable fetches in a row at end of scanline
        if addr >= 0x2000 && addr <= 0x2FFF && addr == self.last_addr {
            self.addr_match_count += 1;
            if self.addr_match_count == 3 {
                if !self.irq_in_frame {
                    self.irq_in_frame = true;
                    self.irq_counter = 0;
                    self.irq_pending = false;
                } else {
                    self.irq_counter += 1;
                    if self.irq_counter == self.irq_target {
                        self.irq_pending = true;
                    }
                }
                self.fetch_in_scanline = 0;
            }
        } else {
            self.addr_match_count = 0;
        }
        self.last_addr = addr;

        // Roughly increment fetch counter for tiles
        if addr <= 0x1FFF {
            // Pattern fetch
        } else if addr >= 0x2000 && addr <= 0x2FFF {
            // NT/AT fetch
            self.fetch_in_scanline += 1;
        }

        if addr >= 0x3000 {
            // Potential end of frame detection (palette)
        }
    }

    fn read_ppu(&mut self, addr: u16, vram: &[u8]) -> Option<u8> {
        if addr >= 0x2000 && addr <= 0x2FFF {
            let slot = (addr >> 10) & 0x03;
            let mode = (self.mirroring_reg >> (slot * 2)) & 0x03;
            
            match mode {
                0 => Some(vram[(addr & 0x03FF) as usize]), // NTA
                1 => Some(vram[(0x0400 | (addr & 0x03FF)) as usize]), // NTB
                2 => {
                    if self.ex_ram_mode < 2 {
                        let data = self.ex_ram[(addr & 0x03FF) as usize];
                        if self.ex_ram_mode == 1 {
                             self.extended_attrib_data = data;
                        }
                        Some(data)
                    } else {
                        Some(0)
                    }
                }
                3 => { // Fill Mode
                    if (addr & 0x3FFF) >= 0x23C0 && (addr & 0x03FF) >= 0x03C0 {
                        Some(self.fill_attr | (self.fill_attr << 2) | (self.fill_attr << 4) | (self.fill_attr << 6))
                    } else {
                        Some(self.fill_tile)
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn write_ppu(&mut self, addr: u16, data: u8, vram: &mut [u8]) -> bool {
        if addr >= 0x2000 && addr <= 0x2FFF {
            let slot = (addr >> 10) & 0x03;
            let mode = (self.mirroring_reg >> (slot * 2)) & 0x03;
            match mode {
                0 => { vram[(addr & 0x03FF) as usize] = data; true }
                1 => { vram[(0x0400 | (addr & 0x03FF)) as usize] = data; true }
                2 => {
                    if self.ex_ram_mode < 2 {
                        self.ex_ram[(addr & 0x03FF) as usize] = data;
                    }
                    true
                }
                _ => true,
            }
        } else {
            false
        }
    }
}
