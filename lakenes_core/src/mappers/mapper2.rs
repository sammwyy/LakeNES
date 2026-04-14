use super::{Mapper, Mirroring};
use alloc::vec::Vec;

pub struct Mapper2 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_banks: usize,
    prg_bank_select: usize,
    mirroring: Mirroring,
}

impl Mapper2 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, prg_banks: usize, mirroring: Mirroring) -> Self {
        Self {
            prg_rom,
            chr_rom,
            prg_banks,
            prg_bank_select: 0,
            mirroring,
        }
    }
}

impl Mapper for Mapper2 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        if addr >= 0x8000 {
            let bank_len = 16384;
            let final_bank_offset = (self.prg_banks - 1) * bank_len;

            if addr < 0xC000 {
                // Switchable bank
                let offset = (addr - 0x8000) as usize;
                let idx = (self.prg_bank_select * bank_len) + offset;
                if idx < self.prg_rom.len() {
                    return self.prg_rom[idx];
                }
            } else {
                // Fixed last bank
                let offset = (addr - 0xC000) as usize;
                let idx = final_bank_offset + offset;
                if idx < self.prg_rom.len() {
                    return self.prg_rom[idx];
                }
            }
        }
        0
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        if addr >= 0x8000 {
            self.prg_bank_select = data as usize;
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        if (addr as usize) < self.chr_rom.len() {
            return self.chr_rom[addr as usize];
        }
        0
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        if (addr as usize) < self.chr_rom.len() {
            self.chr_rom[addr as usize] = data;
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
