use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[macro_export]
#[cfg(target_arch = "wasm32")]
macro_rules! console_log {
    ($($t:tt)*) => ($crate::log::log(&format_args!($($t)*).to_string()))
}

#[macro_export]
#[cfg(not(target_arch = "wasm32"))]
macro_rules! console_log {
    ($($t:tt)*) => (println!($($t)*))
}
