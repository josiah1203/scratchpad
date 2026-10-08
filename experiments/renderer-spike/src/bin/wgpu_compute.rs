use std::{
    error::Error,
    sync::mpsc,
    time::{Duration, Instant},
};

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 1024;
const WARMUP_RUNS: usize = 1;
const MEASURED_RUNS: usize = 7;
const SHADER: &str = r"
struct Dimensions {
    width: u32,
    height: u32,
};

@group(0) @binding(0) var<storage, read> source: array<u32>;
@group(0) @binding(1) var<storage, read_write> destination: array<u32>;
@group(0) @binding(2) var<uniform> dimensions: Dimensions;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    if invocation.x >= dimensions.width || invocation.y >= dimensions.height {
        return;
    }
    let index = invocation.y * dimensions.width + invocation.x;
    let pixel = source[index];
    destination[index] = (pixel & 0xff000000u) | ((pixel ^ 0x00ffffffu) & 0x00ffffffu);
}
";

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn Error>> {
    let source_bytes = make_source_bytes(WIDTH, HEIGHT);
    if std::env::var_os("RENDERER_SPIKE_FORCE_CPU").is_some() {
        return run_cpu_fallback(&source_bytes, "forced_by_environment");
    }

    let instance = wgpu::Instance::default();
    let adapter_result =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }));
    let adapter = match adapter_result {
        Ok(adapter) => adapter,
        Err(error) => {
            return run_cpu_fallback(&source_bytes, &format!("adapter_unavailable: {error}"));
        }
    };
    let adapter_info = adapter.get_info();
    let device_result = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("renderer-spike-compute-device"),
        ..Default::default()
    }));
    let (device, queue) = match device_result {
        Ok(device) => device,
        Err(error) => {
            return run_cpu_fallback(&source_bytes, &format!("device_unavailable: {error}"));
        }
    };

    let pixel_count = u64::from(WIDTH) * u64::from(HEIGHT);
    let byte_count = pixel_count * 4;
    let source = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("synthetic-source-image"),
        size: byte_count,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&source, 0, &source_bytes);

    let destination = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("compute-output-image"),
        size: byte_count,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("compute-output-readback"),
        size: byte_count,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let dimensions = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("image-dimensions"),
        size: 8,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut dimensions_bytes = Vec::with_capacity(8);
    dimensions_bytes.extend_from_slice(&WIDTH.to_le_bytes());
    dimensions_bytes.extend_from_slice(&HEIGHT.to_le_bytes());
    queue.write_buffer(&dimensions, 0, &dimensions_bytes);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("invert-rgb-wgsl"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("invert-rgb-pipeline"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("invert-rgb-bind-group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: source.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: destination.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: dimensions.as_entire_binding() },
        ],
    });

    let mut samples = Vec::with_capacity(WARMUP_RUNS + MEASURED_RUNS);
    let mut output = Vec::new();
    for _ in 0..=MEASURED_RUNS {
        let started = Instant::now();
        let submission = dispatch(
            &device,
            &queue,
            &pipeline,
            &bind_group,
            &destination,
            &readback,
            (WIDTH, HEIGHT),
        );
        wait_for_mapping(&device, &readback, submission)?;
        let mapped = readback.slice(..).get_mapped_range()?;
        output.clear();
        output.extend_from_slice(&mapped);
        drop(mapped);
        readback.unmap();
        samples.push(started.elapsed());
    }

    let expected = invert_rgb(&source_bytes);
    if output != expected {
        return Err("GPU output did not match the deterministic CPU reference".into());
    }

    let measured = &samples[WARMUP_RUNS..];
    let p50 = percentile(measured, 50);
    let p95 = percentile(measured, 95);
    println!("adapter_name={}", adapter_info.name);
    println!("adapter_backend={:?}", adapter_info.backend);
    println!("adapter_device_type={:?}", adapter_info.device_type);
    println!("execution_mode=gpu_compute");
    println!("workload=1024x1024_rgba8_rgb_invert readback_included=true");
    println!("warmup_runs={WARMUP_RUNS} measured_runs={MEASURED_RUNS}");
    println!("p50_ms={:.3} p95_ms={:.3}", as_millis(p50), as_millis(p95));
    println!("reference_match=true");
    Ok(())
}

