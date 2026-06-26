#![no_main]

use libfuzzer_sys::fuzz_target;
use do_memory_core::Episode;

fuzz_target!(|episode: Episode| {
    // Roundtrip test
    if let Ok(serialized) = postcard::to_allocvec(&episode) {
        let deserialized: Episode = postcard::from_bytes(&serialized).expect("Roundtrip should never fail if serialization succeeded");
        assert_eq!(episode, deserialized);
    }
});
