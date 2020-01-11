use dpcore::protocol::{
    open_recording, BinaryWriter, Compatibility, ReadMessage, RecordingWriter, TextWriter,
};

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Write;

#[derive(Copy, Clone, PartialEq)]
pub enum Format {
    Guess,
    Binary,
    Text,
}

pub struct ConvertRecOpts<'a> {
    pub input_file: &'a str,
    pub output_file: &'a str,
    pub output_format: Format,
}

#[derive(Debug)]
struct ConversionError {
    message: &'static str,
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ConversionError {
    fn description(&self) -> &str {
        &self.message
    }
}

pub fn convert_recording(opts: &ConvertRecOpts) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = open_recording(opts.input_file)?;

    if reader.check_compatibility() == Compatibility::Incompatible {
        return Err(Box::new(ConversionError {
            message: "Unsupported format version",
        }));
    }

    let mut writer = write_recording(opts.output_file, opts.output_format)?;

    writer.write_header(reader.get_metadata_all())?;

    loop {
        match reader.read_next() {
            ReadMessage::Ok(m) => {
                writer.write_message(&m)?;
            }
            ReadMessage::Invalid(msg) => {
                eprintln!("Invalid message: {}", msg);
            }
            ReadMessage::IoError(e) => {
                return Err(Box::new(e));
            }
            ReadMessage::Eof => {
                break;
            }
        }
    }
    Ok(())
}

fn write_recording(filename: &str, format: Format) -> io::Result<Box<dyn RecordingWriter>> {
    if format == Format::Guess {
        let guess = if filename.ends_with(".dptxt") || filename == "-" {
            Format::Text
        } else {
            Format::Binary
        };
        return write_recording(filename, guess);
    }

    let file: Box<dyn Write> = if filename == "-" {
        Box::new(io::stdout())
    } else {
        Box::new(File::create(filename)?)
    };

    Ok(match format {
        Format::Guess => unreachable!(),
        Format::Binary => Box::new(BinaryWriter::open(file)),
        Format::Text => Box::new(TextWriter::open(file)),
    })
}
