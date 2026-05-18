use do_memory_core::retrieval::{HdcEncoder, HVec10240};

fn main() {
    let encoder = HdcEncoder::new();
    let v = encoder.encode("test");
    // Try to access internal data or methods
    println!("HVec10240 size: {} bytes", std::mem::size_of::<HVec10240>());
}
