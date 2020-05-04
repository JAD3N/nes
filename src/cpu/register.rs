use std::ops::{Add, Sub};

pub struct Register<T> {
    data: T,
}

impl Register<u8> {
    pub const BITS: usize = 8;

    pub fn store(&mut self, value: u8) {
        self.data = value;
    }

    pub fn store_bit(&mut self, pos: u8, value: u8) {
        self.data = self.data & !(1 << pos) | ((value & 1) << pos);
    }

    pub fn set_bit(&mut self, pos: u8) {
        self.store_bit(pos, 1);
    }

    pub fn clear_bit(&mut self, pos: u8) {
        self.store_bit(pos, 0);
    }

    pub fn clear(&mut self) {
        self.data = 0x0;
    }

    pub fn load(&self) -> u8 {
        self.data
    }

    pub fn load_bit(&self, pos: u8) -> u8 {
        (self.data >> pos) & 1
	}

    pub fn add(&mut self, value: u8) {
        self.data = self.data.wrapping_add(value);
    }

    pub fn sub(&mut self, value: u8) {
       self.data = self.data.wrapping_sub(value);
    }
}

impl Default for Register<u8> {
    fn default() -> Self {
        Register { data: 0 }
    }
}

impl Register<u16> {
    pub const BITS: usize = 16;

    pub fn store(&mut self, value: u16) {
        self.data = value;
    }

    pub fn store_bit(&mut self, pos: u8, value: u16) {
        self.data = self.data & !(1 << pos) | ((value & 1) << pos);
    }

    pub fn set_bit(&mut self, pos: u8) {
        self.store_bit(pos, 1);
    }

    pub fn clear_bit(&mut self, pos: u8) {
        self.store_bit(pos, 0);
    }

    pub fn clear(&mut self) {
        self.data = 0x0;
    }

    pub fn load(&self) -> u16 {
        self.data
    }

    pub fn load_bit(&self, pos: u8) -> u16 {
        (self.data >> pos) & 1
	}

    pub fn add(&mut self, value: u16) {
        self.data = self.data.wrapping_add(value);
    }

    pub fn sub(&mut self, value: u16) {
       self.data = self.data.wrapping_sub(value);
    }
}

impl Default for Register<u16> {
    fn default() -> Self {
        Register { data: 0 }
    }
}