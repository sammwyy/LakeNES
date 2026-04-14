pub struct Joypad {
    strobe: bool,
    button_index: u8,
    button_status: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            strobe: false,
            button_index: 0,
            button_status: 0,
        }
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = (data & 0x01) != 0;
        if self.strobe {
            self.button_index = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_index > 7 {
            return 1;
        }

        let res = (self.button_status & (1 << self.button_index)) != 0;

        if !self.strobe {
            self.button_index += 1;
        }
        res as u8
    }

    pub fn update(&mut self, data: u8) {
        self.button_status = data;
    }
}
