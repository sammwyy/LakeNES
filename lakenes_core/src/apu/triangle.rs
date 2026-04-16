use super::{LengthCounter, LinearCounter};

pub struct Triangle {
    pub enabled: bool,
    pub length: LengthCounter,
    pub linear: LinearCounter,
    pub timer: u16,
    pub timer_period: u16,
    pub sequence_pos: u8,
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            enabled: false,
            length: LengthCounter::new(),
            linear: LinearCounter::new(),
            timer: 0,
            timer_period: 0,
            sequence_pos: 0,
        }
    }

    pub fn write_control(&mut self, val: u8) {
        self.length.halt = (val & 0x80) != 0;
        self.linear.control_flag = (val & 0x80) != 0;
        self.linear.reload_value = val & 0x7F;
    }

    pub fn write_timer_low(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | (val as u16);
    }

    pub fn write_length_timer(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | ((val as u16 & 0x07) << 8);
        self.length.load(val);
        self.linear.reload_flag = true;
    }

    #[inline(always)]
    pub fn step_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            if self.length.count > 0 && self.linear.count > 0 {
                self.sequence_pos = (self.sequence_pos + 1) % 32;
            }
        } else {
            self.timer -= 1;
        }
    }

    #[inline(always)]
    pub fn output(&self) -> u8 {
        if self.length.count == 0 || self.linear.count == 0 {
            return 0;
        }
        // Period < 2: timer runs so fast the DAC averages to ~7.5 (nesdev).
        if self.timer_period < 2 {
            return 7;
        }
        let seq = self.sequence_pos;
        if seq < 16 {
            15 - seq
        } else {
            seq - 16
        }
    }
}
