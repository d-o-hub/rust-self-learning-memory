#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Cursor;
use do_memory_mcp::jsonrpc::read_next_message;

fuzz_target!(|data: &[u8]| {
    let mut cursor = Cursor::new(data);
    let _ = read_next_message(&mut cursor);
});
