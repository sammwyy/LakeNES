use crate::bus::BusDevice;

pub struct RAM {
    data: [u8; 0x800],
}

impl RAM {
    pub fn new() -> Self {
        Self { data: [0; 0x800] }
    }
}

impl BusDevice for RAM {
    fn read(&mut self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.data[addr as usize] = value;
    }
}
