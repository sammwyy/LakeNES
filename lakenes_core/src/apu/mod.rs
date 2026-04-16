pub mod dmc;
pub mod noise;
pub mod pulse;
pub mod triangle;
pub mod units;

pub use dmc::DMC;
pub use noise::Noise;
pub use pulse::Pulse;
pub use triangle::Triangle;
pub use units::{AudioFilter, Envelope, LengthCounter, LinearCounter, Sweep};

use crate::bus::BusDevice;

pub(crate) const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

// Precise APU frame counter
struct FrameCounter {
    cycle: u32,
    mode_5step: bool,
    irq_inhibit: bool,
    irq_flag: bool,
    pending_val: u8,
    pending_delay: i8,
}

impl FrameCounter {
    fn new() -> Self {
        Self {
            cycle: 0,
            mode_5step: false,
            irq_inhibit: false,
            irq_flag: false,
            pending_val: 0,
            pending_delay: -1,
        }
    }

    fn schedule_reset(&mut self, val: u8, delay: u8) {
        self.pending_val = val;
        self.pending_delay = delay as i8;
        // Writing to $4017 always clears the frame IRQ flag immediately.
        self.irq_flag = false;
    }

    /// Returns (quarter_frame, half_frame, irq)
    #[inline(always)]
    fn tick(&mut self) -> (bool, bool, bool) {
        let mut qf = false;
        let mut hf = false;

        // Process pending $4017 write delay
        if self.pending_delay >= 0 {
            if self.pending_delay == 0 {
                let val = self.pending_val;
                self.mode_5step = (val & 0x80) != 0;
                self.irq_inhibit = (val & 0x40) != 0;
                if self.irq_inhibit {
                    self.irq_flag = false;
                }
                self.cycle = 0;

                // In 5-step mode, quarter and half frame signals are clocked immediately.
                if self.mode_5step {
                    qf = true;
                    hf = true;
                }
                self.pending_delay = -1;
            } else {
                self.pending_delay -= 1;
            }
        }

        self.cycle = self.cycle.wrapping_add(1);

        // Fast path for non-event cycles
        if self.cycle < 7457 {
            return (false, false, self.irq_flag);
        }

        if !self.mode_5step {
            // 4-step mode (Mode 0)
            match self.cycle {
                7_457 => qf = true,
                14_913 => {
                    qf = true;
                    hf = true;
                }
                22_371 => qf = true,
                29_828 | 29_829 | 29_830 => {
                    if !self.irq_inhibit {
                        self.irq_flag = true;
                    }
                    if self.cycle == 29_829 {
                        qf = true;
                        hf = true;
                    } else if self.cycle == 29_830 {
                        self.cycle = 0;
                    }
                }
                _ => {}
            }
        } else {
            // 5-step mode (Mode 1)
            match self.cycle {
                7_457 => qf = true,
                14_913 => {
                    qf = true;
                    hf = true;
                }
                22_371 => qf = true,
                37_281 => {
                    qf = true;
                    hf = true;
                }
                37_282 => {
                    self.cycle = 0;
                }
                _ => {}
            }
        }

        (qf, hf, self.irq_flag)
    }
}

