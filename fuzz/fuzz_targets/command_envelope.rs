#![no_main]

use libfuzzer_sys::fuzz_target;
use platform_runtime::command::CommandEnvelope;

fuzz_target!(|input: &[u8]| {
    if let Ok(json_lines) = std::str::from_utf8(input) {
        for line in json_lines.lines() {
            if let Ok(envelope) = serde_json::from_str::<CommandEnvelope>(line) {
                let _ = envelope.validate();
            }
        }
    }
});
