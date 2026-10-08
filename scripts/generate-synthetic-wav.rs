use std::{
    env,
    fs::OpenOptions,
    io::{self, Write},
    path::PathBuf,
};

const SAMPLE_RATE: u32 = 8_000;
const SAMPLE_COUNT: usize = 800;
const WAVEFORM: [i16; 8] = [0, 8_192, 11_585, 8_192, 0, -8_192, -11_585, -8_192];
const DEFAULT_OUTPUT: &str = "tests/fixtures/synthetic-media/synthetic-tone-1khz-8khz-mono.wav";

fn main() -> io::Result<()> {
    let output_path =
        env::args_os().nth(1).map_or_else(|| PathBuf::from(DEFAULT_OUTPUT), PathBuf::from);
    let data_len = u32::try_from(SAMPLE_COUNT * 2).expect("fixture is smaller than 4 GiB");
    let mut bytes = Vec::with_capacity(44 + usize::try_from(data_len).expect("valid data length"));

    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36_u32 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    bytes.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());

    for index in 0..SAMPLE_COUNT {
        bytes.extend_from_slice(&WAVEFORM[index % WAVEFORM.len()].to_le_bytes());
    }

    let mut output = OpenOptions::new().write(true).create_new(true).open(&output_path)?;
    output.write_all(&bytes)?;
    output.sync_all()?;
    println!("wrote {} bytes to {}", bytes.len(), output_path.display());
    Ok(())
}
