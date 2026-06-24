#![no_main]

use libfuzzer_sys::fuzz_target;
use do_memory_core::Episode;

fuzz_target!(|data: &[u8]| {
    // Try to deserialize from the fuzzed data
    if let Ok(episode) = postcard::from_bytes::<Episode>(data) {
        // If successful, roundtrip it
        if let Ok(serialized) = postcard::to_allocvec(&episode) {
            let deserialized: Episode = postcard::from_bytes(&serialized).expect("Roundtrip should never fail if first serialization succeeded");
            assert_eq!(episode, deserialized);
        }
    }
});
