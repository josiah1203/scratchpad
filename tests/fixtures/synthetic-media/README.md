# Synthetic media fixtures

All files in this directory are original generated test data with no third-party source material.
Rights/provenance: synthetic; usage: test-only. These fixtures are not production artwork and carry
no user or customer content.

`checkerboard-4x4.ppm` is an ASCII Netpbm P3 image with a 4×4 RGB pixel grid and an 8-bit channel
maximum. It is intentionally small and dependency-free for parser and asset-ingest tests.

`synthetic-gray-16x16-4frames-24fps.y4m` is a deterministic YUV4MPEG2 video fixture: four 16×16
progressive frames at 24 fps, with planar 4:2:0 JPEG-range chroma and four distinct grayscale luma
levels. It is uncompressed test media for downstream ingest work, not a product codec decision.
Regenerate it from the repository root with:

```sh
rustc --edition=2024 -D warnings scripts/generate-synthetic-y4m.rs -o /tmp/generate-synthetic-y4m
/tmp/generate-synthetic-y4m /tmp/synthetic-video.y4m
cmp /tmp/synthetic-video.y4m tests/fixtures/synthetic-media/synthetic-gray-16x16-4frames-24fps.y4m
```

The generator uses only the Rust standard library and refuses to overwrite an existing file. The
integration test checks the stream header, each frame boundary, plane sizes, and deterministic
sample values. Regular CI recompiles the generator and byte-compares the regenerated fixture.

`synthetic-tone-1khz-8khz-mono.wav` is 100 ms of mono, 16-bit little-endian PCM at 8 kHz. Its
repeating eight-sample waveform approximates a 1 kHz tone without using nondeterministic floating-
point synthesis. Regenerate the fixture from the repository root with:

```sh
rustc --edition=2024 -D warnings scripts/generate-synthetic-wav.rs -o /tmp/generate-synthetic-wav
/tmp/generate-synthetic-wav /tmp/synthetic-tone.wav
cmp /tmp/synthetic-tone.wav tests/fixtures/synthetic-media/synthetic-tone-1khz-8khz-mono.wav
```

The generator refuses to overwrite an existing file. The integration test checks the RIFF/WAVE
container sizes, PCM format fields, sample alignment, sample count, and exact repeated waveform.
The regular CI workflow recompiles the generator and compares its output with the checked-in
fixture to detect accidental drift.

On macOS, `swift scripts/verify-synthetic-wav.swift tests/fixtures/synthetic-media/synthetic-tone-1khz-8khz-mono.wav`
decodes the fixture with AVFoundation and verifies all 800 decoded PCM samples. The scheduled
extended workflow runs this native-decoder check; no third-party codec is added to the product.
The Y4M fixture is currently checked structurally only; exercising it through the eventual approved
video decoder remains downstream work.
