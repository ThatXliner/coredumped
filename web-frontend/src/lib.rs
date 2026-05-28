//! WASM bindings for xterm.js terminal emulator.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    // XtermBridge is exposed as a global by the bundled JS
    pub type XtermBridge;

    #[wasm_bindgen(constructor, js_class = "XtermBridge")]
    pub fn new(container_id: &str) -> XtermBridge;

    #[wasm_bindgen(method)]
    pub fn write(this: &XtermBridge, data: &str);

    #[wasm_bindgen(method)]
    pub fn clear(this: &XtermBridge);

    #[wasm_bindgen(method)]
    pub fn resize(this: &XtermBridge, cols: u32, rows: u32);

    #[wasm_bindgen(method)]
    pub fn cols(this: &XtermBridge) -> u32;

    #[wasm_bindgen(method)]
    pub fn rows(this: &XtermBridge) -> u32;

    #[wasm_bindgen(method, js_name = setKeyCallback)]
    pub fn set_key_callback(this: &XtermBridge, callback: &Closure<dyn FnMut(String)>);

    #[wasm_bindgen(method, js_name = setResizeCallback)]
    pub fn set_resize_callback(this: &XtermBridge, callback: &Closure<dyn FnMut(u32, u32)>);
}

pub mod app;
