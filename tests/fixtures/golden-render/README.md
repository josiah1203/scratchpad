# Synthetic CPU-raster golden

`skia-vector-8x8.ppm` is a hand-auditable 8×8 RGB expected image for two overlapping, integer-aligned
solid rectangles on a fixed background. It contains no font, imported media, user data, or
third-party artwork. The `renderer-spike` test draws those shapes through Skia's CPU raster surface
and compares every RGB pixel and alpha value to this fixture, reporting the first differing
coordinate. This is an experimental renderer harness, not a production renderer guarantee; run it
on each supported platform before defining cross-platform tolerances.
