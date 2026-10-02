use std::io::Write;
use std::process::{Command, Stdio};

use crate::domain::{EmailSink, EmailSinkError};

/// Adaptador concreto que despacha mensagens executando o binário sendmail via subprocesso.
#[derive(Debug, Clone)]
pub struct SendmailSink {
    sendmail_path: String,
}

impl SendmailSink {
    pub fn new(sendmail_path: impl Into<String>) -> Self {
        Self {
            sendmail_path: sendmail_path.into(),
        }
    }
}

impl Default for SendmailSink {
    fn default() -> Self {
        Self::new("/usr/sbin/sendmail")
    }
}

impl EmailSink for SendmailSink {
    fn deliver(
        &mut self,
        sender: &str,
        recipient: &str,
        payload: &[u8],
    ) -> Result<(), EmailSinkError> {
        let mut child = Command::new(&self.sendmail_path)
            .args(["-G", "-i", "-f", sender, recipient])
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(payload)?;
        }

        let status = child.wait()?;
        if !status.success() {
            let code = status.code().unwrap_or(1);
            return Err(EmailSinkError::CommandFailed(code));
        }

        Ok(())
    }
}
