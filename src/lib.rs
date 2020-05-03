extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;
use console_error_panic_hook;

#[wasm_bindgen]
pub struct Test {}

#[wasm_bindgen]
impl Test {
    pub fn new() {
        console_error_panic_hook::set_once();
    }
}