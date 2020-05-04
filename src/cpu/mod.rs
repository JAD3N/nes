pub mod register;

use std::rc::Rc;
use std::cell::RefCell;
use super::{Tick, bus::BusInterface};

pub struct Cpu {
    bus: Rc<RefCell<BusInterface>>,
}

impl Cpu {
    pub fn new(bus: Rc<RefCell<BusInterface>>) -> Cpu {
        Cpu { bus }
    }
}

impl Tick for Cpu {
    fn tick(&mut self) {}
}