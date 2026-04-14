use crate::mappers::Mapper;
use crate::mappers::mapper0::Mapper0;
use crate::mappers::mapper1::Mapper1;
use crate::mappers::mapper2::Mapper2;
use crate::mappers::mapper4::Mapper4;
use alloc::boxed::Box;

const NES_HEADER_SIZE: usize = 16;
const PRG_ROM_BANK_SIZE: usize = 16384;
const CHR_ROM_BANK_SIZE: usize = 8192;

#[derive(Debug)]
pub enum ROMError {
    InvalidHeader,
    IncompleteData,
}

pub struct ROM {
    pub mapper: Box<dyn Mapper>,
    pub mirroring: bool,
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
            crate::mappers::Mirroring::Vertical
        } else {
            crate::mappers::Mirroring::Horizontal
        };
        let mirroring = (header[6] & 0x01) != 0;

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

        let chr_size = if chr_banks == 0 {
            CHR_ROM_BANK_SIZE
        } else {
            chr_banks * CHR_ROM_BANK_SIZE
        };

        let chr_rom = if chr_banks > 0 {
            if bytes.len() < offset + chr_size {
                return Err(ROMError::IncompleteData);
            }
            let data = bytes[offset..offset + chr_size].to_vec();
            let _ = offset; // Final update not read
            data
        } else {
            alloc::vec![0u8; CHR_ROM_BANK_SIZE]
        };

        log::debug!("PRG ROM: {} banks ({} bytes)", prg_banks, prg_rom.len());
        log::debug!("CHR ROM: {} banks ({} bytes)", chr_banks, chr_rom.len());
        log::info!("Mapper ID: {}", mapper_id);

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
            4 => Box::new(Mapper4::new(prg_rom, chr_rom)),
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

        Ok(Self { mapper, mirroring })
    }
}
