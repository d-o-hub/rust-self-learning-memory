#![no_main]

use libfuzzer_sys::fuzz_target;
use do_memory_core::Episode;

fuzz_target!(|bytes: &[u8]| {
    // Roundtrip test: decode arbitrary bytes, then re-encode and decode again.
    if let Ok(episode) = postcard::from_bytes::<Episode>(bytes) {
        let serialized = postcard::to_allocvec(&episode).expect("serialization should never fail");
        let deserialized: Episode =
            postcard::from_bytes(&serialized).expect("Roundtrip should never fail if serialization succeeded");
        // Semantic equality: JSON values are order-insensitive (HashMap metadata)
        // and map non-finite floats to null (NaN reward scores), while any real
        // field corruption diverges. Structural `==` and raw-byte comparison both
        // false-positive on NaN floats and HashMap iteration order respectively.
        if let (Ok(a), Ok(b)) = (
            serde_json::to_value(&episode),
            serde_json::to_value(&deserialized),
        ) {
            assert_eq!(a, b, "roundtrip changed episode semantics");
        }
    }
});
