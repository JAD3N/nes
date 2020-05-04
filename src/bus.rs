use std::rc::Rc;
use std::cell::RefCell;
use super::{
    cpu::Cpu,
    ppu::Ppu,
    memory::Memory,
};

pub trait BusRead {
    fn read(&self, addr: usize) -> Option<u8>;
}

pub trait BusWrite {
    fn write(&mut self, addr: usize, value: u8) -> bool;
}

pub struct BusInterface {
    pub readers: Vec<Rc<RefCell<dyn BusRead>>>,
    pub writers: Vec<Rc<RefCell<dyn BusWrite>>>,
}

impl BusInterface {
    pub fn new() -> Rc<RefCell<BusInterface>> {
        Rc::new(RefCell::new(BusInterface {
            readers: vec![],
            writers: vec![],
        }))
    }

    pub fn read(&self, addr: usize) -> u8 {
        let addr = addr & 0xffff;

        for reader in self.readers.iter() {
            if let Some(value) = reader.borrow().read(addr) {
                return value;
            }
        }

        0
    }

    pub fn read_word(&self, addr: usize) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr + 1) as u16;

        (hi << 8) | lo
    }

    pub fn write(&mut self, addr: usize, value: u8) {
        let addr = addr & 0xffff;

        for writer in self.writers.iter() {
            if writer.borrow_mut().write(addr, value) {
                break;
            }
        }
    }
}

pub struct Bus {
    pub interface: Rc<RefCell<BusInterface>>,

    pub cpu: Rc<RefCell<Cpu>>,
    pub ppu: Rc<RefCell<Ppu>>,
    pub ram: Rc<RefCell<Memory>>,
}

// read()

impl Bus {
    pub fn new() -> Bus {
        let interface = BusInterface::new();

        let cpu = Rc::new(RefCell::new(Cpu::new(interface.clone())));
        let ppu = Rc::new(RefCell::new(Ppu::new()));
        let ram = Rc::new(RefCell::new(Memory::new()));

        {
            let mut bus = interface.borrow_mut();

            // add ram to bus
            bus.readers.push(ram.clone());
            bus.writers.push(ram.clone());
        }

        Bus { interface, cpu, ppu, ram }
    }

    pub fn read(&self, addr: usize) -> u8 {
        self.interface.borrow().read(addr)
    }

    pub fn write(&mut self, addr: usize, value: u8) {
        self.interface.borrow_mut().write(addr, value)
    }
}