use super::{Mapper, Mirroring};
use alloc::{vec, vec::Vec};

pub struct Mapper1 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_ram: Vec<u8>,

    // MMC1 State
    shift_reg: u8,
    control: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,

    // Helper to count writes
    write_count: u8,

    num_prg_banks: usize,
    num_chr_banks: usize,
}

impl Mapper1 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        let num_prg_banks = prg_rom.len() / 16384;
        let num_chr_banks = if !chr_rom.is_empty() {
            chr_rom.len() / 4096
        } else {
            0
        };

        let mut m = Self {
            prg_rom,
            chr_rom,
            prg_ram: vec![0; 8192],
            shift_reg: 0,
            control: 0x0C, // Default: Mode 3 (16k PRG fixed low/switch high? No, usually fixed last)
            // 0x0C = 01100 => Mirr: Vertical? (00), PRG Size 16k (1), CHR Size 4k (1) ???
            // Control:
            // CPPMM
            // M: Mirroring (0: 1ScA, 1: 1ScB, 2: Vert, 3: Horiz)
            // P: PRG Size (0: 32k, 1: 16k)
            // C: CHR Size (0: 8k, 1: 4k)
            // Default 0x1E?
            // Let's init to 0x10 (PRG 16k, CHR 8k?) or 0x00.
            // Power up state of shift reg/control is technically random/undefined or specific.
            // Usually Control starts 0x0C or similar.
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
            write_count: 0,

            num_prg_banks,
            num_chr_banks,
        };
        m.control = 0x0C;
        m
    }

    fn load_register(&mut self, addr: u16, val: u8) {
        match (addr >> 13) & 0x03 {
            0 => {
                // $8000 - $9FFF: Control
                self.control = val;
                // Mirroring handling should be propagated if we had a mechanism
            }
            1 => {
                // $A000 - $BFFF: CHR Bank 0
                self.chr_bank_0 = val;
            }
            2 => {
                // $C000 - $DFFF: CHR Bank 1
                self.chr_bank_1 = val;
            }
            3 => {
                // $E000 - $FFFF: PRG Bank
                self.prg_bank = val;
            }
            _ => {}
        }
    }
}

impl Mapper for Mapper1 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize],
            0x8000..=0xFFFF => {
                let prg_mode = (self.control >> 2) & 0x03;
                let offset = match prg_mode {
                    0 | 1 => {
                        // 32KB Mode
                        // Bank is (self.prg_bank & 0x0E) (ignore low bit)
                        let bank = (self.prg_bank & 0x0E) as usize % self.num_prg_banks;
                        (bank * 32768) + (addr - 0x8000) as usize
                    }
                    2 => {
                        // 16KB Mode: Fix first bank at $8000, Switch $C000
                        if addr < 0xC000 {
                            // Fixed first bank (0)
                            (0 * 16384) + (addr - 0x8000) as usize
                        } else {
                            // Switched bank
                            let bank = (self.prg_bank & 0x0F) as usize % self.num_prg_banks;
                            (bank * 16384) + (addr - 0xC000) as usize
                        }
                    }
                    3 => {
                        // 16KB Mode: Fix last bank at $C000, Switch $8000
                        // This is the standard "Unix" bank mode.
                        if addr < 0xC000 {
                            // Switched bank
                            let bank = (self.prg_bank & 0x0F) as usize % self.num_prg_banks;
                            (bank * 16384) + (addr - 0x8000) as usize
                        } else {
                            // Fixed last bank
                            let bank = self.num_prg_banks - 1;
                            (bank * 16384) + (addr - 0xC000) as usize
                        }
                    }
                    _ => 0,
                };
                // Safety check
                if offset < self.prg_rom.len() {
                    self.prg_rom[offset]
                } else {
                    0 // Logic error or empty ROM
                }
            }
            _ => 0,
        }
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        match addr {
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000) as usize] = data;
            }
            0x8000..=0xFFFF => {
                // Load Shift Register
                if (data & 0x80) != 0 {
                    // Reset
                    self.shift_reg = 0;
                    self.write_count = 0;
                    self.control |= 0x0C; // Reset control slightly? Docs say logic 3/4.
                } else {
                    self.shift_reg = (self.shift_reg >> 1) | ((data & 0x01) << 4);
                    self.write_count += 1;
                    if self.write_count == 5 {
                        self.load_register(addr, self.shift_reg);
                        self.shift_reg = 0;
                        self.write_count = 0;
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

        let chr_mode = (self.control >> 4) & 0x01;
        let offset = match chr_mode {
            0 => {
                // 8KB Mode
                let bank = (self.chr_bank_0 & 0x1E) as usize % (self.num_chr_banks / 2); // 8KB chunks
                (bank * 8192) + addr as usize
            }
            1 => {
                // 4KB Mode
                if addr < 0x1000 {
                    // Bank 0
                    let bank = self.chr_bank_0 as usize % self.num_chr_banks;
                    (bank * 4096) + addr as usize
                } else {
                    // Bank 1
                    let bank = self.chr_bank_1 as usize % self.num_chr_banks;
                    (bank * 4096) + (addr - 0x1000) as usize
                }
            }
            _ => 0,
        };

        // If using CHR RAM (no ROM), usually 8KB on board.
        // But mapper structure assumes ROM provided.
        // If chr_rom.len() > 0, read from it.
        // If 0, use valid RAM? (Mapper 1 supports CHR RAM).
        if offset < self.chr_rom.len() {
            self.chr_rom[offset]
        } else {
            0
        }
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        if self.chr_rom.is_empty() {
            return;
        }

        let chr_mode = (self.control >> 4) & 0x01;
        let offset = match chr_mode {
            0 => {
                // 8KB Mode
                let bank = (self.chr_bank_0 & 0x1E) as usize % (self.num_chr_banks / 2);
                (bank * 8192) + addr as usize
            }
            1 => {
                // 4KB Mode
                if addr < 0x1000 {
                    let bank = self.chr_bank_0 as usize % self.num_chr_banks;
                    (bank * 4096) + addr as usize
                } else {
                    let bank = self.chr_bank_1 as usize % self.num_chr_banks;
                    (bank * 4096) + (addr - 0x1000) as usize
                }
            }
            _ => 0,
        };

        if offset < self.chr_rom.len() {
            self.chr_rom[offset] = data;
        }
    }

    fn mirroring(&self) -> Mirroring {
        match self.control & 0x03 {
            0 => Mirroring::OneScreenLow,
            1 => Mirroring::OneScreenHigh,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => Mirroring::Vertical,
        }
    }
}
