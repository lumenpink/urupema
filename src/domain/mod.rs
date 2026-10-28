pub mod entity;
pub mod ports;
pub mod sanitizer;
pub mod segmentation;
pub mod status;

pub use entity::{ProcessedParts, RawEmail, SanitizedEmail};
pub use ports::{EmailSink, EmailSinkError};
pub use sanitizer::{is_markdown_empty_or_polluted, sanitize_markdown_noise};
pub use segmentation::{segment_email, EmailSegment, HeaderMeta, SegmentKind, SegmentationConfig};
pub use status::ProcessResult;
