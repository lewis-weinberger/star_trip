#![cfg(target_arch = "wasm32")]

use star_trip::Terminal;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// TODO: write some tests!

#[cfg(test)]
pub fn two() -> usize {
    2
}

#[wasm_bindgen_test]
fn pass() {
    assert_eq!(1 + 1, two());
}
