pub mod entity;
pub mod ports;
pub mod status;

pub use entity::{ProcessedParts, RawEmail, SanitizedEmail};
pub use ports::{EmailSink, EmailSinkError};
pub use status::ProcessResult;
