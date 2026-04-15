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

// ---------------------------------------------------------------------------
// First-order IIR audio filter
// ---------------------------------------------------------------------------
struct AudioFilter {
    b0: f32,
    b1: f32,
    a1: f32,
    prev_x: f32,
    prev_y: f32,
}

impl AudioFilter {
    const PI: f32 = core::f32::consts::PI;

    fn high_pass(sample_rate: f32, cutoff: f32) -> Self {
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

    fn low_pass(sample_rate: f32, cutoff: f32) -> Self {
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

    fn tick(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.prev_x - self.a1 * self.prev_y;
        self.prev_y = y;
        self.prev_x = x;
        y
    }
}

// ---------------------------------------------------------------------------
// Precise APU frame counter
//
// NES NTSC exact cycle counts (from nesdev wiki):
//  Mode 0 (4-step):
//    Quarter @  7,457
//    Half    @ 14,913
//    Quarter @ 22,371
//    IRQ     @ 29,828
//    Half+IRQ@ 29,829  (also clears frame counter)
//  Mode 1 (5-step):
//    Quarter @  7,457
//    Half    @ 14,913
//    Quarter @ 22,371
//    (nothing at 29,828-30k)
//    Half    @ 37,281
// ---------------------------------------------------------------------------
struct FrameCounter {
    cycle: u32,
    mode_5step: bool,
    irq_inhibit: bool,
    irq_flag: bool,
}

impl FrameCounter {
    fn new() -> Self {
        Self {
            cycle: 0,
            mode_5step: false,
            irq_inhibit: false,
            irq_flag: false,
        }
    }

    /// Returns (quarter_frame, half_frame, irq)
    fn tick(&mut self) -> (bool, bool, bool) {
        self.cycle = self.cycle.wrapping_add(1);
        let mut qf = false;
        let mut hf = false;
        let mut irq = false;

        if !self.mode_5step {
            // 4-step mode
            match self.cycle {
                7_457 => {
                    qf = true;
                }
                14_913 => {
                    qf = true;
                    hf = true;
                }
                22_371 => {
                    qf = true;
                }
                29_828 => {
                    if !self.irq_inhibit {
                        self.irq_flag = true;
                        irq = true;
                    }
                }
                29_829 => {
                    qf = true;
                    hf = true;
                    if !self.irq_inhibit {
                        self.irq_flag = true;
                        irq = true;
                    }
                    self.cycle = 0;
                }
                _ => {}
            }
        } else {
            // 5-step mode (no IRQ)
            match self.cycle {
                7_457 => {
                    qf = true;
                }
                14_913 => {
                    qf = true;
                    hf = true;
                }
                22_371 => {
                    qf = true;
                }
                37_281 => {
                    qf = true;
                    hf = true;
                    self.cycle = 0;
                }
                _ => {}
            }
        }

        (qf, hf, irq)
    }

    fn write_register(&mut self, val: u8) {
        self.mode_5step = (val & 0x80) != 0;
        self.irq_inhibit = (val & 0x40) != 0;
        if self.irq_inhibit {
            self.irq_flag = false;
        }
        // Reset the cycle counter so the timer starts fresh
        self.cycle = 0;
    }
}

// ---------------------------------------------------------------------------
// APU
// ---------------------------------------------------------------------------
pub struct APU {
    pub pulse1: Pulse,
    pub pulse2: Pulse,
    pub triangle: Triangle,
    pub noise: Noise,
    pub dmc: DMC,
    frame_counter: FrameCounter,
    /// Master CPU cycles since APU reset: pulse/noise timers use φ2-style half-rate stepping.
    /// Must not be tied to the frame counter (that counter resets each frame / on $4017).
    waveform_master_cycles: u64,
    filters: [AudioFilter; 3],
    dmc_cpu_stall_cycles: u64,
}

impl APU {
    fn make_filters(sample_rate: f32) -> [AudioFilter; 3] {
        [
            AudioFilter::high_pass(sample_rate, 90.0),
            AudioFilter::high_pass(sample_rate, 440.0),
            AudioFilter::low_pass(sample_rate, 14_000.0),
        ]
    }

    pub fn new() -> Self {
        Self {
            pulse1: Pulse::new(1),
            pulse2: Pulse::new(2),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: DMC::new(),
            frame_counter: FrameCounter::new(),
            waveform_master_cycles: 0,
            filters: Self::make_filters(44_100.0),
            dmc_cpu_stall_cycles: 0,
        }
    }

    pub fn set_output_sample_rate(&mut self, sample_rate: f32) {
        if sample_rate > 0.0 {
            self.filters = Self::make_filters(sample_rate);
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
                    self.dmc.sample_buffer = None;
                } else if self.dmc.bytes_remaining == 0 {
                    // Fresh start: reload reader state and phase the rate timer (nesdev DMC).
                    self.dmc.current_address = self.dmc.sample_address;
                    self.dmc.bytes_remaining = self.dmc.sample_length;
                    self.dmc.sample_buffer = None;
                    self.dmc.shift_register = 0;
                    self.dmc.bits_remaining = 8;
                    self.dmc.silent = true;
                    self.dmc.timer = dmc::DMC_PERIOD_TABLE[self.dmc.rate_index as usize];
                }
                // Writing $4015 always clears the DMC IRQ flag
                self.dmc.irq_flag = false;
            }
            0x4017 => {
                self.frame_counter.write_register(val);
                // In 5-step mode, immediately fire a half-frame clock
                if self.frame_counter.mode_5step {
                    self.step_quarter_frame();
                    self.step_half_frame();
                }
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
        if self.frame_counter.irq_flag {
            res |= 0x40;
        }
        // Reading $4015 clears frame IRQ and DMC IRQ (2A03 status register).
        self.frame_counter.irq_flag = false;
        self.dmc.irq_flag = false;
        res
    }

    pub fn irq_active(&self) -> bool {
        self.dmc.irq_flag || self.frame_counter.irq_flag
    }

    pub fn step<F>(&mut self, mut read_mem: F)
    where
        F: FnMut(u16) -> u8,
    {
        // Triangle: every CPU cycle. Pulse: every 2 CPU cycles (M2). Frame counter: every cycle.
        let fc = self.frame_counter.tick();

        self.waveform_master_cycles = self.waveform_master_cycles.wrapping_add(1);

        self.triangle.step_timer();

        let tick_pulse = (self.waveform_master_cycles & 1) == 1;
        if tick_pulse {
            self.pulse1.step_timer();
            self.pulse2.step_timer();
        }
        // Noise divider runs at CPU rate; period table values are in CPU cycles (not M2).
        self.noise.step_timer();

        // DMC: reader first if channel is enabled and has samples,
        // but timer always clocks to maintain internal phase.
        if self.dmc.enabled {
            if self.dmc.step_reader(&mut read_mem) {
                // DMC DMA read steals CPU bus cycles.
                // Fine-grain parity/phase is not modeled yet.
                self.dmc_cpu_stall_cycles = self.dmc_cpu_stall_cycles.saturating_add(4);
            }
            self.dmc.step_timer();
        }

        // Apply frame counter signals
        let (qf, hf, _irq) = fc;
        if qf {
            self.step_quarter_frame();
        }
        if hf {
            self.step_half_frame();
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
        let d = self.dmc.output_level;

        // NES APU mixer formula (from nesdev wiki)
        let pulse_out = if p1 + p2 > 0 {
            95.88 / (8128.0 / (p1 + p2) as f32 + 100.0)
        } else {
            0.0
        };

        let tnd_out = if t > 0 || n > 0 || d > 0 {
            159.79 / (1.0 / (t as f32 / 8227.0 + n as f32 / 12241.0 + d as f32 / 22638.0) + 100.0)
        } else {
            0.0
        };

        // Raw output in [0, ~1]
        let raw = pulse_out + tnd_out;

        // Apply three IIR audio filters (HP 90Hz, HP 440Hz, LP 14kHz)
        let mut out = raw;
        for f in self.filters.iter_mut() {
            out = f.tick(out);
        }
        out
    }

    pub fn take_dmc_cpu_stall_cycles(&mut self) -> u64 {
        let stall = self.dmc_cpu_stall_cycles;
        self.dmc_cpu_stall_cycles = 0;
        stall
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