fn run_cpu_fallback(source_bytes: &[u8], reason: &str) -> Result<(), Box<dyn Error>> {
    for _ in 0..WARMUP_RUNS {
        let _ = invert_rgb(source_bytes);
    }

    let mut samples = Vec::with_capacity(MEASURED_RUNS);
    let mut reference_matches = true;
    for _ in 0..MEASURED_RUNS {
        let started = Instant::now();
        let output = invert_rgb(source_bytes);
        samples.push(started.elapsed());
        reference_matches &= is_rgb_inversion(source_bytes, &output);
    }

    if !reference_matches {
        return Err("CPU fallback did not match the RGB inversion contract".into());
    }

    let p50 = percentile(&samples, 50);
    let p95 = percentile(&samples, 95);
    println!("adapter_name=none");
    println!("adapter_backend=none");
    println!("adapter_device_type=none");
    println!("execution_mode=cpu_fallback");
    println!("gpu_fallback_reason={reason}");
    println!("workload=1024x1024_rgba8_rgb_invert readback_included=false");
    println!("warmup_runs={WARMUP_RUNS} measured_runs={MEASURED_RUNS}");
    println!("p50_ms={:.3} p95_ms={:.3}", as_millis(p50), as_millis(p95));
    println!("reference_match=true");
    Ok(())
}

fn dispatch(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::ComputePipeline,
    bind_group: &wgpu::BindGroup,
    destination: &wgpu::Buffer,
    readback: &wgpu::Buffer,
    dimensions: (u32, u32),
) -> wgpu::SubmissionIndex {
    let (width, height) = dimensions;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("invert-rgb-command-encoder"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("invert-rgb-compute-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.dispatch_workgroups(width.div_ceil(16), height.div_ceil(16), 1);
    }
    encoder.copy_buffer_to_buffer(
        destination,
        0,
        readback,
        0,
        u64::from(width) * u64::from(height) * 4,
    );
    queue.submit([encoder.finish()])
}

fn wait_for_mapping(
    device: &wgpu::Device,
    readback: &wgpu::Buffer,
    submission: wgpu::SubmissionIndex,
) -> Result<(), Box<dyn Error>> {
    let (sender, receiver) = mpsc::channel();
    readback.slice(..).map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device.poll(wgpu::PollType::Wait {
        submission_index: Some(submission),
        timeout: Some(Duration::from_mins(1)),
    })?;
    receiver.recv()??;
    Ok(())
}

fn make_source_bytes(width: u32, height: u32) -> Vec<u8> {
    let byte_count = usize::try_from(u64::from(width) * u64::from(height) * 4)
        .expect("synthetic image byte count fits usize");
    let mut bytes = Vec::with_capacity(byte_count);
    for y in 0..height {
        for x in 0..width {
            let red = (x.wrapping_mul(13).wrapping_add(y * 3) & 0xff) as u8;
            let green = (y.wrapping_mul(17).wrapping_add(x / 2) & 0xff) as u8;
            let blue = (x.wrapping_add(y.wrapping_mul(7)) & 0xff) as u8;
            bytes.extend_from_slice(&[blue, green, red, 255]);
        }
    }
    bytes
}

fn invert_rgb(source: &[u8]) -> Vec<u8> {
    let mut result = source.to_vec();
    for pixel in result.chunks_exact_mut(4) {
        pixel[0] ^= 0xff;
        pixel[1] ^= 0xff;
        pixel[2] ^= 0xff;
    }
    result
}

fn is_rgb_inversion(source: &[u8], output: &[u8]) -> bool {
    source.len() == output.len()
        && source.len().is_multiple_of(4)
        && source.chunks_exact(4).zip(output.chunks_exact(4)).all(|(source, output)| {
            output[0] == !source[0]
                && output[1] == !source[1]
                && output[2] == !source[2]
                && output[3] == source[3]
        })
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len().saturating_sub(1) * percentile).div_ceil(100)]
}

fn as_millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

#[cfg(test)]
mod tests {
    use super::{invert_rgb, is_rgb_inversion};

    #[test]
    fn cpu_fallback_inverts_rgb_and_preserves_alpha() {
        let source = [0, 1, 254, 255, 17, 128, 240, 77];
        let output = invert_rgb(&source);

        assert_eq!(output, [255, 254, 1, 255, 238, 127, 15, 77]);
        assert!(is_rgb_inversion(&source, &output));
        assert!(!is_rgb_inversion(&source, &output[..4]));
    }
}
