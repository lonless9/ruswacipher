use wasm_bindgen::prelude::*;

// Import the `console.log` function from the browser
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to make console.log easier to use
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Export a simple add function
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    console_log!("WASM: add({}, {}) called", a, b);
    a + b
}

// Export a multiply function
#[wasm_bindgen]
pub fn multiply(a: i32, b: i32) -> i32 {
    console_log!("WASM: multiply({}, {}) called", a, b);
    a * b
}

// Export a function that returns a string
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    console_log!("WASM: greet('{}') called", name);
    format!("Hello, {}! This message comes from encrypted WASM.", name)
}

// Export a function to test memory operations
#[wasm_bindgen]
pub fn sum_array(numbers: &[i32]) -> i32 {
    console_log!("WASM: sum_array called with {} numbers", numbers.len());
    numbers.iter().sum()
}

// Called when the WASM module is instantiated
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("🦀 Test WASM module loaded successfully!");
}
