use super::{Mapper, Mirroring};
use alloc::{vec, vec::Vec};

pub struct MMC1 {
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

impl MMC1 {
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
            control: 0x0C,
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
                self.control = val;
            }
            1 => {
                self.chr_bank_0 = val;
            }
            2 => {
                self.chr_bank_1 = val;
            }
            3 => {
                self.prg_bank = val;
            }
            _ => {}
        }
    }
}

impl Mapper for MMC1 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize],
            0x8000..=0xFFFF => {
                let prg_mode = (self.control >> 2) & 0x03;
                let offset = match prg_mode {
                    0 | 1 => {
                        let bank = (self.prg_bank & 0x0E) as usize % self.num_prg_banks;
                        (bank * 32768) + (addr - 0x8000) as usize
                    }
                    2 => {
                        if addr < 0xC000 {
                            (0 * 16384) + (addr - 0x8000) as usize
                        } else {
                            let bank = (self.prg_bank & 0x0F) as usize % self.num_prg_banks;
                            (bank * 16384) + (addr - 0xC000) as usize
                        }
                    }
                    3 => {
                        if addr < 0xC000 {
                            let bank = (self.prg_bank & 0x0F) as usize % self.num_prg_banks;
                            (bank * 16384) + (addr - 0x8000) as usize
                        } else {
                            let bank = self.num_prg_banks - 1;
                            (bank * 16384) + (addr - 0xC000) as usize
                        }
                    }
                    _ => 0,
                };
                if offset < self.prg_rom.len() {
                    self.prg_rom[offset]
                } else {
                    0
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
                if (data & 0x80) != 0 {
                    self.shift_reg = 0;
                    self.write_count = 0;
                    self.control |= 0x0C;
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
                let bank = (self.chr_bank_0 & 0x1E) as usize % (self.num_chr_banks / 2);
                (bank * 8192) + addr as usize
            }
            1 => {
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
                let bank = (self.chr_bank_0 & 0x1E) as usize % (self.num_chr_banks / 2);
                (bank * 8192) + addr as usize
            }
            1 => {
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
