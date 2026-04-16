pub struct FrameCounter {
    cycle: u32,
    mode_5step: bool,
    irq_inhibit: bool,
    pub irq_flag: bool,
    pending_val: u8,
    pending_delay: i8,
}

impl FrameCounter {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            mode_5step: false,
            irq_inhibit: false,
            irq_flag: false,
            pending_val: 0,
            pending_delay: -1,
        }
    }

    pub fn schedule_reset(&mut self, val: u8, delay: u8) {
        self.pending_val = val;
        self.pending_delay = delay as i8;
        // Writing to $4017 always clears the frame IRQ flag immediately.
        self.irq_flag = false;
    }

    /// Returns (quarter_frame, half_frame, irq)
    #[inline(always)]
    pub fn tick(&mut self) -> (bool, bool, bool) {
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
                7_458 => qf = true,
                14_914 => {
                    qf = true;
                    hf = true;
                }
                22_372 => qf = true,
                29_828 | 29_829 | 29_830 => {
                    if !self.irq_inhibit {
                        self.irq_flag = true;
                    }
                    if self.cycle == 29_830 {
                        qf = true;
                        hf = true;
                        self.cycle = 0;
                    }
                }
                _ => {}
            }
        } else {
            // 5-step mode (Mode 1)
            match self.cycle {
                7_458 => qf = true,
                14_914 => {
                    qf = true;
                    hf = true;
                }
                22_372 => qf = true,
                37_282 => {
                    qf = true;
                    hf = true;
                    self.cycle = 0;
                }
                _ => {}
            }
        }

        (qf, hf, self.irq_flag)
    }
}
