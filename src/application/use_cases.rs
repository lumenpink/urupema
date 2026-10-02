use thiserror::Error;

use crate::domain::{EmailSink, EmailSinkError, ProcessResult, RawEmail, SanitizedEmail};

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Erro ao analisar mensagem RFC 5322: {0}")]
    ParseError(String),
    #[error("Erro de entrega no sink: {0}")]
    SinkError(#[from] EmailSinkError),
    #[error("Mensagem rejeitada pela política: {0}")]
    PolicyRejected(String),
}

/// Orquestrador do pipeline de higienização e entrega MIME do Urupema.
pub struct Pipeline;

impl Pipeline {
    /// Processa o e-mail bruto recebido e entrega via sink.
    pub fn process<S: EmailSink>(
        _raw: &RawEmail,
        _sender: &str,
        _recipient: &str,
        _sink: &mut S,
    ) -> Result<(SanitizedEmail, ProcessResult), PipelineError> {
        // TDD Etapa 3: Falha intencional antes da implementação concreta (DD-TDD)
        todo!("TDD: implement process_pipeline")
    }
}
