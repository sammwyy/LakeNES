use alloc::string::{String, ToString};
use alloc::format;

#[derive(Copy, Clone)]
pub struct InstructionInfo {
    pub opcode: u8,
    pub name: &'static str,
    pub length: u8,
    pub cycles: u8,
    pub addr_mode: &'static str,
}

pub const OPCODES: [Option<InstructionInfo>; 256] = {
    let mut table = [const { None }; 256];
    
    // Helper to add instructions
    // This is a simplified table. For a real emulator, we'd want more detail.
    // Format: (opcode, name, length, cycles, addr_mode)
    let insts: &[(u8, &str, u8, u8, &str)] = &[
        (0x00, "BRK", 1, 7, "Implied"),
        (0x01, "ORA", 2, 6, "Indirect,X"),
        (0x05, "ORA", 2, 3, "ZeroPage"),
        (0x06, "ASL", 2, 5, "ZeroPage"),
        (0x08, "PHP", 1, 3, "Implied"),
        (0x09, "ORA", 2, 2, "Immediate"),
        (0x0A, "ASL", 1, 2, "Accumulator"),
        (0x0D, "ORA", 3, 4, "Absolute"),
        (0x0E, "ASL", 3, 6, "Absolute"),
        (0x10, "BPL", 2, 2, "Relative"),
        (0x11, "ORA", 2, 5, "Indirect,Y"),
        (0x15, "ORA", 2, 4, "ZeroPage,X"),
        (0x16, "ASL", 2, 6, "ZeroPage,X"),
        (0x18, "CLC", 1, 2, "Implied"),
        (0x19, "ORA", 3, 4, "Absolute,Y"),
        (0x1D, "ORA", 3, 4, "Absolute,X"),
        (0x1E, "ASL", 3, 7, "Absolute,X"),
        (0x20, "JSR", 3, 6, "Absolute"),
        (0x21, "AND", 2, 6, "Indirect,X"),
        (0x24, "BIT", 2, 3, "ZeroPage"),
        (0x25, "AND", 2, 3, "ZeroPage"),
        (0x26, "ROL", 2, 5, "ZeroPage"),
        (0x28, "PLP", 1, 4, "Implied"),
        (0x29, "AND", 2, 2, "Immediate"),
        (0x2A, "ROL", 1, 2, "Accumulator"),
        (0x2C, "BIT", 3, 4, "Absolute"),
        (0x2D, "AND", 3, 4, "Absolute"),
        (0x2E, "ROL", 3, 6, "Absolute"),
        (0x30, "BMI", 2, 2, "Relative"),
        (0x31, "AND", 2, 5, "Indirect,Y"),
        (0x35, "AND", 2, 4, "ZeroPage,X"),
        (0x36, "ROL", 2, 6, "ZeroPage,X"),
        (0x38, "SEC", 1, 2, "Implied"),
        (0x39, "AND", 3, 4, "Absolute,Y"),
        (0x3D, "AND", 3, 4, "Absolute,X"),
        (0x3E, "ROL", 3, 7, "Absolute,X"),
        (0x40, "RTI", 1, 6, "Implied"),
        (0x41, "EOR", 2, 6, "Indirect,X"),
        (0x45, "EOR", 2, 3, "ZeroPage"),
        (0x46, "LSR", 2, 5, "ZeroPage"),
        (0x48, "PHA", 1, 3, "Implied"),
        (0x49, "EOR", 2, 2, "Immediate"),
        (0x4A, "LSR", 1, 2, "Accumulator"),
        (0x4C, "JMP", 3, 3, "Absolute"),
        (0x4D, "EOR", 3, 4, "Absolute"),
        (0x4E, "LSR", 3, 6, "Absolute"),
        (0x50, "BVC", 2, 2, "Relative"),
        (0x51, "EOR", 2, 5, "Indirect,Y"),
        (0x55, "EOR", 2, 4, "ZeroPage,X"),
        (0x56, "LSR", 2, 6, "ZeroPage,X"),
        (0x58, "CLI", 1, 2, "Implied"),
        (0x59, "EOR", 3, 4, "Absolute,Y"),
        (0x5D, "EOR", 3, 4, "Absolute,X"),
        (0x5E, "LSR", 3, 7, "Absolute,X"),
        (0x60, "RTS", 1, 6, "Implied"),
        (0x61, "ADC", 2, 6, "Indirect,X"),
        (0x65, "ADC", 2, 3, "ZeroPage"),
        (0x66, "ROR", 2, 5, "ZeroPage"),
        (0x68, "PLA", 1, 4, "Implied"),
        (0x69, "ADC", 2, 2, "Immediate"),
        (0x6A, "ROR", 1, 2, "Accumulator"),
        (0x6C, "JMP", 3, 5, "Indirect"),
        (0x6D, "ADC", 3, 4, "Absolute"),
        (0x6E, "ROR", 3, 6, "Absolute"),
        (0x70, "BVS", 2, 2, "Relative"),
        (0x71, "ADC", 2, 5, "Indirect,Y"),
        (0x75, "ADC", 2, 4, "ZeroPage,X"),
        (0x76, "ROR", 2, 6, "ZeroPage,X"),
        (0x78, "SEI", 1, 2, "Implied"),
        (0x79, "ADC", 3, 4, "Absolute,Y"),
        (0x7D, "ADC", 3, 4, "Absolute,X"),
        (0x7E, "ROR", 3, 7, "Absolute,X"),
        (0x81, "STA", 2, 6, "Indirect,X"),
        (0x84, "STY", 2, 3, "ZeroPage"),
        (0x85, "STA", 2, 3, "ZeroPage"),
        (0x86, "STX", 2, 3, "ZeroPage"),
        (0x88, "DEY", 1, 2, "Implied"),
        (0x8A, "TXA", 1, 2, "Implied"),
        (0x8C, "STY", 3, 4, "Absolute"),
        (0x8D, "STA", 3, 4, "Absolute"),
        (0x8E, "STX", 3, 4, "Absolute"),
        (0x90, "BCC", 2, 2, "Relative"),
        (0x91, "STA", 2, 6, "Indirect,Y"),
        (0x94, "STY", 2, 4, "ZeroPage,X"),
        (0x95, "STA", 2, 4, "ZeroPage,X"),
        (0x96, "STX", 2, 4, "ZeroPage,Y"),
        (0x98, "TYA", 1, 2, "Implied"),
        (0x99, "STA", 3, 5, "Absolute,Y"),
        (0x9A, "TXS", 1, 2, "Implied"),
        (0x9D, "STA", 3, 5, "Absolute,X"),
        (0xA0, "LDY", 2, 2, "Immediate"),
        (0xA1, "LDA", 2, 6, "Indirect,X"),
        (0xA2, "LDX", 2, 2, "Immediate"),
        (0xA4, "LDY", 2, 3, "ZeroPage"),
        (0xA5, "LDA", 2, 3, "ZeroPage"),
        (0xA6, "LDX", 2, 3, "ZeroPage"),
        (0xA8, "TAY", 1, 2, "Implied"),
        (0xA9, "LDA", 2, 2, "Immediate"),
        (0xAA, "TAX", 1, 2, "Implied"),
        (0xAC, "LDY", 3, 4, "Absolute"),
        (0xAD, "LDA", 3, 4, "Absolute"),
        (0xAE, "LDX", 3, 4, "Absolute"),
        (0xB0, "BCS", 2, 2, "Relative"),
        (0xB1, "LDA", 2, 5, "Indirect,Y"),
        (0xB4, "LDY", 2, 4, "ZeroPage,X"),
        (0xB5, "LDA", 2, 4, "ZeroPage,X"),
        (0xBB, "LDX", 2, 4, "ZeroPage,Y"),
        (0xB8, "CLV", 1, 2, "Implied"),
        (0xB9, "LDA", 3, 4, "Absolute,Y"),
        (0xBA, "TSX", 1, 2, "Implied"),
        (0xBC, "LDY", 3, 4, "Absolute,X"),
        (0xBD, "LDA", 3, 4, "Absolute,X"),
        (0xBE, "LDX", 3, 4, "Absolute,Y"),
        (0xC0, "CPY", 2, 2, "Immediate"),
        (0xC1, "CMP", 2, 6, "Indirect,X"),
        (0xC4, "CPY", 2, 3, "ZeroPage"),
        (0xC5, "CMP", 2, 3, "ZeroPage"),
        (0xC6, "DEC", 2, 5, "ZeroPage"),
        (0xC8, "INY", 1, 2, "Implied"),
        (0xC9, "CMP", 2, 2, "Immediate"),
        (0xCA, "DEX", 1, 2, "Implied"),
        (0xCC, "CPY", 3, 4, "Absolute"),
        (0xCD, "CMP", 3, 4, "Absolute"),
        (0xCE, "DEC", 3, 6, "Absolute"),
        (0xD0, "BNE", 2, 2, "Relative"),
        (0xD1, "CMP", 2, 5, "Indirect,Y"),
        (0xD5, "CMP", 2, 4, "ZeroPage,X"),
        (0xD6, "DEC", 2, 6, "ZeroPage,X"),
        (0xD8, "CLD", 1, 2, "Implied"),
        (0xD9, "CMP", 3, 4, "Absolute,Y"),
        (0xDD, "CMP", 3, 4, "Absolute,X"),
        (0xDE, "DEC", 3, 7, "Absolute,X"),
        (0xE0, "CPX", 2, 2, "Immediate"),
        (0xE1, "SBC", 2, 6, "Indirect,X"),
        (0xE4, "CPX", 2, 3, "ZeroPage"),
        (0xE5, "SBC", 2, 3, "ZeroPage"),
        (0xE6, "INC", 2, 5, "ZeroPage"),
        (0xE8, "INX", 1, 2, "Implied"),
        (0xE9, "SBC", 2, 2, "Immediate"),
        (0xEA, "NOP", 1, 2, "Implied"),
        (0xEC, "CPX", 3, 4, "Absolute"),
        (0xED, "SBC", 3, 4, "Absolute"),
        (0xEE, "INC", 3, 6, "Absolute"),
        (0xF0, "BEQ", 2, 2, "Relative"),
        (0xF1, "SBC", 2, 5, "Indirect,Y"),
        (0xF5, "SBC", 2, 4, "ZeroPage,X"),
        (0xF6, "INC", 2, 6, "ZeroPage,X"),
        (0xF8, "SED", 1, 2, "Implied"),
        (0xF9, "SBC", 3, 4, "Absolute,Y"),
        (0xFD, "SBC", 3, 4, "Absolute,X"),
        (0xFE, "INC", 3, 7, "Absolute,X"),
    ];

    let mut i = 0;
    while i < insts.len() {
        let (op, name, len, cyc, mode) = insts[i];
        table[op as usize] = Some(InstructionInfo {
            opcode: op,
            name,
            length: len,
            cycles: cyc,
            addr_mode: mode,
        });
        i += 1;
    }
    table
};

