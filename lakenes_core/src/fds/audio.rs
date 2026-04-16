pub struct FDSAudio {
    pub wave_ram: [u8; 64],
}

impl FDSAudio {
    pub fn new() -> Self {
        Self { wave_ram: [0; 64] }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x4040..=0x407F => self.wave_ram[(addr - 0x4040) as usize],
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4040..=0x407F => self.wave_ram[(addr - 0x4040) as usize] = data,
            _ => {}
        }
    }

    pub fn step(&mut self, _cycles: u64) {
        // Synthesis logic goes here
    }

    pub fn output(&self) -> f32 {
        0.0
    }
}
