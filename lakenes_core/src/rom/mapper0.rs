use super::{Mapper, Mirroring};
use alloc::vec::Vec;

pub struct Mapper0 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_banks: usize,
    _chr_banks: usize,
    mirroring: Mirroring,
}

impl Mapper0 {
    pub fn new(
        prg_rom: Vec<u8>,
        chr_rom: Vec<u8>,
        prg_banks: usize,
        chr_banks: usize,
        mirroring: Mirroring,
    ) -> Self {
        Self {
            prg_rom,
            chr_rom,
            prg_banks,
            _chr_banks: chr_banks,
            mirroring,
        }
    }
}

impl Mapper for Mapper0 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        if addr >= 0x8000 {
            let mut idx = addr as usize - 0x8000;
            if self.prg_banks == 1 {
                idx = idx % 16384; // Mirror 16k to 32k
            }
            if idx < self.prg_rom.len() {
                return self.prg_rom[idx];
            }
        }
        0
    }

    fn write_prg(&mut self, _addr: u16, _data: u8) {
        // Mapper 0 has no registers
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        if (addr as usize) < self.chr_rom.len() {
            return self.chr_rom[addr as usize];
        }
        0
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        // If CHR is RAM (0 banks usually means we allocated RAM in ROM loader? Or handled here?)
        // Standard NROM with CHR-RAM allows writing.
        // We need to know if it's RAM.
        // Usually, if chr_banks == 0 (from header) it implies RAM.
        // But here we received a Vec. If it is mutable/RAM, we should allow write.
        // For simplicity, if we have space, write it.
        if (addr as usize) < self.chr_rom.len() {
            self.chr_rom[addr as usize] = data;
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
