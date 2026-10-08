use std::{
    env,
    fs::OpenOptions,
    io::{self, Write},
    path::PathBuf,
};

const WIDTH: usize = 16;
const HEIGHT: usize = 16;
const CHROMA_WIDTH: usize = WIDTH / 2;
const CHROMA_HEIGHT: usize = HEIGHT / 2;
const FRAME_LEVELS: [u8; 4] = [32, 96, 160, 224];
const HEADER: &[u8] = b"YUV4MPEG2 W16 H16 F24:1 Ip A1:1 C420jpeg\n";
const FRAME_MARKER: &[u8] = b"FRAME\n";
const DEFAULT_OUTPUT: &str =
    "tests/fixtures/synthetic-media/synthetic-gray-16x16-4frames-24fps.y4m";

fn main() -> io::Result<()> {
    let output_path =
        env::args_os().nth(1).map_or_else(|| PathBuf::from(DEFAULT_OUTPUT), PathBuf::from);
    let mut bytes = Vec::with_capacity(
        HEADER.len() + FRAME_LEVELS.len() * (FRAME_MARKER.len() + WIDTH * HEIGHT * 3 / 2),
    );
    bytes.extend_from_slice(HEADER);

    for luma in FRAME_LEVELS {
        bytes.extend_from_slice(FRAME_MARKER);
        bytes.extend(std::iter::repeat_n(luma, WIDTH * HEIGHT));
        bytes.extend(std::iter::repeat_n(128, CHROMA_WIDTH * CHROMA_HEIGHT * 2));
    }

    let mut output = OpenOptions::new().write(true).create_new(true).open(&output_path)?;
    output.write_all(&bytes)?;
    output.sync_all()?;
    println!("wrote {} bytes to {}", bytes.len(), output_path.display());
    Ok(())
}
