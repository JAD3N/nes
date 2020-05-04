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

pub enum Mode {
    Implied,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
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

    fn next_byte(&mut self) -> u8 {
        let pc = self.pc.load();

        self.pc.add(1);
        self.bus.borrow().read(pc as usize)
    }

    fn next_word(&mut self) -> u16 {
        let pc = self.pc.load();

        self.pc.add(2);
        self.bus.borrow().read_word(pc as usize)
    }

    fn get_address(&mut self, mode: Mode) -> usize {
        match mode {
            Mode::Immediate => {
                let addr = self.pc.load() as usize;
                self.pc.add(1);
                addr
            },
            Mode::ZeroPage => {
                let mut addr = self.next_byte() as u16;
                addr &= 0x00ff;
                addr as usize
            },
            Mode::ZeroPageX => {
                let mut addr = self.next_byte() as u16;
                addr = addr.wrapping_add(self.x.load() as u16);
                addr &= 0x00ff;
                addr as usize
            },
            Mode::ZeroPageY => {
                let mut addr = self.next_byte() as u16;
                addr = addr.wrapping_add(self.y.load() as u16);
                addr &= 0x00ff;
                addr as usize
            },
            Mode::Relative => {
                let mut addr = self.next_byte() as u16;
                if addr & 0x80 != 0 {
                    addr |= 0xff00;
                }
                addr as usize
            },
            Mode::Absolute => self.next_word() as usize,
            Mode::AbsoluteX => {
                let lo = self.next_byte() as u16;
                let hi = self.next_byte() as u16;

                let mut addr = (hi << 8) | lo;
                addr += self.x.load() as u16;

                // TODO: Page change check

                addr as usize
            },
            Mode::AbsoluteY => {
                let lo = self.next_byte() as u16;
                let hi = self.next_byte() as u16;

                let mut addr = (hi << 8) | lo;
                addr += self.y.load() as u16;

                // TODO: Page change check

                addr as usize
            },
            Mode::Indirect => {
                let ptr_lo = self.next_byte() as u16;
                let ptr_hi = self.next_byte() as u16;
                let ptr = (ptr_hi << 8) | ptr_lo;

                let bus = self.bus.borrow();
                let lo;
                let hi;

                // simulate page boundary hardware bug
                if ptr_lo == 0x00FF {
                    lo = bus.read((ptr + 0) as usize) as u16;
                    hi = bus.read((ptr & 0xff00) as usize) as u16;
                } else {
                    lo = bus.read((ptr + 0) as usize) as u16;
                    hi = bus.read((ptr + 1) as usize) as u16;
                }

                ((hi << 8) | lo) as usize
            },
            Mode::IndirectX => {
                let addr = self.next_byte() as u16;
                let bus = self.bus.borrow();

                let lo = bus.read(((addr + (self.x.load() as u16) + 0) & 0x00FF) as usize);
                let hi = bus.read(((addr + (self.x.load() as u16) + 1) & 0x00FF) as usize);

                ((hi << 8) | lo) as usize
            },
            Mode::IndirectY => {
                let addr = self.next_byte() as u16;
                let bus = self.bus.borrow();

                let lo = bus.read(((addr + 0) & 0x00ff) as usize);
                let hi = bus.read(((addr + 1) & 0x00ff) as usize);

                // TODO: Page change check

                ((hi << 8) | lo) as usize
            }
            _ => panic!("Invalid address mode"),
        }
    }

    fn execute(&mut self, opcode: u8) {
        match opcode {
            // fake jmp
            0x00 => {
                let addr = self.next_word();
                self.pc.store(addr);
                self.skip_ticks += 4;
            },
            _ => panic!("Invalid opcode: {:#04x}", opcode),
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