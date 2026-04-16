use super::{Envelope, LengthCounter, Sweep};

pub const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

pub struct Pulse {
    pub enabled: bool,
    pub channel: u8,
    pub duty: u8,
    pub timer: u16,
    pub timer_period: u16,
    pub sequence_pos: u8,
    pub envelope: Envelope,
    pub length: LengthCounter,
    pub sweep: Sweep,
}

impl Pulse {
    pub fn new(channel: u8) -> Self {
        Self {
            enabled: false,
            channel,
            duty: 0,
            timer: 0,
            timer_period: 0,
            sequence_pos: 0,
            envelope: Envelope::new(),
            length: LengthCounter::new(),
            sweep: Sweep::new(channel),
        }
    }

    pub fn write_control(&mut self, val: u8) {
        self.duty = (val >> 6) & 0x03;
        self.envelope.write_control(val);
        self.length.halt = self.envelope.loop_flag;
    }

    pub fn write_sweep(&mut self, val: u8) {
        self.sweep.write_register(val);
    }

    pub fn write_timer_low(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | (val as u16);
    }

    pub fn write_length_timer(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | ((val as u16 & 0x07) << 8);
        self.length.load(val);
        self.timer = self.timer_period; // Period divider IS reset on $4003/$4007
        self.sequence_pos = 0;
        self.envelope.start = true;
    }

    #[inline(always)]
    pub fn step_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            self.sequence_pos = (self.sequence_pos.wrapping_sub(1)) & 7;
        } else {
            self.timer -= 1;
        }
    }

    #[inline(always)]
    pub fn output(&self) -> u8 {
        if self.length.count == 0 || self.sweep.is_muting(self.timer_period) {
            return 0;
        }
        if DUTY_TABLE[self.duty as usize][self.sequence_pos as usize] == 0 {
            return 0;
        }
        self.envelope.output()
    }
}
