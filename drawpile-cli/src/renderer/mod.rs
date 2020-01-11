use dpcore::canvas::CanvasState;
use dpcore::protocol::{open_recording, Compatibility, ReadMessage};

use tracing::{info, warn};

use std::error::Error;
use std::fmt;
use std::io;
use std::time::{Duration, Instant};

use image;

pub struct RenderOpts<'a> {
    pub input_file: &'a str,
}

#[derive(Debug)]
struct RenderError {
    message: &'static str,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for RenderError {
    fn description(&self) -> &str {
        &self.message
    }
}

pub fn render_recording(opts: &RenderOpts) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = open_recording(opts.input_file)?;

    if reader.check_compatibility() == Compatibility::Incompatible {
        return Err(Box::new(RenderError {
            message: "Unsupported format version",
        }));
    }

    let start = Instant::now();
    let mut canvas = CanvasState::new();
    let mut total_render_time = Duration::new(0, 0);

    loop {
        match reader.read_next() {
            ReadMessage::Ok(m) => {
                let now = Instant::now();
                canvas.receive_message(&m);
                total_render_time += now.elapsed();
            }
            ReadMessage::Invalid(msg) => {
                warn!("Invalid message: {}", msg);
            }
            ReadMessage::IoError(e) => {
                return Err(Box::new(e));
            }
            ReadMessage::Eof => {
                break;
            }
        }
    }

    let now = Instant::now();
    let (img, w, h) = canvas.layerstack().to_image();
    save_image("test.png", &img, w, h)?;
    let total_save_time = now.elapsed();
    let total_time = start.elapsed();

    info!(
        "Total render time: {:.3} s",
        total_render_time.as_secs_f64()
    );
    info!("Total save time: {:.3} s", total_save_time.as_secs_f64());
    info!("Total time: {:.3} s", total_time.as_secs_f64());

    Ok(())
}

fn save_image(filename: &str, image: &[u32], width: u32, height: u32) -> io::Result<()> {
    assert_eq!(image.len(), width as usize * height as usize);
    let mut rgba = Vec::<u8>::with_capacity(width as usize * height as usize * 4);

    // TODO pixel data in layers should be stored in a better format
    for px in image.iter() {
        rgba.push(((px & 0x00ff0000) >> 16) as u8);
        rgba.push(((px & 0x0000ff00) >> 8) as u8);
        rgba.push(((px & 0x000000ff) >> 0) as u8);
        rgba.push(((px & 0xff000000) >> 24) as u8);
    }

    image::save_buffer(filename, &rgba, width, height, image::RGBA(8))
}
