use std::rc::Rc;
use std::cell::RefCell;
use super::{Tick, bus::BusInterface};

pub struct Cpu {
    pub bus: Rc<RefCell<BusInterface>>,
    pub pc: u16,
    pub sp: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub s: u8,
    pub p: u8,
    pub addr: usize,
    pub skip_ticks: u64,
    pub debug: String,
}

#[derive(PartialEq)]
enum Mode {
    Accumulator,
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


#[derive(PartialEq)]
enum Interrupt {
    Nmi,
    Reset,
    Irq,
    Break,
}

#[derive(PartialEq)]
enum Flag {
    Carry = 0b00000001,
    Zero = 0b00000010,
    InterruptDisable = 0b00000100,
    Decimal = 0b00001000,
    Break = 0b00010000,
    Push = 0b00100000,
    Overflow = 0b01000000,
    Negative = 0b10000000,
}

type Instruction = (&'static str, Mode, u64);

impl Cpu {
    pub fn new(bus: Rc<RefCell<BusInterface>>) -> Cpu {
        Cpu {
            bus,
            pc: 0, // program counter
            sp: 0, // stack pointer
            a: 0,
            x: 0,
            y: 0,
            s: 0,
            p: 0, // processor flags
            addr: 0,
            skip_ticks: 0,
            debug: String::new(),
        }
    }

    pub fn reset(&mut self) {
        let bus = self.bus.borrow();

        //self.pc.store(bus.read_word(0xFFFC));
        self.skip_ticks = 5;
    }

    pub fn write(&mut self, addr: usize, value: u8) {
        self.bus.borrow_mut().write(addr, value);
    }

    pub fn read(&self, addr: usize) -> u8 {
        self.bus.borrow().read(addr)
    }

    pub fn read_word(&self, addr: usize) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr + 1) as u16;

        (hi << 8) | lo
    }

    pub fn next(&mut self) -> u8 {
        let byte = self.read(self.pc as usize);
        self.pc = self.pc.overflowing_add(1).0;
        byte
    }

    pub fn next_word(&mut self) -> u16 {
        let lo = self.next() as u16;
        let hi = self.next() as u16;

        (hi << 8) | lo
    }

