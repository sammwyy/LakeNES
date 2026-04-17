use super::{Mapper, Mirroring};
use alloc::vec::Vec;

/// MMC2 (iNES Mapper 9) - Nintendo PxROM boards
/// Used by Mike Tyson's Punch-Out!! and Punch-Out!!
///
/// PRG: 8KB switchable at $8000-$9FFF + three fixed 8KB banks ($A000-$FFFF)
/// CHR: Two pairs of 4KB banks, auto-switched by latch on PPU tile $FD/$FE reads
///
/// Key latch detail: latch_0 only triggers on single addresses ($0FD8 / $0FE8),
/// while latch_1 triggers on full ranges ($1FD8-$1FDF / $1FE8-$1FEF).
/// The latch update happens AFTER the tile fetch (implemented in read_chr).
pub struct MMC2 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,

    prg_bank: usize,

    /// [0] = $B000 reg: CHR bank for $0000-$0FFF when latch_0 = $FD
    /// [1] = $C000 reg: CHR bank for $0000-$0FFF when latch_0 = $FE
    /// [2] = $D000 reg: CHR bank for $1000-$1FFF when latch_1 = $FD
    /// [3] = $E000 reg: CHR bank for $1000-$1FFF when latch_1 = $FE
    chr_banks: [usize; 4],

    latch_0: u8,
    latch_1: u8,

    mirroring: Mirroring,

    num_prg_banks_8k: usize,
    num_chr_banks_4k: usize,
}

impl MMC2 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, initial_mirroring: Mirroring) -> Self {
        let num_prg_banks_8k = (prg_rom.len() / 8192).max(1);
        let num_chr_banks_4k = (chr_rom.len() / 4096).max(1);

        Self {
            prg_rom,
            chr_rom,
            prg_bank: 0,
            chr_banks: [0; 4],
            latch_0: 0xFD,
            latch_1: 0xFD,
            mirroring: initial_mirroring,
            num_prg_banks_8k,
            num_chr_banks_4k,
        }
    }
}

impl Mapper for MMC2 {
    fn read_prg(&mut self, addr: u16) -> Option<u8> {
        let bank = match addr {
            0x8000..=0x9FFF => self.prg_bank,
            0xA000..=0xBFFF => self.num_prg_banks_8k.saturating_sub(3),
            0xC000..=0xDFFF => self.num_prg_banks_8k.saturating_sub(2),
            0xE000..=0xFFFF => self.num_prg_banks_8k.saturating_sub(1),
            _ => return None,
        };

        let offset = (bank % self.num_prg_banks_8k) * 8192 + (addr as usize & 0x1FFF);
        self.prg_rom.get(offset).copied()
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        let val = (data & 0x1F) as usize;
        match addr {
            0xA000..=0xAFFF => self.prg_bank = val,
            0xB000..=0xBFFF => self.chr_banks[0] = val,
            0xC000..=0xCFFF => self.chr_banks[1] = val,
            0xD000..=0xDFFF => self.chr_banks[2] = val,
            0xE000..=0xEFFF => self.chr_banks[3] = val,
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
