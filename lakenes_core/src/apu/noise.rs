use super::LENGTH_TABLE;

pub const NOISE_PERIOD_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

pub struct Noise {
    pub enabled: bool,
    pub length_counter: u8,
    pub length_halt: bool,
    pub constant_volume: bool,
    pub volume: u8,
    pub mode: bool,
    pub period_index: u8,
    pub timer: u16,
    pub shift_register: u16,
    pub envelope_start: bool,
    pub envelope_vol: u8,
    pub envelope_divider: u8,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            enabled: false,
            length_counter: 0,
            length_halt: false,
            constant_volume: true,
            volume: 0,
            mode: false,
            period_index: 0,
            timer: 0,
            shift_register: 1,
            envelope_start: false,
            envelope_vol: 0,
            envelope_divider: 0,
        }
    }

    pub fn write_control(&mut self, val: u8) {
        self.length_halt = (val & 0x20) != 0;
        self.constant_volume = (val & 0x10) != 0;
        self.volume = val & 0x0F;
    }

    pub fn write_mode(&mut self, val: u8) {
        self.mode = (val & 0x80) != 0;
        self.period_index = val & 0x0F;
    }

    pub fn write_length(&mut self, val: u8) {
        if self.enabled {
            self.length_counter = LENGTH_TABLE[((val >> 3) & 0x1F) as usize];
        }
        self.envelope_start = true;
    }

    pub fn step_timer(&mut self) {
        if self.timer == 0 {
            self.timer = NOISE_PERIOD_TABLE[self.period_index as usize];
            let shift = if self.mode { 6 } else { 1 };
            let feedback = (self.shift_register & 0x01) ^ ((self.shift_register >> shift) & 0x01);
            self.shift_register >>= 1;
            self.shift_register |= feedback << 14;
        } else {
            self.timer -= 1;
        }
    }

    pub fn step_length(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn step_envelope(&mut self) {
        if self.envelope_start {
            self.envelope_start = false;
            self.envelope_vol = 15;
            self.envelope_divider = self.volume;
        } else {
            if self.envelope_divider == 0 {
                self.envelope_divider = self.volume;
                if self.envelope_vol > 0 {
                    self.envelope_vol -= 1;
                } else if self.length_halt {
                    self.envelope_vol = 15;
                }
            } else {
                self.envelope_divider -= 1;
            }
        }
    }

    pub fn output(&self) -> u8 {
        if !self.enabled || self.length_counter == 0 || (self.shift_register & 0x01) == 1 {
            0
        } else {
            if self.constant_volume {
                self.volume
            } else {
                self.envelope_vol
            }
        }
    }
}
