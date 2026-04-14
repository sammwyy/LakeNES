#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    OneScreenLow,
    OneScreenHigh,
}

pub trait Mapper {
    fn read_prg(&mut self, addr: u16) -> u8;
    fn write_prg(&mut self, addr: u16, data: u8);
    fn read_chr(&mut self, addr: u16) -> u8;
    fn write_chr(&mut self, addr: u16, data: u8);
    fn irq_flag(&self) -> bool {
        false
    }
    fn mirroring(&self) -> Mirroring {
        Mirroring::Vertical // Default fallback
    }
}

pub mod mapper0;
pub mod mapper1;
pub mod mapper2;
pub mod mapper4;
