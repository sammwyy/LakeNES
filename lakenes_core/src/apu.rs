use crate::bus::BusDevice;

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0], // 12.5%
    [0, 1, 1, 0, 0, 0, 0, 0], // 25%
    [0, 1, 1, 1, 1, 0, 0, 0], // 50%
    [1, 0, 0, 1, 1, 1, 1, 1], // 75% (inverse 25)
];

const NOISE_PERIOD_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
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
    fn new(channel: u8) -> Self {
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

    fn write_control(&mut self, val: u8) {
        self.duty = (val >> 6) & 0x03;
        self.length_halt = (val & 0x20) != 0;
        self.constant_volume = (val & 0x10) != 0;
        self.volume = val & 0x0F;
    }

    fn write_sweep(&mut self, val: u8) {
        self.sweep_enabled = (val & 0x80) != 0;
        self.sweep_period = (val >> 4) & 0x07;
        self.sweep_negate = (val & 0x08) != 0;
        self.sweep_shift = val & 0x07;
        self.sweep_reload = true;
    }

    fn write_timer_low(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | (val as u16);
    }

    fn write_length_timer(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | ((val as u16 & 0x07) << 8);
        if self.enabled {
            self.length_counter = LENGTH_TABLE[((val >> 3) & 0x1F) as usize];
        }
        self.sequence_pos = 0;
        self.envelope_start = true;
    }

    fn step_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            self.sequence_pos = (self.sequence_pos.wrapping_add(1)) % 8;
        } else {
            self.timer -= 1;
        }
    }

    fn step_length(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    fn step_envelope(&mut self) {
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

    fn step_sweep(&mut self) {
        if self.sweep_divider == 0 {
            if self.sweep_enabled && self.sweep_period > 0 && self.sweep_shift > 0 {
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
                        } else {
                            if self.timer_period > change {
                                self.timer_period -= change;
                            }
                        }
                    }
                } else {
                    if self.timer_period + change < 0x800 {
                        self.timer_period += change;
                    }
                }
            }
            self.sweep_divider = self.sweep_period;
        } else {
            self.sweep_divider -= 1;
        }
    }

    fn output(&self) -> u8 {
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
    fn new() -> Self {
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

    fn write_control(&mut self, val: u8) {
        self.length_halt = (val & 0x80) != 0;
        self.linear_counter_load = val & 0x7F;
    }

    fn write_timer_low(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | (val as u16);
    }

    fn write_length_timer(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | ((val as u16 & 0x07) << 8);
        if self.enabled {
            self.length_counter = LENGTH_TABLE[((val >> 3) & 0x1F) as usize];
        }
        self.linear_reload = true;
    }

    fn step_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            if self.length_counter > 0 && self.linear_counter > 0 {
                self.sequence_pos = (self.sequence_pos + 1) % 32;
            }
        } else {
            self.timer -= 1;
        }
    }

    fn step_length(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    fn step_linear(&mut self) {
        if self.linear_reload {
            self.linear_counter = self.linear_counter_load;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }
        if !self.length_halt {
            self.linear_reload = false;
        }
    }

    fn output(&self) -> u8 {
        if self.length_counter == 0 || self.linear_counter == 0 {
            return 0;
        }
        let seq = self.sequence_pos;
        if seq < 16 { 15 - seq } else { seq - 16 }
    }
}

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
    fn new() -> Self {
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

    fn write_control(&mut self, val: u8) {
        self.length_halt = (val & 0x20) != 0;
        self.constant_volume = (val & 0x10) != 0;
        self.volume = val & 0x0F;
    }

    fn write_mode(&mut self, val: u8) {
        self.mode = (val & 0x80) != 0;
        self.period_index = val & 0x0F;
    }

    fn write_length(&mut self, val: u8) {
        if self.enabled {
            self.length_counter = LENGTH_TABLE[((val >> 3) & 0x1F) as usize];
        }
        self.envelope_start = true;
    }

    fn step_timer(&mut self) {
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

    fn step_length(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    fn step_envelope(&mut self) {
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

    fn output(&self) -> u8 {
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

pub struct APU {
    pub pulse1: Pulse,
    pub pulse2: Pulse,
    pub triangle: Triangle,
    pub noise: Noise,
    pub frame_counter: u64,
    pub mode_5step: bool,
    pub irq_inhibit: bool,
    hpf_state: f32,
}

impl APU {
    pub fn new() -> Self {
        Self {
            pulse1: Pulse::new(1),
            pulse2: Pulse::new(2),
            triangle: Triangle::new(),
            noise: Noise::new(),
            frame_counter: 0,
            mode_5step: false,
            irq_inhibit: false,
            hpf_state: 0.0,
        }
    }

    pub fn write_register(&mut self, addr: u16, val: u8) {
        match addr {
            0x4000 => self.pulse1.write_control(val),
            0x4001 => self.pulse1.write_sweep(val),
            0x4002 => self.pulse1.write_timer_low(val),
            0x4003 => self.pulse1.write_length_timer(val),
            0x4004 => self.pulse2.write_control(val),
            0x4005 => self.pulse2.write_sweep(val),
            0x4006 => self.pulse2.write_timer_low(val),
            0x4007 => self.pulse2.write_length_timer(val),
            0x4008 => self.triangle.write_control(val),
            0x400A => self.triangle.write_timer_low(val),
            0x400B => self.triangle.write_length_timer(val),
            0x400C => self.noise.write_control(val),
            0x400E => self.noise.write_mode(val),
            0x400F => self.noise.write_length(val),
            0x4015 => {
                self.pulse1.enabled = (val & 0x01) != 0;
                self.pulse2.enabled = (val & 0x02) != 0;
                self.triangle.enabled = (val & 0x04) != 0;
                self.noise.enabled = (val & 0x08) != 0;
                if !self.pulse1.enabled {
                    self.pulse1.length_counter = 0;
                }
                if !self.pulse2.enabled {
                    self.pulse2.length_counter = 0;
                }
                if !self.triangle.enabled {
                    self.triangle.length_counter = 0;
                }
                if !self.noise.enabled {
                    self.noise.length_counter = 0;
                }
            }
            0x4017 => {
                self.mode_5step = (val & 0x80) != 0;
                self.irq_inhibit = (val & 0x40) != 0;
            }
            _ => {}
        }
    }

    pub fn read_status(&mut self) -> u8 {
        let mut res = 0;
        if self.pulse1.length_counter > 0 {
            res |= 0x01;
        }
        if self.pulse2.length_counter > 0 {
            res |= 0x02;
        }
        if self.triangle.length_counter > 0 {
            res |= 0x04;
        }
        if self.noise.length_counter > 0 {
            res |= 0x08;
        }
        res
    }

    pub fn step(&mut self) {
        if self.frame_counter % 2 == 0 {
            self.pulse1.step_timer();
            self.pulse2.step_timer();
        }
        self.triangle.step_timer();
        self.noise.step_timer();
        self.frame_counter = self.frame_counter.wrapping_add(1);
        if self.frame_counter % 7457 == 0 {
            self.step_quarter_frame();
            if self.frame_counter % 14914 == 0 {
                self.step_half_frame();
            }
        }
    }

    fn step_quarter_frame(&mut self) {
        self.pulse1.step_envelope();
        self.pulse2.step_envelope();
        self.noise.step_envelope();
        self.triangle.step_linear();
    }

    fn step_half_frame(&mut self) {
        self.pulse1.step_length();
        self.pulse1.step_sweep();
        self.pulse2.step_length();
        self.pulse2.step_sweep();
        self.triangle.step_length();
        self.noise.step_length();
    }

    pub fn output_sample(&mut self) -> f32 {
        let p1 = self.pulse1.output();
        let p2 = self.pulse2.output();
        let t = self.triangle.output();
        let n = self.noise.output();

        let pulse_out = if p1 + p2 > 0 {
            95.88 / (8128.0 / (p1 + p2) as f32 + 100.0)
        } else {
            0.0
        };
        let tnd_out = if t > 0 || n > 0 {
            159.79 / (1.0 / (t as f32 / 8227.0 + n as f32 / 12241.0) + 100.0)
        } else {
            0.0
        };

        let out = pulse_out + tnd_out;
        self.hpf_state += (out - self.hpf_state) * 0.01;
        out - self.hpf_state
    }
}

impl BusDevice for APU {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x4015 => self.read_status(),
            _ => 0,
        }
    }
    fn write(&mut self, addr: u16, value: u8) {
        self.write_register(addr, value);
    }
}
