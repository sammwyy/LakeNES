use super::LENGTH_TABLE;

// First-order IIR audio filter
pub struct AudioFilter {
    b0: f32,
    b1: f32,
    a1: f32,
    prev_x: f32,
    prev_y: f32,
}

impl AudioFilter {
    const PI: f32 = core::f32::consts::PI;

    #[inline(always)]
    pub fn high_pass(sample_rate: f32, cutoff: f32) -> Self {
        let c = sample_rate / Self::PI / cutoff;
        let a0i = 1.0 / (1.0 + c);
        Self {
            b0: c * a0i,
            b1: -(c * a0i),
            a1: (1.0 - c) * a0i,
            prev_x: 0.0,
            prev_y: 0.0,
        }
    }

    #[inline(always)]
    pub fn low_pass(sample_rate: f32, cutoff: f32) -> Self {
        let c = sample_rate / Self::PI / cutoff;
        let a0i = 1.0 / (1.0 + c);
        Self {
            b0: a0i,
            b1: a0i,
            a1: (1.0 - c) * a0i,
            prev_x: 0.0,
            prev_y: 0.0,
        }
    }

    #[inline(always)]
    pub fn tick(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.prev_x - self.a1 * self.prev_y;
        self.prev_y = y;
        self.prev_x = x;
        y
    }
}

pub struct Envelope {
    pub start: bool,
    pub loop_flag: bool,
    pub constant_volume: bool,
    pub volume_setting: u8,
    pub decay_count: u8,
    pub divider: u8,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            start: false,
            loop_flag: false,
            constant_volume: false,
            volume_setting: 0,
            decay_count: 0,
            divider: 0,
        }
    }

    pub fn write_control(&mut self, val: u8) {
        self.loop_flag = (val & 0x20) != 0;
        self.constant_volume = (val & 0x10) != 0;
        self.volume_setting = val & 0x0F;
    }

    pub fn tick(&mut self) {
        if self.start {
            self.start = false;
            self.decay_count = 15;
            self.divider = self.volume_setting;
        } else if self.divider == 0 {
            self.divider = self.volume_setting;
            if self.decay_count > 0 {
                self.decay_count -= 1;
            } else if self.loop_flag {
                self.decay_count = 15;
            }
        } else {
            self.divider -= 1;
        }
    }

    pub fn output(&self) -> u8 {
        if self.constant_volume {
            self.volume_setting
        } else {
            self.decay_count
        }
    }
}

pub struct LengthCounter {
    pub enabled: bool,
    pub count: u8,
    pub halt: bool,
}

impl LengthCounter {
    pub fn new() -> Self {
        Self {
            enabled: false,
            count: 0,
            halt: false,
        }
    }

    pub fn tick(&mut self) {
        if !self.halt && self.count > 0 {
            self.count -= 1;
        }
    }

    pub fn load(&mut self, val: u8) {
        if self.enabled {
            self.count = LENGTH_TABLE[(val as usize) >> 3];
        }
    }
}

pub struct Sweep {
    pub enabled: bool,
    pub period: u8,
    pub negate: bool,
    pub shift: u8,
    pub reload: bool,
    pub divider: u8,
    pub channel: u8, // 1 for Pulse 1, 2 for Pulse 2
}

impl Sweep {
    pub fn new(channel: u8) -> Self {
        Self {
            enabled: false,
            period: 0,
            negate: false,
            shift: 0,
            reload: false,
            divider: 0,
            channel,
        }
    }

    pub fn write_register(&mut self, val: u8) {
        self.enabled = (val & 0x80) != 0;
        self.period = (val >> 4) & 0x07;
        self.negate = (val & 0x08) != 0;
        self.shift = val & 0x07;
        self.reload = true;
    }

    pub fn tick(&mut self, timer_period: &mut u16) {
        if self.reload {
            if self.divider == 0 && self.enabled && self.shift > 0 && !self.is_muting(*timer_period) {
                self.adjust_period(timer_period);
            }
            self.divider = self.period;
            self.reload = false;
        } else if self.divider == 0 {
            if self.enabled && self.shift > 0 && !self.is_muting(*timer_period) {
                self.adjust_period(timer_period);
            }
            self.divider = self.period;
        } else {
            self.divider -= 1;
        }
    }

    fn adjust_period(&self, timer_period: &mut u16) {
        *timer_period = self.target_period(*timer_period) & 0x07FF;
    }

    #[inline(always)]
    fn target_period(&self, timer_period: u16) -> u16 {
        let change = timer_period >> self.shift;
        if self.negate {
            let extra = if self.channel == 1 { 1 } else { 0 };
            timer_period.wrapping_sub(change).wrapping_sub(extra)
        } else {
            timer_period.wrapping_add(change)
        }
    }

    pub fn is_muting(&self, timer_period: u16) -> bool {
        timer_period < 8 || (self.shift > 0 && self.target_period(timer_period) > 0x07FF)
    }
}

pub struct LinearCounter {
    pub reload_flag: bool,
    pub reload_value: u8,
    pub control_flag: bool,
    pub count: u8,
}

impl LinearCounter {
    pub fn new() -> Self {
        Self {
            reload_flag: false,
            reload_value: 0,
            control_flag: false,
            count: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.reload_flag {
            self.count = self.reload_value;
        } else if self.count > 0 {
            self.count -= 1;
        }

        if !self.control_flag {
            self.reload_flag = false;
        }
    }
}
