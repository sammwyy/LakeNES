use alloc::vec::Vec;

pub struct FDSDrive {
    pub sides: Vec<Vec<u8>>,
    pub side_masks: Vec<Vec<bool>>,
    pub current_side_idx: Option<usize>,
    pub head_position: usize,
    pub motor_on: bool,
    pub ready_flag: bool,
    pub end_of_head: bool,
    pub transfer_reset: bool,
    pub mode_read: bool,
    pub data_register: u8,
    pub byte_transfer_flag: bool,
    pub transfer_timer: f64,
    pub in_gap: bool,

    // Status
    pub disk_irq_enabled: bool,
    pub disk_irq_pending: bool,
}

impl FDSDrive {
    pub fn new(fds_data: Vec<u8>) -> Self {
        let mut sides = Vec::new();
        let mut side_masks = Vec::new();

        if fds_data.len() > 16 && &fds_data[0..4] == b"FDS\x1A" {
            let num_sides = fds_data[4] as usize;
            let mut offset = 16;
            for _ in 0..num_sides {
                if offset + 65500 <= fds_data.len() {
                    let side_data = &fds_data[offset..offset + 65500];
                    let (pd, mask) = Self::compile_side(side_data);
                    sides.push(pd);
                    side_masks.push(mask);
                    offset += 65500;
                }
            }
        } else if fds_data.len() >= 65500 {
            // Raw side format or multiple raw sides
            for side in fds_data.chunks_exact(65500) {
                let (pd, mask) = Self::compile_side(side);
                sides.push(pd);
                side_masks.push(mask);
            }
        }

        for (i, side) in sides.iter().enumerate() {
            log::info!("Side {} compiled size: {} bytes", i, side.len());
            log::info!(
                "Side {} first 20 bytes: {:02X?}",
                i,
                &side[..20.min(side.len())]
            );
            if let Some(pos) = side.iter().position(|&b| b == 0x80) {
                log::info!("Side {} first 0x80 at position {}", i, pos);
                log::info!(
                    "Bytes after 0x80: {:02X?}",
                    &side[pos..((pos + 10).min(side.len()))]
                );
            }
        }

        Self {
            sides,
            side_masks,
            current_side_idx: Some(0),
            head_position: 0,
            motor_on: false,
            ready_flag: false,
            end_of_head: false,
            transfer_reset: true,
            mode_read: true,
            data_register: 0,
            byte_transfer_flag: false,
            transfer_timer: 0.0,
            in_gap: true,
            disk_irq_enabled: false,
            disk_irq_pending: false,
        }
    }

    fn compile_side(data: &[u8]) -> (Vec<u8>, Vec<bool>) {
        let mut pd = Vec::new();
        let mut mask = Vec::new();

        // Leading gap
        pd.extend(alloc::vec![0; 3499]);
        mask.extend(alloc::vec![false; 3499]);
        pd.push(0x80); // Lead-in gap terminator
        mask.push(false);

        let mut offset = 0;
        let mut next_block4_size: usize = 0;

        while offset < data.len() {
            let block_type = data[offset];
            if block_type == 0 {
                break;
            }

            let block_size = match block_type {
                1 => 56,
                2 => 2,
                3 => {
                    log::info!(
                        "Block 3 at offset {}: {:02X?}",
                        offset,
                        &data[offset..(offset + 17).min(data.len())]
                    );
                    if offset + 15 <= data.len() {
                        let size_lo = data[offset + 13] as usize;
                        let size_hi = data[offset + 14] as usize;
                        next_block4_size = size_lo | (size_hi << 8);
                    }
                    16
                }
                4 => next_block4_size + 1,
                _ => break,
            };

            if offset + block_size <= data.len() {
                // Block data
                pd.extend_from_slice(&data[offset..offset + block_size]);
                mask.extend(alloc::vec![true; block_size]);

                pd.push(0); // CRC dummy
                mask.push(true);
                pd.push(0); // CRC dummy
                mask.push(true);

                pd.push(0x80); // Block end terminator (starts gap)
                mask.push(false);

                // Inter-block gap
                pd.extend(alloc::vec![0; 121]);
                mask.extend(alloc::vec![false; 121]);

                pd.push(0x80); // Next block lead-in terminator (ends gap)
                mask.push(false);

                offset += block_size;
            } else {
                break;
            }
        }

        (pd, mask)
    }

    pub fn step(&mut self, cycles: u64) {
        if self.motor_on {
            if self.transfer_reset {
                self.head_position = 0;
                self.in_gap = true;
                self.ready_flag = false;
            } else {
                if self.byte_transfer_flag {
                    return;
                }

                self.transfer_timer += cycles as f64;
                while self.transfer_timer >= 149.0 {
                    self.transfer_timer -= 149.0;

                    if let Some(side_idx) = self.current_side_idx {
                        if let (Some(side), Some(mask)) =
                            (self.sides.get_mut(side_idx), self.side_masks.get(side_idx))
                        {
                            if self.head_position < side.len() {
                                let byte = side[self.head_position];
                                let is_data = mask[self.head_position];
                                self.head_position += 1;

                                if !is_data {
                                    if byte == 0x80 {
                                        self.in_gap = false;
                                    } else {
                                        self.in_gap = true;
                                    }
                                } else {
                                    self.in_gap = false;
                                    if self.mode_read {
                                        self.data_register = byte;
                                    } else {
                                        side[self.head_position - 1] = self.data_register;
                                    }
                                    self.byte_transfer_flag = true;
                                    if self.disk_irq_enabled {
                                        self.disk_irq_pending = true;
                                    }
                                    break;
                                }
                            } else {
                                if !self.end_of_head {
                                    log::warn!("END OF HEAD at position {}", self.head_position);
                                }
                                self.end_of_head = true;
                                self.ready_flag = false;
                                break;
                            }
                        }
                    }
                }
                self.ready_flag = true;
            }
        } else {
            self.ready_flag = false;
            if self.transfer_reset {
                self.head_position = 0;
                self.in_gap = true;
            }
        }
    }
}
