use super::{Mapper, Mirroring};
use alloc::vec::Vec;

/// MMC4 (iNES Mapper 10) - Nintendo FxROM boards
/// Used by Fire Emblem, Fire Emblem Gaiden, and Famicom Wars.
///
/// Key differences from MMC2 (PRG only):
/// - PRG banks are 16 KiB (not 8 KiB): $8000-$BFFF switchable, $C000-$FFFF fixed to last bank
/// - PRG RAM at $6000-$7FFF
/// CHR latch behavior is IDENTICAL to MMC2.
pub struct MMC4 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_ram: Vec<u8>,

    /// Currently selected 16K PRG bank mapped to $8000-$BFFF.
    prg_bank: usize,

    /// CHR bank values loaded into the 4 latch registers:
    ///   [0] = $B000 register: 4K CHR for $0000 when latch_0 == $FD
    ///   [1] = $C000 register: 4K CHR for $0000 when latch_0 == $FE
    ///   [2] = $D000 register: 4K CHR for $1000 when latch_1 == $FD
    ///   [3] = $E000 register: 4K CHR for $1000 when latch_1 == $FE
    chr_banks: [usize; 4],

    /// Latch selector for PPU $0000-$0FFF. Initialized to $FE on reset.
    latch_0: u8,
    /// Latch selector for PPU $1000-$1FFF. Initialized to $FE on reset.
    latch_1: u8,

    mirroring: Mirroring,

    num_prg_banks_16k: usize,
    num_chr_banks_4k: usize,
}

impl MMC4 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, initial_mirroring: Mirroring) -> Self {
        let num_prg_banks_16k = (prg_rom.len() / 16384).max(1);
        let num_chr_banks_4k = (chr_rom.len() / 4096).max(1);

        // On reset, last 2 PRG pages are loaded into $8000-$FFFF.
        // Since the first bank is switchable, initialize it to the second-to-last.
        let initial_prg_bank = num_prg_banks_16k.saturating_sub(2);

        Self {
            prg_rom,
            chr_rom,
            prg_ram: alloc::vec![0u8; 8192],
            prg_bank: initial_prg_bank,
            chr_banks: [0; 4],
            // Spec: "latch selectors contain $FE on reset"
            latch_0: 0xFE,
            latch_1: 0xFE,
            mirroring: initial_mirroring,
            num_prg_banks_16k,
            num_chr_banks_4k,
        }
    }
}

impl Mapper for MMC4 {
    fn read_ex(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize],
            _ => 0,
        }
    }

    fn write_ex(&mut self, addr: u16, data: u8) {
        match addr {
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize] = data,
            _ => {}
        }
    }

    fn read_prg(&mut self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xBFFF => {
                let bank = self.prg_bank % self.num_prg_banks_16k;
                let offset = bank * 16384 + (addr as usize - 0x8000);
                self.prg_rom.get(offset).copied().unwrap_or(0)
            }
            0xC000..=0xFFFF => {
                // Fixed to the last 16K bank
                let bank = self.num_prg_banks_16k.saturating_sub(1);
                let offset = bank * 16384 + (addr as usize - 0xC000);
                self.prg_rom.get(offset).copied().unwrap_or(0)
            }
            _ => 0,
        }
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        match addr {
            0xA000..=0xAFFF => self.prg_bank = (data & 0x0F) as usize,
            0xB000..=0xBFFF => self.chr_banks[0] = (data & 0x1F) as usize,
            0xC000..=0xCFFF => self.chr_banks[1] = (data & 0x1F) as usize,
            0xD000..=0xDFFF => self.chr_banks[2] = (data & 0x1F) as usize,
            0xE000..=0xEFFF => self.chr_banks[3] = (data & 0x1F) as usize,
            0xF000..=0xFFFF => {
                self.mirroring = if (data & 0x01) == 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                };
            }
            _ => (),
        }
    }

    /// Latch update happens AFTER the tile fetch (spec requirement).
    /// We read using the current latch first, then update based on the tile address.
    /// MMC4 uses symmetrical ranges for both pattern tables (unlike MMC2).
    fn read_chr(&mut self, addr: u16) -> u8 {
        // Step 1: Read using the CURRENT latch value
        let bank = match addr {
            0x0000..=0x0FFF => {
                if self.latch_0 == 0xFD { self.chr_banks[0] } else { self.chr_banks[1] }
            }
            0x1000..=0x1FFF => {
                if self.latch_1 == 0xFD { self.chr_banks[2] } else { self.chr_banks[3] }
            }
            _ => return 0,
        };

        let offset = (bank % self.num_chr_banks_4k) * 4096 + (addr as usize & 0x0FFF);
        let value = self.chr_rom.get(offset).copied().unwrap_or(0);

        // Step 2: AFTER reading, update latch (per Disch: full tile fetch = $0FDx/$0FEx/$1FDx/$1FEx)
        // CHR behavior is identical to MMC2.
        match addr {
            0x0FD0..=0x0FDF => self.latch_0 = 0xFD,
            0x0FE0..=0x0FEF => self.latch_0 = 0xFE,
            0x1FD0..=0x1FDF => self.latch_1 = 0xFD,
            0x1FE0..=0x1FEF => self.latch_1 = 0xFE,
            _ => (),
        }

        value
    }

    fn write_chr(&mut self, _addr: u16, _data: u8) {}

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
