use super::*;
use crate::cpu::CPU;

pub fn clc(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.set_flag(FLAG_C, false);
}

pub fn cld(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.set_flag(FLAG_D, false);
}

pub fn cli(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.set_flag(FLAG_I, false);
}

pub fn clv(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.set_flag(FLAG_V, false);
}

pub fn sec(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.set_flag(FLAG_C, true);
}

pub fn sed(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.set_flag(FLAG_D, true);
}

pub fn sei(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.set_flag(FLAG_I, true);
}
