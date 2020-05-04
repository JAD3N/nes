use super::Tick;

pub const TICKS_PER_CYCLE: u64 = 3;

pub struct Ppu;

impl Ppu {
    pub fn new() -> Ppu {
        Ppu
    }
}

impl Tick for Ppu {
    fn tick(&mut self) {}
}