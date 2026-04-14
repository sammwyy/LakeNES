use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

fn branch(cpu: &mut CPU, bus: &mut Bus, condition: bool) {
    let offset = cpu.fetch_byte(bus) as i8;
    if condition {
        cpu.cycles += 1; // 1 cycle if branch taken
        let old_pc = cpu.pc;
        cpu.pc = cpu.pc.wrapping_add(offset as u16);
        if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) {
            cpu.cycles += 1; // 1 more if page crossed
        }
    }
    cpu.cycles += 2; // Always 2 cycles for fetch + base execute
}

// NOTE: The previous implementations had:
// if condition { cycles += 3; ... crossed += 1 } else { cycles += 2 }
// My 'branch' function above does: cycles += 2; if condition { cycles += 1; ... }
// Which is equivalent.

pub fn bcc(cpu: &mut CPU, bus: &mut Bus) {
    let cond = !cpu.get_flag(FLAG_C);
    branch(cpu, bus, cond);
}

pub fn bcs(cpu: &mut CPU, bus: &mut Bus) {
    let cond = cpu.get_flag(FLAG_C);
    branch(cpu, bus, cond);
}

pub fn beq(cpu: &mut CPU, bus: &mut Bus) {
    let cond = cpu.get_flag(FLAG_Z);
    branch(cpu, bus, cond);
}

pub fn bmi(cpu: &mut CPU, bus: &mut Bus) {
    let cond = cpu.get_flag(FLAG_N);
    branch(cpu, bus, cond);
}

pub fn bne(cpu: &mut CPU, bus: &mut Bus) {
    let cond = !cpu.get_flag(FLAG_Z);
    branch(cpu, bus, cond);
}

pub fn bpl(cpu: &mut CPU, bus: &mut Bus) {
    let cond = !cpu.get_flag(FLAG_N);
    branch(cpu, bus, cond);
}

pub fn bvc(cpu: &mut CPU, bus: &mut Bus) {
    let cond = !cpu.get_flag(FLAG_V);
    branch(cpu, bus, cond);
}

pub fn bvs(cpu: &mut CPU, bus: &mut Bus) {
    let cond = cpu.get_flag(FLAG_V);
    branch(cpu, bus, cond);
}
