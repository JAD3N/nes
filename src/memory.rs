use super::bus::{BusRead, BusWrite};

pub const SIZE: usize = 2 * 1024;

pub struct Memory {
    pub data: [u8; SIZE],
}

impl Memory {
    pub fn new() -> Memory {
        Memory { data: [0; SIZE] }
    }
}

impl BusRead for Memory {
    fn read(&self, addr: u16) -> Option<u8> {
        if addr <= 0x1FFF {
            Some(self.data[(addr & 0x07FF) as usize])
        } else {
            None
        }
    }
}

impl BusWrite for Memory {
    fn write(&mut self, addr: u16, value: u8) -> bool {
        if addr <= 0x1FFF {
            self.data[(addr & 0x07FF) as usize] = value;
            true
        } else {
            false
        }
    }
}