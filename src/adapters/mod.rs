pub mod cli;
pub mod in_memory;
pub mod sendmail;

pub use cli::CliArgs;
pub use in_memory::InMemorySink;
pub use sendmail::SendmailSink;
