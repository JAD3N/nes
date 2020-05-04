use super::bus::{BusRead, BusWrite};

pub const SIZE: usize = 2 * 1024;

pub struct Ram {
    pub memory: [u8; SIZE],
}

impl Ram {
    pub fn new() -> Ram {
        Ram { memory: [0; SIZE] }
    }
}

impl BusRead for Ram {
    fn read(&self, addr: u16) -> Option<u8> {
        if addr <= 0x1FFF {
            Some(self.memory[(addr & 0x07FF) as usize])
        } else {
            None
        }
    }
}

impl BusWrite for Ram {
    fn write(&mut self, addr: u16, value: u8) -> bool {
        if addr <= 0x1FFF {
            self.memory[(addr & 0x07FF) as usize] = value;
            true
        } else {
            false
        }
    }
}