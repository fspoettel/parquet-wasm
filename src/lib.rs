extern crate web_sys;

mod utils;

use js_sys::Uint8Array;

use arrow2::io::ipc::write::{WriteOptions, StreamWriter};
// NOTE: It's FileReader on latest main but RecordReader in 0.9.2
use arrow2::io::parquet::read::FileReader;
use std::io::Cursor;

use wasm_bindgen;
use wasm_bindgen::prelude::*;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[cfg(target_arch = "wasm32")]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! log {
    ( $( $t:tt )* ) => {
        println!("LOG - {}", format!( $( $t )* ));
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
/*#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;*/

#[wasm_bindgen]
pub fn read_parquet(parquet_file_bytes: &[u8]) -> Result<Uint8Array, JsValue> {
    // Create Parquet reader
    let input_file = Cursor::new(parquet_file_bytes);
    let file_reader = match FileReader::try_new(input_file, None, None, None, None) {
        Ok(file_reader) => file_reader,
        Err(error) => return Err(JsValue::from_str(format!("{}", error).as_str())),
    };
    let schema = file_reader.schema().clone();

    // Create IPC writer
    let mut output_file = Vec::new();
    let options = WriteOptions { compression: None };
    let mut writer = StreamWriter::new(&mut output_file, options);
    match writer.start(&schema, None) {
        Ok(_) => {}
        Err(error) => return Err(JsValue::from_str(format!("{}", error).as_str())),
    }

    // Iterate over reader chunks, writing each into the IPC writer
    for maybe_chunk in file_reader {
        let chunk = match maybe_chunk {
            Ok(chunk) => chunk,
            Err(error) => {
                return Err(JsValue::from_str(format!("{}", error).as_str()));
            }
        };

        match writer.write(&chunk, None) {
            Ok(_) => {}
            Err(error) => return Err(JsValue::from_str(format!("{}", error).as_str())),
        };
    }

    match writer.finish() {
        Ok(_) => {}
        Err(error) => return Err(JsValue::from_str(format!("{}", error).as_str())),
    };

    return Ok(unsafe { Uint8Array::view(&output_file) });
}

#[wasm_bindgen]
pub fn init() {
    utils::set_panic_hook();
}
