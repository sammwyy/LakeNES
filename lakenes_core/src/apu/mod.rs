pub mod dmc;
pub mod noise;
pub mod pulse;
pub mod triangle;

pub use dmc::DMC;
pub use noise::Noise;
pub use pulse::Pulse;
pub use triangle::Triangle;

use crate::bus::BusDevice;

pub(crate) const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

pub struct APU {
    pub pulse1: Pulse,
    pub pulse2: Pulse,
    pub triangle: Triangle,
    pub noise: Noise,
    pub dmc: DMC,
    pub frame_counter: u64,
    pub frame_cycle: u32,
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
            dmc: DMC::new(),
            frame_counter: 0,
            frame_cycle: 0,
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
            0x4010 => self.dmc.write_rate(val),
            0x4011 => self.dmc.write_level(val),
            0x4012 => self.dmc.sample_address = 0xC000 | ((val as u16) << 6),
            0x4013 => self.dmc.sample_length = ((val as u16) << 4) + 1,
            0x4015 => {
                self.pulse1.enabled = (val & 0x01) != 0;
                self.pulse2.enabled = (val & 0x02) != 0;
                self.triangle.enabled = (val & 0x04) != 0;
                self.noise.enabled = (val & 0x08) != 0;
                self.dmc.enabled = (val & 0x10) != 0;
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
                if !self.dmc.enabled {
                    self.dmc.bytes_remaining = 0;
                } else if self.dmc.bytes_remaining == 0 {
                    self.dmc.current_address = self.dmc.sample_address;
                    self.dmc.bytes_remaining = self.dmc.sample_length;
                }
                self.dmc.irq_flag = false;
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
        if self.dmc.bytes_remaining > 0 {
            res |= 0x10;
        }
        if self.dmc.irq_flag {
            res |= 0x80;
        }
        res
    }

    pub fn step<F>(&mut self, mut read_mem: F)
    where
        F: FnMut(u16) -> u8,
    {
        // Pulse channels are clocked every 2 CPU cycles
        if self.frame_counter % 2 == 0 {
            self.pulse1.step_timer();
            self.pulse2.step_timer();
        }
        self.triangle.step_timer();
        self.noise.step_timer();
        self.dmc.step_reader(&mut read_mem);
        self.dmc.step_timer();

        self.frame_counter = self.frame_counter.wrapping_add(1);
        self.frame_cycle += 1;

        let frame_step_cycles = 7457; // NTSC quarter frame cycles

        if self.frame_cycle >= frame_step_cycles {
            self.frame_cycle = 0;
            self.step_frame_counter();
        }
    }

    fn step_frame_counter(&mut self) {
        // Basic 4-step frame counter logic
        let step = (self.frame_counter / 7457) % 4;

        match step {
            0 | 2 => {
                self.step_quarter_frame();
            }
            1 => {
                self.step_quarter_frame();
                self.step_half_frame();
            }
            3 => {
                if !self.mode_5step {
                    self.step_quarter_frame();
                    self.step_half_frame();
                }
            }
            _ => {}
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
        let d = self.dmc.output_level;
        let tnd_out = if t > 0 || n > 0 || d > 0 {
            159.79 / (1.0 / (t as f32 / 8227.0 + n as f32 / 12241.0 + d as f32 / 22638.0) + 100.0)
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
