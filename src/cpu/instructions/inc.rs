use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let (addr, value) = match opcode {
        0xE6 => {
            cpu.cycles += 5;
            let addr = get_address_zeropage(cpu, bus);
            let val = bus.read(addr);
            (addr, val)
        }
        0xF6 => {
            cpu.cycles += 6;
            let addr = get_address_zeropage_x(cpu, bus);
            let val = bus.read(addr);
            (addr, val)
        }
        0xEE => {
            cpu.cycles += 6;
            let addr = get_address_absolute(cpu, bus);
            let val = bus.read(addr);
            (addr, val)
        }
        0xFE => {
            cpu.cycles += 7;
            let addr = get_address_absolute_x(cpu, bus).0;
            let val = bus.read(addr);
            (addr, val)
        }
        _ => unreachable!(),
    };

    let result = value.wrapping_add(1);
    bus.write(addr, result);
    cpu.update_zero_negative(result);
}
