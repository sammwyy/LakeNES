use alloc::boxed::Box;

pub mod mapper0;
pub mod mapper1;
pub mod mapper2;
pub mod mapper3;
pub mod mapper4;
pub mod mapper7;

use mapper0::Mapper0;
use mapper1::Mapper1;
use mapper2::Mapper2;
use mapper3::Mapper3;
use mapper4::Mapper4;
use mapper7::Mapper7;

const NES_HEADER_SIZE: usize = 16;
const PRG_ROM_BANK_SIZE: usize = 16384;
const CHR_ROM_BANK_SIZE: usize = 8192;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    OneScreenLow,
    OneScreenHigh,
}

#[derive(Debug)]
pub enum ROMError {
    InvalidHeader,
    IncompleteData,
}

pub trait Mapper {
    fn read_prg(&mut self, addr: u16) -> u8;
    fn write_prg(&mut self, addr: u16, data: u8);
    fn read_chr(&mut self, addr: u16) -> u8;
    fn write_chr(&mut self, addr: u16, data: u8);
    /// PPU-driven address bus (0x0000–0x3FFF). Used by MMC3 for A12 IRQ timing;
    /// nametable/attribute fetches must be included so A12 can go low between pattern fetches.
    fn ppu_bus_address(&mut self, _addr: u16) {}
    fn irq_flag(&self) -> bool {
        false
    }
    fn mirroring(&self) -> Mirroring {
        Mirroring::Vertical
    }
}

pub struct ROM {
    pub mapper: Box<dyn Mapper>,
    pub mapper_id: u8,
    pub prg_size: usize,
    pub chr_size: usize,
}

impl ROM {
    pub fn load_from_bytes(bytes: &[u8]) -> core::result::Result<Self, ROMError> {
        if bytes.len() < NES_HEADER_SIZE {
            return Err(ROMError::IncompleteData);
        }

        let header = &bytes[0..NES_HEADER_SIZE];

        if &header[0..4] != b"NES\x1A" {
            return Err(ROMError::InvalidHeader);
        }

        let prg_banks = header[4] as usize;
        let chr_banks = header[5] as usize;
        let mapper_id = (header[7] & 0xF0) | (header[6] >> 4);

        let mirroring_mode = if (header[6] & 0x01) != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        let mut offset = NES_HEADER_SIZE;

        if header[6] & 0x04 != 0 {
            offset += 512; // Trainer
        }

        let prg_size = prg_banks * PRG_ROM_BANK_SIZE;
        if bytes.len() < offset + prg_size {
            return Err(ROMError::IncompleteData);
        }
        let prg_rom = bytes[offset..offset + prg_size].to_vec();
        offset += prg_size;

        let chr_size_on_disk = if chr_banks == 0 {
            CHR_ROM_BANK_SIZE
        } else {
            chr_banks * CHR_ROM_BANK_SIZE
        };

        let chr_rom = if chr_banks > 0 {
            if bytes.len() < offset + chr_size_on_disk {
                return Err(ROMError::IncompleteData);
            }
            let data = bytes[offset..offset + chr_size_on_disk].to_vec();
            data
        } else {
            alloc::vec![0u8; CHR_ROM_BANK_SIZE]
        };

        log::info!("PRG ROM: {} banks ({} bytes)", prg_banks, prg_rom.len());
        log::info!("CHR ROM: {} banks ({} bytes)", chr_banks, chr_rom.len());
        log::info!("Mapper ID: {}", mapper_id);

        let prg_len = prg_rom.len();
        let chr_len = chr_rom.len();

        let mapper: Box<dyn Mapper> = match mapper_id {
            0 => Box::new(Mapper0::new(
                prg_rom,
                chr_rom,
                prg_banks,
                chr_banks,
                mirroring_mode,
            )),
            1 => Box::new(Mapper1::new(prg_rom, chr_rom)),
            2 => Box::new(Mapper2::new(prg_rom, chr_rom, prg_banks, mirroring_mode)),
            3 => Box::new(Mapper3::new(
                prg_rom,
                chr_rom,
                prg_banks,
                chr_banks == 0,
                mirroring_mode,
            )),
            4 => Box::new(Mapper4::new(prg_rom, chr_rom)),
            7 => Box::new(Mapper7::new(prg_rom, chr_rom)),
            _ => {
                log::warn!(
                    "Mapper {} not implemented, falling back to Mapper 0",
                    mapper_id
                );
                Box::new(Mapper0::new(
                    prg_rom,
                    chr_rom,
                    prg_banks,
                    chr_banks,
                    mirroring_mode,
                ))
            }
        };

        Ok(Self {
            mapper,
            mapper_id,
            prg_size: prg_len,
            chr_size: chr_len,
        })
    }
}