    pub fn push(&mut self, value: u8) {
        let addr = 0x0100 + (self.sp as u16);
        self.write(addr as usize, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn push_word(&mut self, value: u16) {
        self.push((value >> 8) as u8);
        self.push(value as u8);
    }

    pub fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let addr = 0x0100 + (self.sp as u16);
        self.read(addr as usize)
    }

    pub fn pop_word(&mut self) -> u16 {
        (self.pop() as u16) | ((self.pop() as u16) << 8)
    }

    fn read_operand_address(&mut self, mode: Mode) -> (usize, bool) {
        match mode {
            Mode::Immediate => {
                let addr = self.pc as usize;
                self.pc = self.pc.overflowing_add(1).0;

                (addr, false)
            },
            Mode::ZeroPage => (self.next() as usize, false),
            Mode::ZeroPageX => {
                let mut addr = self.next() as u16;

                addr += self.x as u16;
                addr &= 0x00ff;

                (addr as usize, false)
            },
            Mode::ZeroPageY => {
                let mut addr = self.next() as u16;

                addr += self.y as u16;
                addr &= 0x00ff;

                (addr as usize, false)
            },
            Mode::Relative => {
                let mut addr = self.next() as u16;

                if addr & 0b10000000 != 0 {
                    addr |= 0xff00;
                }

                (addr as usize, false)
            },
            Mode::Absolute => (self.next_word() as usize, false),
            Mode::AbsoluteX => {
                let lo = self.next() as u16;
                let hi = self.next() as u16;
                let mut addr = (hi << 8) | lo;

                // add x to absolute address
                addr += self.x as u16;

                // checks whether page has changed
                (addr as usize, addr & 0xff00 != hi << 8)
            },
            Mode::AbsoluteY => {
                let lo = self.next() as u16;
                let hi = self.next() as u16;
                let mut addr = (hi << 8) | lo;

                // add y to absolute address
                addr += self.y as u16;

                // checks whether page has changed
                (addr as usize, addr & 0xff00 != hi << 8)
            },
            Mode::Indirect => {
                let mut lo = self.next() as u16;
                let mut hi = self.next() as u16;
                let mut addr = (hi << 8) | lo;
                let bus = self.bus.borrow();

                // simulate page boundary hardware bug
                if lo == 0x00ff {
                    lo = bus.read((addr + 0) as usize) as u16;
                    hi = bus.read((addr & 0xff00) as usize) as u16;
                } else {
                    lo = bus.read((addr + 0) as usize) as u16;
                    hi = bus.read((addr + 1) as usize) as u16;
                }

                addr = (hi << 8) | lo;

                (addr as usize, false)
            },
            Mode::IndirectX => {
                let mut addr = self.next() as u16;
                let bus = self.bus.borrow();
                let lo = bus.read(((addr + (self.x as u16) + 0) & 0x00FF) as usize) as u16;
                let hi = bus.read(((addr + (self.x as u16) + 1) & 0x00FF) as usize) as u16;

                addr = (hi << 8) | lo;

                (addr as usize, false)
            },
            Mode::IndirectY => {
                let mut addr = self.next() as u16;
                let bus = self.bus.borrow();
                let lo = bus.read(((addr + 0) & 0x00ff) as usize) as u16;
                let hi = bus.read(((addr + 1) & 0x00ff) as usize) as u16;

                // create addr from bytes
                addr = (hi << 8) | lo;

                (addr as usize, addr & 0xff00 != hi << 8)
            }
            _ => panic!("Invalid addressing mode!"),
        }
    }

    fn get_instruction(opcode: usize) -> Instruction {
        match opcode {
            // ADC
            0x69 => ("ADC", Mode::Immediate, 2),
            0x65 => ("ADC", Mode::ZeroPage, 3),
            0x75 => ("ADC", Mode::ZeroPageX, 4),
            0x6d => ("ADC", Mode::Absolute, 4),
            0x7d => ("ADC", Mode::AbsoluteX, 4),
            0x79 => ("ADC", Mode::AbsoluteY, 4),
            0x61 => ("ADC", Mode::IndirectX, 6),
            0x71 => ("ADC", Mode::IndirectY, 5),

            // AND
            0x29 => ("AND", Mode::Immediate, 2),
            0x25 => ("AND", Mode::ZeroPage, 3),
            0x35 => ("AND", Mode::ZeroPageX, 4),
            0x2d => ("AND", Mode::Absolute, 4),
            0x3d => ("AND", Mode::AbsoluteX, 4),
            0x39 => ("AND", Mode::AbsoluteY, 4),
            0x21 => ("AND", Mode::IndirectX, 6),
            0x31 => ("AND", Mode::IndirectY, 5),

            // ASL
            0x01 => ("ASL", Mode::Accumulator, 2),
            0x06 => ("ASL", Mode::ZeroPage, 5),
            0x16 => ("ASL", Mode::ZeroPageX, 6),
            0x0e => ("ASL", Mode::Absolute, 6),
            0x1e => ("ASL", Mode::AbsoluteX, 7),

            // BCC
            0x90 => ("BCC", Mode::Relative, 2),

            // BCS
            0xb0 => ("BCS", Mode::Relative, 2),

            // BEQ
            0xf0 => ("BEQ", Mode::Relative, 2),

            // BIT
            0x24 => ("BIT", Mode::ZeroPage, 3),
            0x2c => ("BIT", Mode::Absolute, 4),

            // BMI
            0x30 => ("BMI", Mode::Relative, 2),

            // BNE
            0xd0 => ("BNE", Mode::Relative, 2),

            // BPL
            0x10 => ("BPL", Mode::Relative, 2),

            // BRK
            0x00 => ("BRK", Mode::Implied, 7),

            // BVC
            0x50 => ("BVC", Mode::Relative, 2),

            // BVS
            0x70 => ("BVS", Mode::Relative, 2),

            // Clear Flags
            0x18 => ("CLC", Mode::Implied, 2),
            0xd8 => ("CLD", Mode::Implied, 2),
            0x58 => ("CLI", Mode::Implied, 2),
            0xb8 => ("CLV", Mode::Implied, 2),

            _ => panic!("Unknown opcode: {:#04x}", opcode),
        }
    }

    fn execute(&mut self, opcode: usize) {
        let (name, mode, skip_ticks) = Self::get_instruction(opcode);

        if match name {
            "ADC" => self.adc(mode),
            "AND" => self.and(mode),
            "ASL" => self.asl(mode),
            "BCC" => self.bcc(),
            "BCS" => self.bcs(),
            "BEQ" => self.beq(),
            "BIT" => self.bit(mode),
            "BMI" => self.bmi(),
            "BNE" => self.bne(),
            "BPL" => self.bpl(),
            "BRK" => self.brk(),
            "BVC" => self.bvc(),
            "BVS" => self.bvs(),
            "CLC" => self.clc(),
            "CLD" => self.cld(),
            "CLI" => self.cli(),
            "CLV" => self.clv(),
            _ => false,
        } {
            // add additional tick
            self.skip_ticks += 1;
        }

        // add instruction skip ticks
        self.skip_ticks = skip_ticks;

        self.debug.clear();
        self.debug.push_str(name);
    }

    fn set_flag(&mut self, flag: Flag, value: bool) {
        let flag = flag as u8;

        if value {
            self.p |= flag;
        } else {
            self.p &= !flag;
        }
    }

    fn get_flag(&self, flag: Flag) -> bool {
        self.p & (flag as u8) > 0
    }

    fn adc(&mut self, mode: Mode) -> bool {
        let (addr, skip_tick) = self.read_operand_address(mode);
        let operand = self.read(addr);

        let value = (self.a as u16)
            + (operand as u16)
            + (self.get_flag(Flag::Carry) as u16);

        self.set_flag(Flag::Carry, value > 255);
        self.set_flag(Flag::Zero, (value & 0x00ff) == 0);

        // !(A ^ O) & (A ^ V) & 0x0080
        self.set_flag(Flag::Overflow,
            (
                !((self.a as u16) ^ (operand as u16))
                & ((self.a as u16) ^ (value as u16))
            )
            & 0x0080 != 0
        );

        self.set_flag(Flag::Negative, (value & 0b10000000) != 0);

        // set a to value byte
        self.a = (value & 0x00ff) as u8;

        skip_tick
    }

    fn and(&mut self, mode: Mode) -> bool {
        let (addr, skip_tick) = self.read_operand_address(mode);
        let operand = self.read(addr);

        self.a &= operand;

        self.set_flag(Flag::Zero, (self.a & 0x00ff) == 0);
        self.set_flag(Flag::Negative, (self.a & 0b10000000) != 0);

        skip_tick
    }

    fn asl(&mut self, mode: Mode) -> bool {
        let value = if mode == Mode::Accumulator {
            let value = (self.a as u16) << 1;

            // truncate shifted value
            self.a = (value & 0x00ff) as u8;

            // return value for flags
            value
        } else {
            let addr = self.read_operand_address(mode).0;
            let operand = self.read(addr);
            let value = (operand as u16) << 1;

            self.write(addr, (value & 0x00ff) as u8);

            value
        };

        self.set_flag(Flag::Carry, (value & 0xff00) > 0);
        self.set_flag(Flag::Zero, (value & 0x00ff) == 0);
        self.set_flag(Flag::Negative, (value & 0b10000000) != 0);

        false
    }

    fn branch(&mut self, cond: bool) {
        let addr = self.read_operand_address(Mode::Relative).0;

        if cond {
            self.skip_ticks += 1;

            if addr & 0xff00 != (self.pc as usize) & 0xff00 {
                self.skip_ticks += 1;
            }

            self.pc = addr as u16;
        }
    }

    fn bcc(&mut self) -> bool {
        self.branch(!self.get_flag(Flag::Carry));
        false
    }

    fn bcs(&mut self) -> bool {
        self.branch(self.get_flag(Flag::Carry));
        false
    }

    fn beq(&mut self) -> bool {
        self.branch(self.get_flag(Flag::Zero));
        false
    }

    fn bit(&mut self, mode: Mode) -> bool {
        let (addr, skip_tick) = self.read_operand_address(mode);
        let operand = self.read(addr);
        let value = self.a & operand;

        self.set_flag(Flag::Zero, value == 0);
        self.set_flag(Flag::Overflow, (operand & 0b01000000) != 0);
        self.set_flag(Flag::Negative, (operand & 0b10000000) != 0);

        false
    }

    fn bmi(&mut self) -> bool {
        self.branch(self.get_flag(Flag::Negative));
        false
    }

    fn bne(&mut self) -> bool {
        self.branch(!self.get_flag(Flag::Zero));
        false
    }

    fn bpl(&mut self) -> bool {
        self.branch(!self.get_flag(Flag::Negative));
        false
    }

    fn brk(&mut self) -> bool {
        self.next();

        self.set_flag(Flag::InterruptDisable, true);
        self.push_word(self.pc);

        self.set_flag(Flag::Break, true);
        self.push(self.s);
        self.set_flag(Flag::Break, false);

        self.pc = self.read_word(0xFFFE);

        false
    }

    fn bvc(&mut self) -> bool {
        self.branch(!self.get_flag(Flag::Overflow));
        false
    }

    fn bvs(&mut self) -> bool {
        self.branch(self.get_flag(Flag::Overflow));
        false
    }

    fn clc(&mut self) -> bool {
        self.set_flag(Flag::Carry, false);
        false
    }

    fn cld(&mut self) -> bool {
        self.set_flag(Flag::Decimal, false);
        false
    }

    fn cli(&mut self) -> bool {
        self.set_flag(Flag::InterruptDisable, false);
        false
    }

    fn clv(&mut self) -> bool {
        self.set_flag(Flag::Overflow, false);
        false
    }
}

impl Tick for Cpu {
    fn tick(&mut self) {
        if self.skip_ticks > 0 {
            self.skip_ticks -= 1;
            return;
        }

        let opcode = self.next() as usize;
        self.execute(opcode);
    }
}