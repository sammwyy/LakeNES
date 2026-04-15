use super::LENGTH_TABLE;

pub const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0], // 12.5%
    [0, 1, 1, 0, 0, 0, 0, 0], // 25%
    [0, 1, 1, 1, 1, 0, 0, 0], // 50%
    [1, 0, 0, 1, 1, 1, 1, 1], // 75% (inverse 25)
];

pub struct Pulse {
    pub enabled: bool,
    pub channel: u8,
    pub length_counter: u8,
    pub length_halt: bool,
    pub constant_volume: bool,
    pub volume: u8,
    pub duty: u8,
    pub timer: u16,
    pub timer_period: u16,
    pub sequence_pos: u8,
    pub envelope_start: bool,
    pub envelope_vol: u8,
    pub envelope_divider: u8,
    pub sweep_enabled: bool,
    pub sweep_period: u8,
    pub sweep_negate: bool,
    pub sweep_shift: u8,
    pub sweep_reload: bool,
    pub sweep_divider: u8,
}

impl Pulse {
    pub fn new(channel: u8) -> Self {
        Self {
            enabled: false,
            channel,
            length_counter: 0,
            length_halt: false,
            constant_volume: true,
            volume: 0,
            duty: 0,
            timer: 0,
            timer_period: 0,
            sequence_pos: 0,
            envelope_start: false,
            envelope_vol: 0,
            envelope_divider: 0,
            sweep_enabled: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_reload: false,
            sweep_divider: 0,
        }
    }

    pub fn write_control(&mut self, val: u8) {
        self.duty = (val >> 6) & 0x03;
        self.length_halt = (val & 0x20) != 0;
        self.constant_volume = (val & 0x10) != 0;
        self.volume = val & 0x0F;
    }

    pub fn write_sweep(&mut self, val: u8) {
        self.sweep_enabled = (val & 0x80) != 0;
        self.sweep_period = (val >> 4) & 0x07;
        self.sweep_negate = (val & 0x08) != 0;
        self.sweep_shift = val & 0x07;
        self.sweep_reload = true;
    }

    pub fn write_timer_low(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | (val as u16);
    }

    pub fn write_length_timer(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | ((val as u16 & 0x07) << 8);
        if self.enabled {
            self.length_counter = LENGTH_TABLE[((val >> 3) & 0x1F) as usize];
        }
        self.sequence_pos = 0;
        self.envelope_start = true;
    }

    pub fn step_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            self.sequence_pos = (self.sequence_pos.wrapping_add(1)) % 8;
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
        // Period 0 in the volume nibble is treated as 16 (same as sweep period 0 → 8).
        let period = if self.volume == 0 {
            16
        } else {
            self.volume
        };
        if self.envelope_start {
            self.envelope_start = false;
            self.envelope_vol = 15;
            self.envelope_divider = period;
        } else if self.envelope_divider == 0 {
            self.envelope_divider = period;
            if self.envelope_vol > 0 {
                self.envelope_vol -= 1;
            } else if self.length_halt {
                self.envelope_vol = 15;
            }
        } else {
            self.envelope_divider -= 1;
        }
    }

    pub fn step_sweep(&mut self) {
        let period = if self.sweep_period == 0 {
            8
        } else {
            self.sweep_period
        };

        if self.sweep_reload {
            self.sweep_reload = false;
            self.sweep_divider = period;
            return;
        }

        if self.sweep_divider == 0 {
            if self.sweep_enabled && self.sweep_shift > 0 {
                let change = self.timer_period >> self.sweep_shift;
                if self.sweep_negate {
                    if self.timer_period >= 8 {
                        if self.channel == 1 {
                            if self.timer_period > change {
                                let mut np = self.timer_period - change;
                                if np > 0 {
                                    np -= 1;
                                }
                                self.timer_period = np;
                            }
                        } else if self.timer_period > change {
                            self.timer_period -= change;
                        }
                    }
                } else if self.timer_period + change < 0x800 {
                    self.timer_period += change;
                }
            }
            self.sweep_divider = period;
        } else {
            self.sweep_divider -= 1;
        }
    }

    pub fn output(&self) -> u8 {
        if !self.enabled
            || self.length_counter == 0
            || self.timer_period < 8
            || self.timer_period > 0x7FF
        {
            return 0;
        }
        if DUTY_TABLE[self.duty as usize][self.sequence_pos as usize] == 0 {
            return 0;
        }
        if self.constant_volume {
            self.volume
        } else {
            self.envelope_vol
        }
    }
}
