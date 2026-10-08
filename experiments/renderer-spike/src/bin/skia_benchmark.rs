use std::{
    env,
    error::Error,
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

use renderer_spike::{Camera, Scene, render_png, render_rgba};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 800;
const WARMUP_FRAMES: usize = 8;
const MEASURED_FRAMES: usize = 40;

fn main() -> Result<(), Box<dyn Error>> {
    let output_path = env::args_os().nth(1).map(PathBuf::from);
    println!("backend=skia-raster workload=synthetic-canvas size={WIDTH}x{HEIGHT}");
    println!("warmup_frames={WARMUP_FRAMES} measured_frames={MEASURED_FRAMES}");

    for tile_count in [256, 2_048, 10_000] {
        let scene = Scene::new(tile_count);
        let mut samples = Vec::with_capacity(MEASURED_FRAMES);
        let mut checksum = 0_u64;

        for frame in 0..(WARMUP_FRAMES + MEASURED_FRAMES) {
            let started = Instant::now();
            let pixels = render_rgba(&scene, Camera::default(), WIDTH, HEIGHT)?;
            let elapsed = started.elapsed();
            checksum = checksum_pixels(&pixels);
            if frame >= WARMUP_FRAMES {
                samples.push(elapsed);
            }
        }

        let p50 = percentile(&samples, 50);
        let p95 = percentile(&samples, 95);
        println!(
            "tiles={tile_count} p50_ms={:.3} p95_ms={:.3} checksum={checksum:016x}",
            as_millis(p50),
            as_millis(p95)
        );

        if tile_count == 2_048
            && let Some(path) = &output_path
        {
            let png = render_png(&scene, Camera::default(), WIDTH, HEIGHT)?;
            fs::write(path, png)?;
            println!("image={}", path.display());
        }
    }
    Ok(())
}

fn checksum_pixels(pixels: &[u8]) -> u64 {
    pixels.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let index = (sorted.len().saturating_sub(1) * percentile).div_ceil(100);
    sorted[index]
}

fn as_millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}
