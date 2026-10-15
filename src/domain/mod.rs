pub mod entity;
pub mod ports;
pub mod segmentation;
pub mod status;

pub use entity::{ProcessedParts, RawEmail, SanitizedEmail};
pub use ports::{EmailSink, EmailSinkError};
pub use segmentation::{
    EmailSegment, HeaderMeta, SegmentKind, SegmentationConfig, segment_email,
};
pub use status::ProcessResult;
