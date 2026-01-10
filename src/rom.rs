use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

const NES_HEADER_SIZE: usize = 16;
const PRG_ROM_BANK_SIZE: usize = 16384;
const CHR_ROM_BANK_SIZE: usize = 8192;

pub struct ROM {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    mapper: u8,
    mirroring: bool,
}

impl ROM {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut header = [0u8; NES_HEADER_SIZE];
        file.read_exact(&mut header)?;

        if &header[0..4] != b"NES\x1A" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid NES header",
            ));
        }

        let prg_banks = header[4] as usize;
        let chr_banks = header[5] as usize;
        let mapper = (header[7] & 0xF0) | (header[6] >> 4);
        let mirroring = (header[6] & 0x01) != 0;

        if header[6] & 0x04 != 0 {
            let mut trainer = [0u8; 512];
            file.read_exact(&mut trainer)?;
        }

        let mut prg_rom = vec![0u8; prg_banks * PRG_ROM_BANK_SIZE];
        file.read_exact(&mut prg_rom)?;

        let mut chr_rom = vec![0u8; chr_banks * CHR_ROM_BANK_SIZE];
        if chr_banks > 0 {
            file.read_exact(&mut chr_rom)?;
        }

        log::debug!("PRG ROM: {} banks ({} bytes)", prg_banks, prg_rom.len());
        log::debug!("CHR ROM: {} banks ({} bytes)", chr_banks, chr_rom.len());

        Ok(Self {
            prg_rom,
            chr_rom,
            mapper,
            mirroring,
        })
    }

    pub fn mapper(&self) -> u8 {
        self.mapper
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                let offset = (addr - 0x8000) as usize;
                let mask = self.prg_rom.len() - 1;
                self.prg_rom[offset & mask]
            }
            _ => 0,
        }
    }
}