pub struct APU {
    pub pulse1: Pulse,
    pub pulse2: Pulse,
    pub triangle: Triangle,
    pub noise: Noise,
    pub dmc: DMC,
    frame_counter: FrameCounter,
    waveform_master_cycles: u64,
    filters: [AudioFilter; 3],
    dmc_cpu_stall_cycles: u64,
    pub volume_master: f32,
    pub volume_pulse1: f32,
    pub volume_pulse2: f32,
    pub volume_triangle: f32,
    pub volume_noise: f32,
    pub volume_dmc: f32,
    sample_accumulator: f32,
    sample_count: u32,
    last_sample: f32,
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
            volume_master: 1.0,
            volume_pulse1: 1.0,
            volume_pulse2: 1.0,
            volume_triangle: 1.0,
            volume_noise: 1.0,
            volume_dmc: 1.0,
            sample_accumulator: 0.0,
            sample_count: 0,
            last_sample: 0.0,
        }
    }

    pub fn set_volumes(
        &mut self,
        master: f32,
        pulse1: f32,
        pulse2: f32,
        triangle: f32,
        noise: f32,
        dmc: f32,
    ) {
        self.volume_master = master / 100.0;
        self.volume_pulse1 = pulse1 / 100.0;
        self.volume_pulse2 = pulse2 / 100.0;
        self.volume_triangle = triangle / 100.0;
        self.volume_noise = noise / 100.0;
        self.volume_dmc = dmc / 100.0;
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
                // Status register ($4015 bits 0-4 are enables)
                self.pulse1.enabled = (val & 0x01) != 0;
                self.pulse2.enabled = (val & 0x02) != 0;
                self.triangle.enabled = (val & 0x04) != 0;
                self.noise.enabled = (val & 0x08) != 0;
                self.dmc.enabled = (val & 0x10) != 0;

                self.pulse1.length.enabled = self.pulse1.enabled;
                self.pulse2.length.enabled = self.pulse2.enabled;
                self.triangle.length.enabled = self.triangle.enabled;
                self.noise.length.enabled = self.noise.enabled;

                if !self.pulse1.enabled { self.pulse1.length.count = 0; }
                if !self.pulse2.enabled { self.pulse2.length.count = 0; }
                if !self.triangle.enabled { self.triangle.length.count = 0; }
                if !self.noise.enabled { self.noise.length.count = 0; }

                if !self.dmc.enabled {
                    self.dmc.bytes_remaining = 0;
                    self.dmc.sample_buffer = None;
                } else if self.dmc.bytes_remaining == 0 {
                    self.dmc.current_address = self.dmc.sample_address;
                    self.dmc.bytes_remaining = self.dmc.sample_length;
                    self.dmc.sample_buffer = None;
                    self.dmc.shift_register = 0;
                    self.dmc.bits_remaining = 8;
                    self.dmc.silent = true;
                    self.dmc.timer = dmc::DMC_PERIOD_TABLE[self.dmc.rate_index as usize];
                }
                self.dmc.irq_flag = false;
            }
            0x4017 => {
                let delay = if (self.waveform_master_cycles % 2) == 0 { 3 } else { 4 };
                self.frame_counter.schedule_reset(val, delay);
            }
            _ => {}
        }
    }

    pub fn read_status(&mut self) -> u8 {
        let mut res = 0;
        if self.pulse1.length.count > 0 { res |= 0x01; }
        if self.pulse2.length.count > 0 { res |= 0x02; }
        if self.triangle.length.count > 0 { res |= 0x04; }
        if self.noise.length.count > 0 { res |= 0x08; }
        if self.dmc.bytes_remaining > 0 { res |= 0x10; }
        if self.dmc.irq_flag { res |= 0x80; }
        if self.frame_counter.irq_flag { res |= 0x40; }
        self.frame_counter.irq_flag = false;
        self.dmc.irq_flag = false;
        res
    }

    pub fn irq_active(&self) -> bool {
        self.dmc.irq_flag || self.frame_counter.irq_flag
    }

    #[inline(always)]
    pub fn step<F>(&mut self, mut read_mem: F)
    where
        F: FnMut(u16) -> u8,
    {
        let fc = self.frame_counter.tick();
        self.waveform_master_cycles = self.waveform_master_cycles.wrapping_add(1);

        // Timers
        self.triangle.step_timer();
        if (self.waveform_master_cycles & 1) == 0 {
            self.pulse1.step_timer();
            self.pulse2.step_timer();
        }
        self.noise.step_timer();
        if self.dmc.enabled {
            if self.dmc.step_reader(&mut read_mem) {
                self.dmc_cpu_stall_cycles = self.dmc_cpu_stall_cycles.saturating_add(4);
            }
            self.dmc.step_timer();
        }

        // Downsampling accumulation
        self.sample_accumulator += self.get_raw_sample();
        self.sample_count += 1;

        // Frame Counter signals
        let (qf, hf, _irq) = fc;
        if qf {
            self.pulse1.envelope.tick();
            self.pulse2.envelope.tick();
            self.noise.envelope.tick();
            self.triangle.linear.tick();
        }
        if hf {
            self.pulse1.length.tick();
            self.pulse1.sweep.tick(&mut self.pulse1.timer_period);
            self.pulse2.length.tick();
            self.pulse2.sweep.tick(&mut self.pulse2.timer_period);
            self.triangle.length.tick();
            self.noise.length.tick();
        }
    }

    #[inline(always)]
    fn get_raw_sample(&self) -> f32 {
        let p1 = self.pulse1.output() as f32 * self.volume_pulse1;
        let p2 = self.pulse2.output() as f32 * self.volume_pulse2;
        let t = self.triangle.output() as f32 * self.volume_triangle;
        let n = self.noise.output() as f32 * self.volume_noise;
        let d = self.dmc.output_level as f32 * self.volume_dmc;

        let pulse_sum = p1 + p2;
        let pulse_out = if pulse_sum > 0.0 { 95.88 / (8128.0 / pulse_sum + 100.0) } else { 0.0 };
        let tnd_out = if t > 0.0 || n > 0.0 || d > 0.0 {
            159.79 / (1.0 / (t / 8227.0 + n / 12241.0 + d / 22638.0) + 100.0)
        } else {
            0.0
        };

        (pulse_out + tnd_out) * self.volume_master
    }

    #[inline(always)]
    pub fn output_sample(&mut self) -> f32 {
        if self.sample_count == 0 { return self.last_sample; }
        let mut out = self.sample_accumulator / self.sample_count as f32;
        self.sample_accumulator = 0.0;
        self.sample_count = 0;
        for filter in &mut self.filters { out = filter.tick(out); }
        self.last_sample = out;
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
        match addr { 0x4015 => self.read_status(), _ => 0 }
    }
    fn write(&mut self, addr: u16, value: u8) {
        self.write_register(addr, value);
    }
}