pub fn disassemble(rom_data: &[u8], addr: u16) -> (String, u16) {
    if addr as usize >= rom_data.len() {
        return ("???".to_string(), addr + 1);
    }

    let opcode = rom_data[addr as usize];
    if let Some(ref info) = OPCODES[opcode as usize] {
        let mut result = info.name.to_string();
        let next_addr = addr + info.length as u16;
        
        match info.length {
            2 => {
                if addr as usize + 1 < rom_data.len() {
                    let val = rom_data[addr as usize + 1];
                    if info.addr_mode == "Relative" {
                        let offset = val as i8 as i16;
                        let target = (addr as i16 + 2 + offset) as u16;
                        result.push_str(&format!(" ${:04X}", target));
                    } else if info.addr_mode == "Immediate" {
                        result.push_str(&format!(" #${:02X}", val));
                    } else {
                        result.push_str(&format!(" ${:02X}", val));
                    }
                }
            }
            3 => {
                if addr as usize + 2 < rom_data.len() {
                    let lo = rom_data[addr as usize + 1] as u16;
                    let hi = rom_data[addr as usize + 2] as u16;
                    let val = (hi << 8) | lo;
                    result.push_str(&format!(" ${:04X}", val));
                }
            }
            _ => {}
        }
        
        (result, next_addr)
    } else {
        (format!(".byte ${:02X}", opcode), addr + 1)
    }
}
