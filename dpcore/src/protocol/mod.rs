pub mod message;
mod protover;
mod reader;
mod serialization;
mod textmessage;
mod textparser;
mod writer;

pub use message::{Message, VERSION};
pub use protover::ProtocolVersion;
pub use reader::{
    open_recording, BinaryReader, Compatibility, ReadMessage, RecordingReader, TextReader,
};
pub use serialization::DeserializationError;
pub use textparser::TextParser;
pub use writer::{BinaryWriter, RecordingWriter, TextWriter};
