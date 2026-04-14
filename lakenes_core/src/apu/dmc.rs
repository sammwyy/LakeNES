pub const DMC_PERIOD_TABLE: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

pub struct DMC {
    pub enabled: bool,
    pub irq_enabled: bool,
    pub loop_flag: bool,
    pub rate_index: u8,
    pub timer: u16,
    pub output_level: u8,
    pub sample_address: u16,
    pub sample_length: u16,
    pub current_address: u16,
    pub bytes_remaining: u16,

    // Internal state
    pub sample_buffer: Option<u8>,
    pub shift_register: u8,
    pub bits_remaining: u8,
    pub silent: bool,
    pub irq_flag: bool,
}

impl DMC {
    pub fn new() -> Self {
        Self {
            enabled: false,
            irq_enabled: false,
            loop_flag: false,
            rate_index: 0,
            timer: 0,
            output_level: 0,
            sample_address: 0xC000,
            sample_length: 1,
            current_address: 0,
            bytes_remaining: 0,
            sample_buffer: None,
            shift_register: 0,
            bits_remaining: 8,
            silent: true,
            irq_flag: false,
        }
    }

    pub fn write_rate(&mut self, val: u8) {
        self.irq_enabled = (val & 0x80) != 0;
        self.loop_flag = (val & 0x40) != 0;
        self.rate_index = val & 0x0F;
        if !self.irq_enabled {
            self.irq_flag = false;
        }
    }

    pub fn write_level(&mut self, val: u8) {
        self.output_level = val & 0x7F;
    }

    pub fn step_timer(&mut self) {
        if self.timer == 0 {
            self.timer = DMC_PERIOD_TABLE[self.rate_index as usize];
            self.clock_output();
        } else {
            self.timer -= 1;
        }
    }

    fn clock_output(&mut self) {
        if self.bits_remaining == 0 {
            self.bits_remaining = 8;
            if let Some(val) = self.sample_buffer.take() {
                self.shift_register = val;
                self.silent = false;
            } else {
                self.silent = true;
            }
        } else {
            self.bits_remaining -= 1;
        }

        if !self.silent {
            if (self.shift_register & 0x01) != 0 {
                if self.output_level <= 125 {
                    self.output_level += 2;
                }
            } else {
                if self.output_level >= 2 {
                    self.output_level -= 2;
                }
            }
        }
        self.shift_register >>= 1;
    }

    pub fn step_reader<F>(&mut self, read_mem: &mut F)
    where
        F: FnMut(u16) -> u8,
    {
        if self.sample_buffer.is_none() && self.bytes_remaining > 0 {
            let val = read_mem(self.current_address);
            self.sample_buffer = Some(val);

            if self.current_address == 0xFFFF {
                self.current_address = 0x8000;
            } else {
                self.current_address += 1;
            }

            self.bytes_remaining -= 1;
            if self.bytes_remaining == 0 {
                if self.loop_flag {
                    self.current_address = self.sample_address;
                    self.bytes_remaining = self.sample_length;
                } else if self.irq_enabled {
                    self.irq_flag = true;
                }
            }
        }
    }
}
