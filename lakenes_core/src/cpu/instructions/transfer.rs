// use super::*; // removed unused
use crate::cpu::CPU;

pub fn tax(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.x = cpu.a;
    cpu.update_zero_negative(cpu.x);
}

pub fn tay(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.y = cpu.a;
    cpu.update_zero_negative(cpu.y);
}

pub fn tsx(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.x = cpu.sp;
    cpu.update_zero_negative(cpu.x);
}

pub fn txa(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.a = cpu.x;
    cpu.update_zero_negative(cpu.a);
}

pub fn txs(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.sp = cpu.x;
}

pub fn tya(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.a = cpu.y;
    cpu.update_zero_negative(cpu.a);
}
