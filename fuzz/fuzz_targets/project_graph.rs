#![no_main]

use libfuzzer_sys::fuzz_target;
use platform_runtime::graph::ProjectGraph;

fuzz_target!(|input: &[u8]| {
    if let Ok(json) = std::str::from_utf8(input) {
        let _ = ProjectGraph::from_json(json);
    }
});
