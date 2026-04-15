use super::{Mapper, Mirroring};
use alloc::vec::Vec;

pub struct Mapper3 {
    prg_rom: Vec<u8>,
    chr: Vec<u8>,
    chr_bank: usize,
    chr_is_ram: bool,
    prg_banks: usize,
    mirroring: Mirroring,
}

impl Mapper3 {
    pub fn new(prg_rom: Vec<u8>, chr: Vec<u8>, prg_banks: usize, chr_is_ram: bool, mirroring: Mirroring) -> Self {
        Self {
            prg_rom,
            chr,
            chr_bank: 0,
            chr_is_ram,
            prg_banks,
            mirroring,
        }
    }
}

impl Mapper for Mapper3 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        if addr < 0x8000 {
            return 0;
        }
        let mut idx = (addr as usize) - 0x8000;
        if self.prg_banks == 1 {
            idx %= 16 * 1024;
        }
        self.prg_rom.get(idx).copied().unwrap_or(0)
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        if addr >= 0x8000 {
            self.chr_bank = (data as usize) & 0x03;
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        let bank_base = self.chr_bank * 8 * 1024;
        let idx = bank_base + (addr as usize & 0x1FFF);
        self.chr.get(idx % self.chr.len()).copied().unwrap_or(0)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        if !self.chr_is_ram || self.chr.is_empty() {
            return;
        }
        let idx = addr as usize & 0x1FFF;
        if idx < self.chr.len() {
            self.chr[idx] = data;
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
