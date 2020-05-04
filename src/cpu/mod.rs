pub mod register;

use register::*;
use std::rc::Rc;
use std::cell::RefCell;
use web_sys::console;
use super::{Tick, bus::BusInterface};

pub struct Cpu {
    bus: Rc<RefCell<BusInterface>>,
    pc: Register<u16>,
    sp: Register<u8>,
    a: Register<u8>,
    x: Register<u8>,
    y: Register<u8>,
    s: Register<u8>,
    p: Register<u8>,
    skip_ticks: u64,
    debug: String,
}

impl Cpu {
    pub fn new(bus: Rc<RefCell<BusInterface>>) -> Cpu {
        Cpu {
            bus,
            pc: Register::default(), // program counter
            sp: Register::default(), // stack pointer
            a: Register::default(),
            x: Register::default(),
            y: Register::default(),
            s: Register::default(),
            p: Register::default(), // processor flags
            skip_ticks: 0,
            debug: String::new(),
        }
    }

    pub fn reset(&mut self) {
        let bus = self.bus.borrow();

        self.pc.store(bus.read_word(0xFFFC));
        self.skip_ticks = 5;

        console::log_1(&format!("PC: {}", self.pc.load()).into());
    }

    pub fn next_byte(&mut self) -> u8 {
        let pc = self.pc.load();

        self.pc.add(1);
        self.bus.borrow().read(pc)
    }

    pub fn next_word(&mut self) -> u16 {
        let pc = self.pc.load();

        self.pc.add(2);
        self.bus.borrow().read_word(pc)
    }

    fn execute(&mut self, opcode: u8) {
        match opcode {
            // fake jmp
            0x00 => {
                let addr = self.next_word();
                self.pc.store(addr);
                self.skip_ticks += 4;
            },
            _ => panic!("invalid opcode: {:#04x}", opcode),
        }
    }
}

impl Tick for Cpu {
    fn tick(&mut self) {
        if self.skip_ticks > 0 {
            self.skip_ticks -= 1;
            return;
        }

        let opcode = self.next_byte();
        self.execute(opcode);
    }
}