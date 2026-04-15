pub const DMC_PERIOD_TABLE: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

pub struct DMC {
    pub enabled: bool,
    pub irq_enabled: bool,
    pub loop_flag: bool,
    pub rate_index: u8,
    pub timer: u16,
    pub output_level: u8,
    pub sample_address: u16,
    pub sample_length: u16,
    pub current_address: u16,
    pub bytes_remaining: u16,

    // Internal state
    pub sample_buffer: Option<u8>,
    pub shift_register: u8,
    pub bits_remaining: u8,
    pub silent: bool,
    pub irq_flag: bool,
}

impl DMC {
    pub fn new() -> Self {
        Self {
            enabled: false,
            irq_enabled: false,
            loop_flag: false,
            rate_index: 0,
            // Timer starts at the first period value so the first clock fires correctly
            timer: 0,
            output_level: 0,
            sample_address: 0xC000,
            sample_length: 1,
            current_address: 0,
            bytes_remaining: 0,
            sample_buffer: None,
            shift_register: 0,
            bits_remaining: 8,
            silent: true,
            irq_flag: false,
        }
    }

    pub fn write_rate(&mut self, val: u8) {
        self.irq_enabled = (val & 0x80) != 0;
        self.loop_flag = (val & 0x40) != 0;
        self.rate_index = val & 0x0F;
        if !self.irq_enabled {
            self.irq_flag = false;
        }
    }

    pub fn write_level(&mut self, val: u8) {
        self.output_level = val & 0x7F;
    }

    /// Clocked every CPU cycle. The DMC timer counts down and fires an output
    /// bit when it reaches zero, then reloads from the period table.
    pub fn step_timer(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        } else {
            // Reload timer FIRST (period - 1 because this cycle is consumed)
            self.timer = DMC_PERIOD_TABLE[self.rate_index as usize] - 1;
            self.clock_output();
        }
    }

    /// Clock one output bit from the shift register.
    /// NES DMC hardware timing:
    ///   1. Apply current LSB to DAC (if not silent)
    ///   2. Shift register right by 1
    ///   3. Decrement bits_remaining
    ///   4. If bits_remaining reaches 0, reload from sample_buffer (or go silent)
    fn clock_output(&mut self) {
        // Step 1: Apply the current LSB to the output level
        if !self.silent {
            if (self.shift_register & 0x01) != 0 {
                if self.output_level <= 125 {
                    self.output_level += 2;
                }
            } else if self.output_level >= 2 {
                self.output_level -= 2;
            }
        }

        // Step 2: Shift the register right
        self.shift_register >>= 1;

        // Step 3: Decrement and check for reload
        self.bits_remaining -= 1;
        if self.bits_remaining == 0 {
            self.bits_remaining = 8;
            // Step 4: Reload from sample buffer
            if let Some(val) = self.sample_buffer.take() {
                self.shift_register = val;
                self.silent = false;
            } else {
                self.silent = true;
            }
        }
    }

    /// Should be called once per CPU cycle (after step_timer). Fetches the next
    /// sample byte from memory when the sample buffer is empty and bytes remain.
    pub fn step_reader<F>(&mut self, read_mem: &mut F) -> bool
    where
        F: FnMut(u16) -> u8,
    {
        let mut fetched = false;
        if self.sample_buffer.is_none() && self.bytes_remaining > 0 {
            // DMA stall would happen here in accurate emulators; we skip the stall
            let val = read_mem(self.current_address);
            self.sample_buffer = Some(val);
            fetched = true;

            // Advance address, wrapping from 0xFFFF back to 0x8000
            if self.current_address == 0xFFFF {
                self.current_address = 0x8000;
            } else {
                self.current_address += 1;
            }

            self.bytes_remaining -= 1;
            if self.bytes_remaining == 0 {
                if self.loop_flag {
                    // Restart the sample
                    self.current_address = self.sample_address;
                    self.bytes_remaining = self.sample_length;
                } else if self.irq_enabled {
                    self.irq_flag = true;
                }
            }
        }
        fetched
    }
}
