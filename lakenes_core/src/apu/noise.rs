use super::{Envelope, LengthCounter};

pub const NOISE_PERIOD_TABLE: [u16; 16] = [
    2, 4, 8, 16, 32, 48, 64, 80, 101, 127, 190, 254, 381, 508, 1017, 2034,
];

pub struct Noise {
    pub enabled: bool,
    pub mode: bool,
    pub period_index: u8,
    pub timer: u16,
    pub shift_register: u16,
    pub envelope: Envelope,
    pub length: LengthCounter,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            enabled: false,
            mode: false,
            period_index: 0,
            timer: 0,
            shift_register: 1,
            envelope: Envelope::new(),
            length: LengthCounter::new(),
        }
    }

    pub fn write_control(&mut self, val: u8) {
        self.envelope.write_control(val);
        self.length.halt = self.envelope.loop_flag;
    }

    pub fn write_mode(&mut self, val: u8) {
        self.mode = (val & 0x80) != 0;
        self.period_index = val & 0x0F;
        self.timer = NOISE_PERIOD_TABLE[self.period_index as usize].saturating_sub(1);
    }

    pub fn write_length(&mut self, val: u8) {
        self.length.load(val);
        self.envelope.start = true;
    }

    #[inline(always)]
    pub fn step_timer(&mut self) {
        if self.timer == 0 {
            self.timer = NOISE_PERIOD_TABLE[self.period_index as usize].saturating_sub(1);
            let shift = if self.mode { 6 } else { 1 };
            let feedback = (self.shift_register & 0x01) ^ ((self.shift_register >> shift) & 0x01);
            self.shift_register >>= 1;
            self.shift_register |= feedback << 14;
        } else {
            self.timer -= 1;
        }
    }

    #[inline(always)]
    pub fn output(&self) -> u8 {
        if self.length.count == 0 || (self.shift_register & 0x01) == 1 {
            0
        } else {
            self.envelope.output()
        }
    }
}
