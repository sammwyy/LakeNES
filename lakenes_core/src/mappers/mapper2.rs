use super::{Mapper, Mirroring};
use alloc::vec::Vec;

pub struct Mapper2 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_banks: usize,
    prg_bank_select: usize,
    last_bank_offset: usize,
    mirroring: Mirroring,
}

impl Mapper2 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, prg_banks: usize, mirroring: Mirroring) -> Self {
        let last_bank_offset = (prg_banks.saturating_sub(1)) * 16384;
        Self {
            prg_rom,
            chr_rom,
            prg_banks,
            prg_bank_select: 0,
            last_bank_offset,
            mirroring,
        }
    }
}

impl Mapper for Mapper2 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        if addr >= 0x8000 {
            if addr < 0xC000 {
                // Switchable bank
                let idx = (self.prg_bank_select * 16384) + (addr as usize - 0x8000);
                if idx < self.prg_rom.len() {
                    return self.prg_rom[idx];
                }
            } else {
                // Fixed last bank
                let idx = self.last_bank_offset + (addr as usize - 0xC000);
                if idx < self.prg_rom.len() {
                    return self.prg_rom[idx];
                }
            }
        }
        0
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        if addr >= 0x8000 {
            // UNROM uses a bank select register.
            // We should mask it to avoid indexing out of bounds.
            debug_assert!(self.prg_banks.is_power_of_two());
            self.prg_bank_select = (data as usize) & (self.prg_banks - 1);
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
