extern crate wasm_bindgen;

pub mod bus;
pub mod cpu;
pub mod ppu;
pub mod memory;

use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use console_error_panic_hook;
use web_sys::console;

pub const CYCLES_PER_FRAME: u64 = 29781;

pub trait Tick {
    fn tick(&mut self);
}

#[wasm_bindgen]
pub struct Nes {
    remaining_cycles: u64,
    bus: bus::Bus,
}

#[wasm_bindgen]
impl Nes {
    pub fn new() -> Nes {
        // set console panic hook
        console_error_panic_hook::set_once();

        Nes {
            remaining_cycles: 0,
            bus: bus::Bus::new(),
        }
    }

    pub fn reset(&mut self) {
        // reset cpu
        self.bus.cpu.borrow_mut().reset();
    }

    pub fn tick_cpu(&mut self) -> String {
        let mut cpu = self.bus.cpu.borrow_mut();
        cpu.tick();
        cpu.debug.clone()
    }

    pub fn tick_frame(&mut self) {
        let cpu = &mut self.bus.cpu;
        let ppu = &mut self.bus.ppu;

        self.remaining_cycles = CYCLES_PER_FRAME;

        while self.remaining_cycles > 0 {
            // TODO: some nmi/irq stuff goes here (probably interrupts)

            cpu.borrow_mut().tick();

            for _ in 0..ppu::TICKS_PER_CYCLE {
                ppu.borrow_mut().tick();
            }

            self.remaining_cycles -= 1;
        }

        // TODO: Apu tick
    }

    pub fn write(&mut self, addr: usize, value: u8) {
        self.bus.write(addr, value);
    }

    pub fn read(&mut self, addr: usize) -> u8 {
        self.bus.read(addr)
    }
}