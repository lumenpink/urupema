pub mod entity;
pub mod ports;
pub mod segmentation;
pub mod status;

pub use entity::{ProcessedParts, RawEmail, SanitizedEmail};
pub use ports::{EmailSink, EmailSinkError};
pub use segmentation::{segment_email, EmailSegment, HeaderMeta, SegmentKind, SegmentationConfig};
pub use status::ProcessResult;
