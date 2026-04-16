use super::LENGTH_TABLE;

pub struct Triangle {
    pub enabled: bool,
    pub length_counter: u8,
    pub length_halt: bool,
    pub linear_counter_load: u8,
    pub linear_counter: u8,
    pub linear_reload: bool,
    pub timer: u16,
    pub timer_period: u16,
    pub sequence_pos: u8,
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            enabled: false,
            length_counter: 0,
            length_halt: false,
            linear_counter_load: 0,
            linear_counter: 0,
            linear_reload: false,
            timer: 0,
            timer_period: 0,
            sequence_pos: 0,
        }
    }

    pub fn write_control(&mut self, val: u8) {
        self.length_halt = (val & 0x80) != 0;
        self.linear_counter_load = val & 0x7F;
    }

    pub fn write_timer_low(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | (val as u16);
    }

    pub fn write_length_timer(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | ((val as u16 & 0x07) << 8);
        if self.enabled {
            self.length_counter = LENGTH_TABLE[((val >> 3) & 0x1F) as usize];
        }
        self.linear_reload = true;
    }

    #[inline(always)]
    pub fn step_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            if self.length_counter > 0 && self.linear_counter > 0 {
                self.sequence_pos = (self.sequence_pos + 1) % 32;
            }
        } else {
            self.timer -= 1;
        }
    }

    #[inline(always)]
    pub fn step_length(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    #[inline(always)]
    pub fn step_linear(&mut self) {
        if self.linear_reload {
            self.linear_counter = self.linear_counter_load;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }
        if !self.length_halt {
            self.linear_reload = false;
        }
    }

    #[inline(always)]
    pub fn output(&self) -> u8 {
        if self.length_counter == 0 || self.linear_counter == 0 {
            return 0;
        }
        // Period < 2: timer runs so fast the DAC averages to ~7.5 (nesdev).
        if self.timer_period < 2 {
            return 7;
        }
        let seq = self.sequence_pos;
        if seq < 16 { 15 - seq } else { seq - 16 }
    }
}
